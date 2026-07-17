fn refresh_review_surface(args: &Args, session_dir: &Path) -> Result<()> {
    let context_path = session_dir.join("temporal-context.json");
    let packet_path = session_dir.join("skill-refinement-packet.md");
    let voice_path = session_dir.join("replay-voice-parameters.json");
    let replay_plan_path = session_dir.join("replay-voice-execution-plan.json");
    let evidence_report_path = session_dir.join("evidence-boundary-report.json");
    let packet_inspection_path = session_dir.join("packet-inspection.json");
    let dogfood_receipt_path = session_dir.join("dogfood-receipt.json");
    review::write_review_artifact_for_receipt(
        session_dir,
        &context_path,
        regular_file_exists(&packet_path).then_some(packet_path.as_path()),
        regular_file_exists(&voice_path).then_some(voice_path.as_path()),
        regular_file_exists(&replay_plan_path).then_some(replay_plan_path.as_path()),
        regular_file_exists(&evidence_report_path).then_some(evidence_report_path.as_path()),
        regular_file_exists(&packet_inspection_path).then_some(packet_inspection_path.as_path()),
        regular_file_exists(&dogfood_receipt_path).then_some(dogfood_receipt_path.as_path()),
        args.receipt_run_id.as_deref(),
    )?;
    Ok(())
}

fn regular_file_exists(path: &Path) -> bool {
    regular_file_metadata(path).is_ok()
}

fn blockers(
    args: &Args,
    manifest: &Value,
    status: &Value,
    temporal: &Value,
    evidence: &Value,
    inspection: &Value,
    post_commit_drain: &Value,
    parent_operation: &Value,
    final_alignment: &Value,
) -> Vec<&'static str> {
    let mut blockers = Vec::new();
    if status.get("state").and_then(Value::as_str) != Some("stopped") {
        blockers.push("narration helper did not stop cleanly");
    }
    if manifest.pointer("/startCoordination/recordReplayAndMicrophoneSameOperation")
        != Some(&Value::Bool(true))
    {
        blockers.push(
            "Record & Replay and microphone capture were not started by one coordinated operation",
        );
    }
    if !parent_operation_receipt_matches_current_artifacts(args, parent_operation) {
        blockers
            .push("parent operation receipt is missing or mismatches current Record & Replay/audio artifacts");
    }
    if args
        .receipt_run_id
        .as_deref()
        .is_none_or(|run_id| run_id.trim().is_empty())
    {
        blockers.push("trusted current-run receipt id is missing");
    }
    if !current_run_proof_shapes_valid(
        manifest,
        status,
        temporal,
        evidence,
        inspection,
        post_commit_drain,
        final_alignment,
    ) {
        blockers.push("current-run proof schemas or producer statuses are invalid");
    }
    blockers.push("trusted external current-run execution attestation is unavailable");
    if args.recording_metadata.is_none() {
        blockers.push("Record & Replay metadata path is required for live capture proof");
    } else if !artifact_is_nonempty_file(args.recording_metadata.as_deref()) {
        blockers.push("Record & Replay metadata path must exist as a non-empty file");
    }
    if args.recording_events.is_none() {
        blockers.push("Record & Replay events path is required for live capture proof");
    } else if !artifact_is_nonempty_file(args.recording_events.as_deref()) {
        blockers.push("Record & Replay events path must exist as a non-empty file");
    }
    if evidence.pointer("/evidenceSurfaces/audioClockPresent") != Some(&Value::Bool(true)) {
        blockers.push("audio clock anchor is missing or unproven");
    }
    if first_u64(&[temporal.pointer("/transcriptSegments")]).unwrap_or(0) == 0 {
        blockers.push("no transcript segments are present");
    }
    if !post_commit_drain.is_object() {
        blockers.push("post-commit transcription drain receipt is missing");
    } else {
        if max_u64(&[
            post_commit_drain.pointer("/completedSegments"),
            post_commit_drain.pointer("/captureStats/realtimeCompletedSegmentsObserved"),
        ])
        .unwrap_or(0)
            == 0
        {
            blockers.push("post-commit transcription drain did not complete any segments");
        }
        if first_u64(&[post_commit_drain.pointer("/errors")]).unwrap_or(0) > 0 {
            blockers.push("post-commit transcription drain recorded errors");
        }
    }
    if first_u64(&[temporal.pointer("/recordReplayEvents")]).unwrap_or(0) == 0 {
        blockers.push("no Record & Replay events are present");
    }
    if final_alignment.get("status").and_then(Value::as_str) != Some("aligned") {
        blockers.push("final transcript alignment is missing or has unresolved mismatches");
    }
    if final_alignment
        .get("unresolvedMismatches")
        .and_then(Value::as_u64)
        .unwrap_or(1)
        > 0
    {
        blockers.push("final transcript alignment has unresolved mismatches");
    }
    if inspection.get("status").and_then(Value::as_str).is_none() {
        blockers.push("packet inspection has not been run");
    }
    if inspection
        .pointer("/privacyBoundary/generatedArtifactLeakScan/status")
        .and_then(Value::as_str)
        == Some("blocked")
    {
        blockers.push("generated artifact leak scan is blocked");
    }
    if inspection.pointer("/privacyBoundary/allowedToShareWithoutReview")
        != Some(&Value::Bool(false))
    {
        blockers.push("privacy boundary has not been inspected");
    }
    blockers.push("operator review of generated artifacts is still required");
    blockers
}

