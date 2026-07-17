fn product_cohesion_review(
    context_exists: bool,
    packet_exists: bool,
    evidence_report_exists: bool,
    replay_plan_exists: bool,
    replay_plan_speaks_audio: Option<bool>,
    replay_plan_valid: bool,
    packet_inspection_exists: bool,
    dogfood_receipt_exists: bool,
    conflict_count: usize,
    leak_finding_count: usize,
    raw_local_sensitive_artifact_count: usize,
    narration_density_status: &str,
    transcript_quality_complete: bool,
    review_proofs_valid: bool,
) -> Value {
    serde_json::json!({
        "status": "fixture-contract-review-only",
        "claimCeiling": "static contract inspection only; real packet product-cohesion review is still owed",
        "checkedSurfaces": [
            "review state separates transcript context from UI evidence",
            "redaction status is visible before sharing",
            "generated-artifact leak scan status is visible before sharing",
            "raw-local category-only sensitivity summary is visible before sharing",
            "dogfood receipt status is visible for live proof review",
            "recovery actions are present for missing artifacts and conflicts",
            "replay voice parameters remain marked as planned, not executed",
            "replay voice execution status and proof obligations are visible",
            "replay voice dry-run execution plan status and cue count are visible",
            "replay voice preview audio boundary is visible",
            "narration density status and transcript counts are visible"
        ],
        "blockers": product_cohesion_blockers(
            context_exists,
            packet_exists,
            evidence_report_exists,
            replay_plan_exists,
            replay_plan_speaks_audio,
            replay_plan_valid,
            packet_inspection_exists,
            dogfood_receipt_exists,
            conflict_count,
            leak_finding_count,
            raw_local_sensitive_artifact_count,
            narration_density_status,
            transcript_quality_complete,
            review_proofs_valid,
        )
    })
}

fn product_cohesion_blockers(
    context_exists: bool,
    packet_exists: bool,
    evidence_report_exists: bool,
    replay_plan_exists: bool,
    replay_plan_speaks_audio: Option<bool>,
    replay_plan_valid: bool,
    packet_inspection_exists: bool,
    dogfood_receipt_exists: bool,
    conflict_count: usize,
    leak_finding_count: usize,
    raw_local_sensitive_artifact_count: usize,
    narration_density_status: &str,
    transcript_quality_complete: bool,
    review_proofs_valid: bool,
) -> Vec<&'static str> {
    let mut blockers = Vec::new();
    if !context_exists {
        blockers.push("temporal context missing");
    }
    if !packet_exists {
        blockers.push("skill refinement packet missing");
    }
    if !evidence_report_exists {
        blockers.push("evidence boundary report missing");
    }
    if !replay_plan_exists {
        blockers.push("replay voice dry-run execution plan missing");
    } else if !replay_plan_valid {
        blockers.push("replay voice dry-run execution plan is malformed or incomplete");
    }
    if replay_plan_speaks_audio.unwrap_or(false) {
        blockers.push("replay voice execution plan is not marked as no-audio dry-run");
    }
    if !packet_inspection_exists {
        blockers.push("packet inspection missing");
    }
    if !dogfood_receipt_exists {
        blockers.push("dogfood receipt missing for live proof review");
    }
    if conflict_count > 0 {
        blockers.push("conflict warnings need operator review");
    }
    if leak_finding_count > 0 {
        blockers.push("generated artifact leak findings need privacy review");
    }
    if raw_local_sensitive_artifact_count > 0 {
        blockers.push(
            "raw-local transcript artifacts contain sensitive categories requiring privacy review",
        );
    }
    if narration_density_status == "too-sparse-for-non-toy-replay" {
        blockers.push("narration density is too sparse for confident packet reuse");
    }
    if !transcript_quality_complete {
        blockers.push("transcript quality pipeline is incomplete or disabled");
    }
    if !review_proofs_valid {
        blockers.push("review proofs are malformed, stale, self-authored, or unbound");
    }
    blockers.push("real non-toy packet product-cohesion review still owed");
    blockers.push("replay voice execution receipt still owed");
    blockers
}

