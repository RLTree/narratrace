fn write_review_artifact_for_run(
    session_dir: &Path,
    context_path: &Path,
    packet_path: Option<&Path>,
    voice_path: Option<&Path>,
    replay_plan_path: Option<&Path>,
    evidence_report_path: Option<&Path>,
    packet_inspection_path: Option<&Path>,
    dogfood_receipt_path: Option<&Path>,
    expected_run_id: Option<&str>,
) -> Result<ReviewArtifact> {
    let context = read_json(context_path).unwrap_or(Value::Null);
    let context_exists = regular_file_exists(context_path);
    let final_alignment = final_alignment_review_state(session_dir);
    let transcript_quality = transcript_quality_state(session_dir);
    let transcript_quality_chain = transcript_quality.chain_label();
    let diagnostics = context
        .get("alignmentDiagnostics")
        .cloned()
        .unwrap_or(Value::Null);
    let conflicts = context
        .pointer("/conflictDiagnostics/warnings")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let alignments = context
        .get("alignments")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let redaction_status = context
        .pointer("/redactionPolicy/status")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let claim_ceiling = diagnostics
        .get("claimCeiling")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let out_of_window_rnr_events = diagnostics
        .get("outOfWindowRecordReplayEvents")
        .and_then(Value::as_u64)
        .unwrap_or(0) as usize;
    let voice = voice_path
        .and_then(|path| read_json(path).ok())
        .unwrap_or(Value::Null);
    let voice_status = voice
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("not-generated");
    let voice_bindings = voice
        .get("segmentBindings")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0);
    let voice_execution_status = voice
        .pointer("/timelineBindingContract/executionStatus")
        .and_then(Value::as_str)
        .unwrap_or("not-generated");
    let voice_proof_obligations = voice
        .get("proofObligations")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0);
    let replay_plan = replay_plan_path
        .and_then(|path| read_json(path).ok())
        .unwrap_or(Value::Null);
    let replay_plan_status = replay_plan
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("not-generated");
    let replay_plan_cue_count = replay_plan
        .get("cueCount")
        .and_then(Value::as_u64)
        .unwrap_or(0) as usize;
    let replay_plan_speaks_audio = replay_plan
        .pointer("/proofBoundary/speaksAudio")
        .and_then(Value::as_bool);
    let replay_plan_exists = replay_plan_path.is_some_and(regular_file_exists);
    let replay_plan_valid = replay_voice_plan_valid(&replay_plan);
    let replay_plan_speaks_audio_label = replay_plan_speaks_audio
        .map(|speaks| speaks.to_string())
        .unwrap_or_else(|| "not-generated".to_string());
    let evidence_report_exists = evidence_report_path.is_some_and(regular_file_exists);
    let packet_inspection = packet_inspection_path
        .map(read_packet_inspection)
        .unwrap_or(Value::Null);
    let packet_inspection_status = inspection_status(&packet_inspection);
    let narration_density_status = inspection::narration_density_status(&packet_inspection);
    let transcript_word_count = inspection::transcript_word_count(&packet_inspection);
    let transcript_char_count = inspection::transcript_char_count(&packet_inspection);
    let leak_status = leak_scan_status(&packet_inspection);
    let leak_count = leak_finding_count(&packet_inspection);
    let blocking_leak_count = blocking_leak_finding_count(&packet_inspection);
    let leak_categories = leak_categories(&packet_inspection);
    let raw_local_sensitive_count = raw_local_sensitive_artifact_count(&packet_inspection);
    let raw_local_sensitive_categories = raw_local_sensitive_categories(&packet_inspection);
    let dogfood_receipt = dogfood_receipt_path
        .and_then(|path| read_json(path).ok())
        .unwrap_or(Value::Null);
    let dogfood_receipt_status = dogfood_receipt
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("not-generated");
    let proofs_valid = review_proofs_valid(session_dir, expected_run_id);
    let capture_helper_state = dogfood_receipt
        .pointer("/capture/helperState")
        .and_then(Value::as_str)
        .unwrap_or("not-generated");
    let capture_audio_input = capture_audio_input_label(&dogfood_receipt);
    let post_commit_completed_segments = dogfood_receipt
        .pointer("/capture/postCommitDrain/completedSegments")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let post_commit_messages = dogfood_receipt
        .pointer("/capture/postCommitDrain/messages")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let post_commit_error_count = dogfood_receipt
        .pointer("/capture/postCommitDrain/errors")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let stop_timeout_status = dogfood_receipt
        .pointer("/capture/stopTimeout/status")
        .and_then(Value::as_str)
        .unwrap_or("not-present");
    let narration_density_sparse = narration_density_status == "too-sparse-for-non-toy-replay";
    let status = review_status(
        context_exists,
        conflicts.len(),
        blocking_leak_count,
        raw_local_sensitive_count,
        dogfood_receipt_path.is_some_and(regular_file_exists),
        replay_plan_speaks_audio,
        replay_plan_exists,
        replay_plan_valid,
        narration_density_sparse,
        transcript_quality.is_complete(),
        proofs_valid,
    );
    let recovery = recovery_actions(
        context_exists,
        conflicts.len(),
        evidence_report_exists,
        replay_plan_exists,
        replay_plan_speaks_audio,
        blocking_leak_count,
        raw_local_sensitive_count,
        dogfood_receipt_path.is_some_and(regular_file_exists),
        out_of_window_rnr_events,
        narration_density_sparse,
        transcript_quality.is_complete(),
    );
    let review_path = session_dir.join("review-artifact.html");
    write_private(
        &review_path,
        html::render(&ReviewHtmlInput {
            session_dir,
            context_path,
            packet_path,
            voice_path,
            replay_plan_path,
            evidence_report_path,
            packet_inspection_path,
            dogfood_receipt_path,
            claim_ceiling,
            status,
            redaction_status,
            voice_status,
            voice_execution_status,
            replay_plan_status,
            replay_plan_speaks_audio: &replay_plan_speaks_audio_label,
            packet_inspection_status,
            dogfood_receipt_status,
            final_alignment_status: &final_alignment.status,
            final_alignment_word_authority: &final_alignment.word_authority,
            final_alignment_unresolved_mismatches: final_alignment.unresolved_mismatches,
            transcript_quality_chain: &transcript_quality_chain,
            narration_density_status,
            transcript_word_count,
            transcript_char_count,
            capture_helper_state,
            capture_audio_input: &capture_audio_input,
            post_commit_completed_segments,
            post_commit_messages,
            post_commit_error_count,
            stop_timeout_status,
            leak_status,
            leak_count,
            leak_categories: &leak_categories,
            raw_local_sensitive_count,
            raw_local_sensitive_categories: &raw_local_sensitive_categories,
            alignment_count: alignments.len(),
            conflict_count: conflicts.len(),
            diagnostics: &diagnostics,
            voice_bindings,
            voice_proof_obligations,
            replay_plan_cue_count,
            recovery: &recovery,
            conflicts: &conflicts,
        }),
    )?;
    let contract_path = write_review_contract(
        session_dir,
        context_path,
        packet_path,
        voice_path,
        replay_plan_path,
        evidence_report_path,
        packet_inspection_path,
        dogfood_receipt_path,
        &replay_plan,
        &packet_inspection,
        &dogfood_receipt,
        &context,
        &diagnostics,
        alignments.len(),
        conflicts.len(),
        voice_bindings,
        voice_execution_status,
        voice_proof_obligations,
        replay_plan_status,
        replay_plan_cue_count,
        replay_plan_speaks_audio,
        blocking_leak_count,
        raw_local_sensitive_count,
        &raw_local_sensitive_categories,
        out_of_window_rnr_events,
        narration_density_status,
        transcript_word_count,
        transcript_char_count,
        &final_alignment.status,
        &final_alignment.word_authority,
        final_alignment.unresolved_mismatches,
        &transcript_quality,
        proofs_valid,
    )?;
    Ok(ReviewArtifact {
        html_path: review_path,
        contract_path,
    })
}
