use serde_json::Value;
use std::path::Path;

pub(super) struct ReviewHtmlInput<'a> {
    pub session_dir: &'a Path,
    pub context_path: &'a Path,
    pub packet_path: Option<&'a Path>,
    pub voice_path: Option<&'a Path>,
    pub replay_plan_path: Option<&'a Path>,
    pub evidence_report_path: Option<&'a Path>,
    pub packet_inspection_path: Option<&'a Path>,
    pub dogfood_receipt_path: Option<&'a Path>,
    pub claim_ceiling: &'a str,
    pub status: &'a str,
    pub redaction_status: &'a str,
    pub voice_status: &'a str,
    pub voice_execution_status: &'a str,
    pub replay_plan_status: &'a str,
    pub replay_plan_speaks_audio: &'a str,
    pub packet_inspection_status: &'a str,
    pub dogfood_receipt_status: &'a str,
    pub final_alignment_status: &'a str,
    pub final_alignment_word_authority: &'a str,
    pub final_alignment_unresolved_mismatches: u64,
    pub transcript_quality_chain: &'a str,
    pub narration_density_status: &'a str,
    pub transcript_word_count: u64,
    pub transcript_char_count: u64,
    pub capture_helper_state: &'a str,
    pub capture_audio_input: &'a str,
    pub post_commit_completed_segments: u64,
    pub post_commit_messages: u64,
    pub post_commit_error_count: u64,
    pub stop_timeout_status: &'a str,
    pub leak_status: &'a str,
    pub leak_count: usize,
    pub leak_categories: &'a [String],
    pub raw_local_sensitive_count: usize,
    pub raw_local_sensitive_categories: &'a [String],
    pub alignment_count: usize,
    pub conflict_count: usize,
    pub diagnostics: &'a Value,
    pub voice_bindings: usize,
    pub voice_proof_obligations: usize,
    pub replay_plan_cue_count: usize,
    pub recovery: &'a [&'static str],
    pub conflicts: &'a [Value],
}