pub(super) fn review_status(
    context_exists: bool,
    conflict_count: usize,
    leak_finding_count: usize,
    raw_local_sensitive_artifact_count: usize,
    dogfood_receipt_exists: bool,
    replay_plan_speaks_audio: Option<bool>,
    replay_plan_exists: bool,
    replay_plan_valid: bool,
    narration_density_sparse: bool,
    transcript_quality_complete: bool,
    review_proofs_valid: bool,
) -> &'static str {
    if !context_exists {
        "blocked-missing-temporal-context"
    } else if leak_finding_count > 0
        || raw_local_sensitive_artifact_count > 0
        || !dogfood_receipt_exists
        || !replay_plan_exists
        || !replay_plan_valid
        || replay_plan_speaks_audio.unwrap_or(false)
        || narration_density_sparse
        || !transcript_quality_complete
        || !review_proofs_valid
    {
        "blocked"
    } else if conflict_count > 0 {
        "requires-operator-review"
    } else {
        "requires-operator-review"
    }
}

pub(super) fn replay_voice_plan_valid(replay_plan: &Value) -> bool {
    replay_plan.get("schema").and_then(Value::as_str)
        == Some("narrated-record-replay.replay-voice-execution-plan.v1")
        && replay_plan.get("status").and_then(Value::as_str) == Some("dry-run-not-spoken")
        && replay_plan
            .get("cueCount")
            .and_then(Value::as_u64)
            .is_some_and(|count| {
                replay_plan
                    .get("cues")
                    .and_then(Value::as_array)
                    .is_some_and(|cues| count == cues.len() as u64)
            })
        && replay_plan
            .pointer("/proofBoundary/speaksAudio")
            .and_then(Value::as_bool)
            == Some(false)
        && replay_plan.get("cues").and_then(Value::as_array).is_some()
}

pub(super) fn recovery_actions(
    context_exists: bool,
    conflict_count: usize,
    evidence_report_exists: bool,
    replay_plan_exists: bool,
    replay_plan_speaks_audio: Option<bool>,
    leak_finding_count: usize,
    raw_local_sensitive_artifact_count: usize,
    dogfood_receipt_exists: bool,
    out_of_window_rnr_event_count: usize,
    narration_density_sparse: bool,
    transcript_quality_complete: bool,
) -> Vec<&'static str> {
    let mut actions = Vec::new();
    if !context_exists {
        actions.push("Run packet generation first so temporal-context.json exists.");
    }
    if !evidence_report_exists {
        actions.push("Regenerate the packet so evidence-boundary-report.json exists.");
    }
    if !replay_plan_exists {
        actions.push(
            "Run replay-voice-preview after packet generation to create a dry-run execution plan.",
        );
    }
    if replay_plan_speaks_audio.unwrap_or(false) {
        actions.push(
            "Do not treat replay voice preview as proof unless it is a no-audio dry-run or has a live replay receipt.",
        );
    }
    if conflict_count > 0 {
        actions.push("Inspect conflict warnings before converting transcript action claims into replay steps.");
    }
    if out_of_window_rnr_event_count > 0 {
        actions.push("Inspect out-of-window Record & Replay events; boundary events can be expected, but broad UI-event drift weakens replay confidence.");
    }
    if leak_finding_count > 0 {
        actions.push("Inspect generated artifact leak scan before sharing or durable reuse.");
    }
    if raw_local_sensitive_artifact_count > 0 {
        actions.push("Inspect raw-local sensitive categories before sharing or durable reuse.");
    }
    if narration_density_sparse {
        actions.push("Run another coordinated dogfood with denser narration before reusing the packet for confident replay refinement.");
    }
    if !transcript_quality_complete {
        actions.push("Regenerate the packet with post-stop batch transcription, cleanup, and final alignment enabled before trusting final words.");
    }
    if !dogfood_receipt_exists {
        actions.push("Run receipt after packet inspection to summarize dogfood artifacts without raw transcript text.");
    }
    if actions.is_empty() {
        actions.push(
            "Inspect the static review artifact and evidence boundary report before durable skill refinement.",
        );
    }
    actions
}