fn artifact_is_nonempty_file(path: Option<&str>) -> bool {
    path.map(Path::new)
        .and_then(|path| regular_file_metadata(path).ok())
        .is_some_and(|metadata| metadata.len() > 0)
}

fn sensitive_raw_local_artifacts(inspection: &Value) -> Value {
    let Some(raw_local) = inspection
        .pointer("/privacyBoundary/rawLocalOnly")
        .and_then(Value::as_array)
    else {
        return Value::Null;
    };
    Value::Array(
        raw_local
            .iter()
            .filter(|artifact| {
                artifact
                    .get("containsSensitivePatterns")
                    .and_then(Value::as_bool)
                    == Some(true)
            })
            .cloned()
            .collect(),
    )
}

fn parent_operation_receipt_matches_current_artifacts(
    args: &Args,
    parent_operation: &Value,
) -> bool {
    let Ok(session_dir) = required_session_dir(args) else {
        return false;
    };
    let (Some(metadata_path), Some(events_path)) = (
        args.recording_metadata.as_deref(),
        args.recording_events.as_deref(),
    ) else {
        return false;
    };
    let Ok(evaluation) = evaluate_parent_operation(&session_dir, metadata_path, events_path) else {
        return false;
    };
    let Some(expected_run_id) = args
        .receipt_run_id
        .as_deref()
        .filter(|id| !id.trim().is_empty())
    else {
        return false;
    };
    let Some(actual) =
        crate::parent_operation::ParentOperationBinding::from_receipt(parent_operation)
    else {
        return false;
    };
    let expected = crate::parent_operation::ParentOperationBinding::from_evaluation(
        &evaluation,
        Some(expected_run_id),
    );
    parent_operation.get("status").and_then(Value::as_str) == Some(evaluation.status_text.as_str())
        && actual == expected
}

fn first_u64(values: &[Option<&Value>]) -> Option<u64> {
    for value in values.iter().flatten() {
        if let Some(items) = value.as_array() {
            return Some(items.len() as u64);
        }
        if let Some(number) = value.as_u64() {
            return Some(number);
        }
    }
    None
}

fn post_commit_completed_segments(post_commit_drain: &Value) -> Option<u64> {
    max_u64(&[
        post_commit_drain.pointer("/completedSegments"),
        post_commit_drain.pointer("/captureStats/realtimeCompletedSegmentsObserved"),
    ])
}

fn max_u64(values: &[Option<&Value>]) -> Option<u64> {
    values
        .iter()
        .flatten()
        .filter_map(|value| {
            value
                .as_array()
                .map(|items| items.len() as u64)
                .or_else(|| value.as_u64())
        })
        .max()
}

fn read_json(path: &Path) -> Result<Value> {
    let text = crate::safe_path::read_regular_text_bounded(path, MAX_RECEIPT_JSON_BYTES)?;
    Ok(serde_json::from_str(&text)?)
}
