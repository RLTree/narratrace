fn current_run_proof_shapes_valid(
    manifest: &Value,
    status: &Value,
    temporal: &Value,
    evidence: &Value,
    inspection: &Value,
    post_commit_drain: &Value,
    final_alignment: &Value,
) -> bool {
    manifest.get("schema").and_then(Value::as_str) == Some("narrated-record-replay.v1")
        && manifest.pointer("/startCoordination/recordReplayAndMicrophoneSameOperation")
            == Some(&Value::Bool(true))
        && status.get("state").and_then(Value::as_str) == Some("stopped")
        && temporal.get("schema").and_then(Value::as_str)
            == Some("narrated-record-replay.temporal-context.v1")
        && temporal
            .get("transcriptSegments")
            .and_then(Value::as_array)
            .is_some_and(|rows| !rows.is_empty())
        && temporal
            .get("recordReplayEvents")
            .and_then(Value::as_array)
            .is_some_and(|rows| !rows.is_empty())
        && evidence.get("schema").and_then(Value::as_str)
            == Some("narrated-record-replay.evidence-boundary-report.v1")
        && evidence.pointer("/evidenceSurfaces/audioClockPresent") == Some(&Value::Bool(true))
        && post_commit_drain.get("schema").and_then(Value::as_str)
            == Some("narrated-record-replay.post-commit-drain.v1")
        && post_commit_completed_segments(post_commit_drain).unwrap_or(0) > 0
        && post_commit_drain
            .get("errors")
            .and_then(Value::as_array)
            .is_some_and(Vec::is_empty)
        && final_alignment.get("schema").and_then(Value::as_str)
            == Some("narrated-record-replay.final-transcript-alignment.v1")
        && final_alignment.get("status").and_then(Value::as_str) == Some("aligned")
        && final_alignment
            .get("unresolvedMismatches")
            .and_then(Value::as_u64)
            == Some(0)
        && packet_inspection_proof_valid(inspection)
}

fn packet_inspection_proof_valid(inspection: &Value) -> bool {
    inspection.get("schema").and_then(Value::as_str)
        == Some("narrated-record-replay.packet-inspection.v1")
        && inspection.get("status").and_then(Value::as_str) == Some("requires-real-packet-review")
        && inspection.pointer("/privacyBoundary/allowedToShareWithoutReview")
            == Some(&Value::Bool(false))
        && inspection
            .pointer("/privacyBoundary/generatedArtifactLeakScan/status")
            .and_then(Value::as_str)
            .is_some_and(|status| {
                matches!(
                    status,
                    "no-obvious-sensitive-patterns-detected" | "expected-local-references-only"
                )
            })
        && inspection
            .pointer("/privacyBoundary/generatedArtifactLeakScan/findings")
            .and_then(Value::as_array)
            .is_some()
}

fn generated_review_artifacts(session_dir: &Path) -> Vec<artifacts::Artifact> {
    [
        ("skill-refinement-packet", "skill-refinement-packet.md"),
        ("timestamped-notes", "timestamped-notes.md"),
        ("thought-process", "thought-process.md"),
        ("temporal-context", "temporal-context.json"),
        ("evidence-boundary-report", "evidence-boundary-report.json"),
        ("packet-inspection", "packet-inspection.json"),
        (
            "replay-voice-execution-plan",
            "replay-voice-execution-plan.json",
        ),
        (
            "batch-transcription-receipt",
            "batch-transcription-receipt.json",
        ),
        ("cleanup-receipt", "cleanup-receipt.json"),
        (
            "final-transcript-alignment-receipt",
            "final-transcript-alignment-receipt.json",
        ),
        (
            "final-transcript-alignment",
            "final-transcript-alignment.json",
        ),
        ("review-contract", "review-contract.json"),
        ("review-artifact", "review-artifact.html"),
    ]
    .into_iter()
    .map(|(name, file)| artifact(name, session_dir.join(file)))
    .collect()
}

fn digest_bindings_complete(recording: &[Value], raw_local: &[Value], generated: &[Value]) -> bool {
    entries_have_digests(recording, &["recording-metadata", "recording-events"])
        && entries_have_digests(
            raw_local,
            &[
                "manifest",
                "capture-status",
                "capture-clock",
                "post-commit-drain",
                "parent-operation-receipt",
            ],
        )
        && entries_have_digests(
            generated,
            &[
                "temporal-context",
                "evidence-boundary-report",
                "packet-inspection",
                "final-transcript-alignment",
            ],
        )
}

fn entries_have_digests(entries: &[Value], required_names: &[&str]) -> bool {
    required_names.iter().all(|name| {
        entries.iter().any(|entry| {
            entry.get("name").and_then(Value::as_str) == Some(name)
                && entry.get("exists").and_then(Value::as_bool) == Some(true)
                && entry.get("isRegularFile").and_then(Value::as_bool) == Some(true)
                && entry
                    .get("contentFingerprint")
                    .and_then(Value::as_str)
                    .is_some_and(|digest| digest.starts_with("sha256:") && digest.len() == 71)
        })
    })
}

#[allow(clippy::too_many_arguments)]
fn current_run_values_unchanged(
    session_dir: &Path,
    manifest: &Value,
    status: &Value,
    temporal: &Value,
    evidence: &Value,
    inspection: &Value,
    post_commit_drain: &Value,
    parent_operation: &Value,
    final_alignment: &Value,
) -> bool {
    [
        ("manifest.json", manifest),
        ("status.json", status),
        ("temporal-context.json", temporal),
        ("evidence-boundary-report.json", evidence),
        ("packet-inspection.json", inspection),
        ("post-commit-drain.json", post_commit_drain),
        ("parent-operation-receipt.json", parent_operation),
        ("final-transcript-alignment.json", final_alignment),
    ]
    .iter()
    .all(|(file, expected)| {
        read_json(&session_dir.join(file)).is_ok_and(|current| current == **expected)
    })
}