pub(super) fn render(input: &ReviewHtmlInput<'_>) -> String {
    format!(
        concat!(
            "<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n",
            "<title>Narrated Record & Replay Review</title>\n",
            "<style>body{{font-family:system-ui,sans-serif;line-height:1.45;margin:2rem;max-width:980px}}code,pre{{background:#f5f5f5}}section{{border-top:1px solid #ddd;padding-top:1rem;margin-top:1rem}}.warn{{color:#8a4b00}}.ok{{color:#116329}}</style>\n",
            "</head>\n<body>\n<h1>Narrated Record & Replay Review</h1>\n",
            "<section>\n<h2>Artifacts</h2>\n<ul>\n",
            "<li>Session: <code>{}</code></li>\n",
            "<li>Temporal context: <code>{}</code></li>\n",
            "<li>Skill packet: <code>{}</code></li>\n",
            "<li>Replay voice parameters: <code>{}</code></li>\n",
            "<li>Replay voice execution plan: <code>{}</code></li>\n",
            "<li>Evidence boundary report: <code>{}</code></li>\n",
            "<li>Packet inspection: <code>{}</code></li>\n",
            "<li>Dogfood receipt: <code>{}</code></li>\n",
            "</ul>\n</section>\n",
            "<section>\n<h2>Claim Ceiling</h2>\n",
            "<p>{}</p>\n",
            "<p>Review state: <strong>{}</strong></p>\n",
            "<p>Redaction status: <strong>{}</strong></p>\n",
            "<p>Replay voice status: <strong>{}</strong></p>\n",
            "<p>Replay voice execution status: <strong>{}</strong></p>\n",
            "<p>Replay voice preview status: <strong>{}</strong></p>\n",
            "<p>Replay voice preview speaks audio: <strong>{}</strong></p>\n",
            "<p>Packet inspection status: <strong>{}</strong></p>\n",
            "<p>Dogfood receipt status: <strong>{}</strong></p>\n",
            "<p>Final transcript alignment: <strong>{}</strong></p>\n",
            "<p>Transcript word authority: <strong>{}</strong></p>\n",
            "<p>Final transcript unresolved mismatches: <strong>{}</strong></p>\n",
            "<p>Transcript quality receipts: <strong>{}</strong></p>\n",
            "<p>Narration density status: <strong>{}</strong></p>\n",
            "<p>Transcript words/chars: <strong>{}/{}</strong></p>\n",
            "<p>Generated artifact leak scan: <strong>{}</strong> (findings: {})</p>\n",
            "<p>Leak categories: {}</p>\n",
            "<p>Raw-local sensitive artifacts: <strong>{}</strong></p>\n",
            "<p>Raw-local sensitive categories: {}</p>\n",
            "</section>\n",
            "<section>\n<h2>Capture State</h2>\n<ul>\n",
            "<li>Helper state: <strong>{}</strong></li>\n",
            "<li>Audio input: <code>{}</code></li>\n",
            "<li>Post-commit completed transcript segments: {}</li>\n",
            "<li>Post-commit messages: {}</li>\n",
            "<li>Post-commit errors: {}</li>\n",
            "<li>Stop timeout status: <strong>{}</strong></li>\n",
            "</ul>\n</section>\n",
            "<section>\n<h2>Summary</h2>\n<ul>\n",
            "<li>Alignments: {}</li>\n",
            "<li>Conflict warnings: {}</li>\n",
            "<li>Malformed timestamps: {}</li>\n",
            "<li>Events without timestamps: {}</li>\n",
            "<li>Out-of-window events: {}</li>\n",
            "<li>Replay voice segment bindings: {}</li>\n",
            "<li>Replay voice proof obligations: {}</li>\n",
            "<li>Replay voice preview cues: {}</li>\n",
            "</ul>\n</section>\n",
            "<section>\n<h2>Recovery Actions</h2>\n{}\n</section>\n",
            "<section>\n<h2>Review Checklist</h2>\n<ul>\n",
            "<li>Observed UI events must be treated as action evidence.</li>\n",
            "<li>Transcript text is intent/context unless the UI event directly proves it.</li>\n",
            "<li>Warnings below require operator inspection before converting transcript action claims into replay steps.</li>\n",
            "<li>Inspect the evidence boundary report before durable skill refinement.</li>\n",
            "<li>Inspect dogfood receipts for artifact completeness before live-claim closeout.</li>\n",
            "<li>Inspect generated artifact leak scan before sharing or durable reuse.</li>\n",
            "<li>Inspect raw-local sensitive categories before sharing or durable reuse.</li>\n",
            "<li>Inspect final transcript mismatches before trusting cleaned words.</li>\n",
            "<li>Replay voice parameters are planned values, not proof that replay executed with voice.</li>\n",
            "<li>Replay voice preview is a dry-run plan unless a replay engine receipt exists.</li>\n",
            "<li>Replay voice execution status must stay visible until a replay engine receipt exists.</li>\n",
            "<li>Do not export raw transcript or audio by default.</li>\n",
            "</ul>\n</section>\n",
            "<section>\n<h2>Conflict Warnings</h2>\n{}\n</section>\n",
            "</body>\n</html>\n"
        ),
        html_escape(&input.session_dir.display().to_string()),
        html_escape(&input.context_path.display().to_string()),
        html_escape(&artifact_path(input.packet_path)),
        html_escape(&artifact_path(input.voice_path)),
        html_escape(&artifact_path(input.replay_plan_path)),
        html_escape(&artifact_path(input.evidence_report_path)),
        html_escape(&artifact_path(input.packet_inspection_path)),
        html_escape(&artifact_path(input.dogfood_receipt_path)),
        html_escape(input.claim_ceiling),
        html_escape(input.status),
        html_escape(input.redaction_status),
        html_escape(input.voice_status),
        html_escape(input.voice_execution_status),
        html_escape(input.replay_plan_status),
        html_escape(input.replay_plan_speaks_audio),
        html_escape(input.packet_inspection_status),
        html_escape(input.dogfood_receipt_status),
        html_escape(input.final_alignment_status),
        html_escape(input.final_alignment_word_authority),
        input.final_alignment_unresolved_mismatches,
        html_escape(input.transcript_quality_chain),
        html_escape(input.narration_density_status),
        input.transcript_word_count,
        input.transcript_char_count,
        html_escape(input.leak_status),
        input.leak_count,
        html_escape(&input.leak_categories.join(", ")),
        input.raw_local_sensitive_count,
        html_escape(&input.raw_local_sensitive_categories.join(", ")),
        html_escape(input.capture_helper_state),
        html_escape(input.capture_audio_input),
        input.post_commit_completed_segments,
        input.post_commit_messages,
        input.post_commit_error_count,
        html_escape(input.stop_timeout_status),
        input.alignment_count,
        input.conflict_count,
        input
            .diagnostics
            .get("malformedRecordReplayTimestamps")
            .and_then(Value::as_array)
            .map(Vec::len)
            .unwrap_or(0),
        input
            .diagnostics
            .get("recordReplayEventsWithoutTimestamp")
            .and_then(Value::as_u64)
            .unwrap_or(0),
        input
            .diagnostics
            .get("outOfWindowRecordReplayEvents")
            .and_then(Value::as_u64)
            .unwrap_or(0),
        input.voice_bindings,
        input.voice_proof_obligations,
        input.replay_plan_cue_count,
        recovery_action_list(input.recovery),
        warning_list(input.conflicts),
    )
}

fn artifact_path(path: Option<&Path>) -> String {
    path.map(|path| path.display().to_string())
        .unwrap_or_else(|| "not-generated".to_string())
}

pub(super) fn warning_list(conflicts: &[Value]) -> String {
    if conflicts.is_empty() {
        return "<p class=\"ok\">No transcript action claims currently require UI evidence.</p>"
            .to_string();
    }
    let mut out = String::from("<ul>");
    for warning in conflicts {
        let segment = warning
            .get("segmentId")
            .map(Value::to_string)
            .unwrap_or_else(|| "unknown".to_string());
        let reason = warning
            .get("reason")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let severity = warning
            .get("severity")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let transcript_envelope = serde_json::json!({
            "text": warning.get("transcriptText"),
            "textBoundary": warning.get("transcriptTextBoundary"),
        });
        let text =
            crate::timeline::consume_transcript_segment_text(&transcript_envelope).unwrap_or("");
        out.push_str(&format!(
            "<li class=\"warn\">Segment {}: {} / {}. <q>{}</q></li>",
            html_escape(&segment),
            html_escape(reason),
            html_escape(severity),
            html_escape(text)
        ));
    }
    out.push_str("</ul>");
    out
}

fn recovery_action_list(actions: &[&str]) -> String {
    let mut out = String::from("<ul>");
    for action in actions {
        out.push_str(&format!("<li>{}</li>", html_escape(action)));
    }
    out.push_str("</ul>");
    out
}

fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
