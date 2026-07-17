use crate::safe_path::{read_regular_text_bounded, regular_file_metadata};
use crate::timeline::consume_transcript_segment_text;
use serde_json::{Value, json};
use std::path::Path;

const MAX_USEFULNESS_TEXT_BYTES: u64 = 8 * 1024 * 1024;
const TEMPORAL_CONTEXT_SCHEMA: &str = "narrated-record-replay.temporal-context.v1";

pub(super) fn packet_usefulness_review(
    packet_path: &Path,
    notes_path: &Path,
    thought_path: &Path,
    temporal_path: &Path,
    evidence_path: &Path,
    temporal: &Value,
    evidence: &Value,
) -> Value {
    let packet = read_regular_text(packet_path);
    let packet_exists = regular_file_exists(packet_path);
    let notes_exists = regular_file_exists(notes_path);
    let thought_exists = regular_file_exists(thought_path);
    let temporal_exists = regular_file_exists(temporal_path);
    let evidence_exists = regular_file_exists(evidence_path);
    let transcript_segments = evidence
        .pointer("/evidenceSurfaces/transcriptSegments")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let aligned_segments = evidence
        .pointer("/evidenceSurfaces/alignedSegments")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let conflict_warnings = temporal
        .pointer("/conflictDiagnostics/warnings")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0);
    let quality = narration_quality(temporal);
    json!({
        "status": "fixture-review-only",
        "claimCeiling": "section and artifact presence checks only; real non-toy packet usefulness review is still owed",
        "signals": {
            "packetExists": packet_exists,
            "timestampedNotesExists": notes_exists,
            "thoughtProcessExists": thought_exists,
            "temporalContextExists": temporal_exists,
            "evidenceBoundaryReportExists": evidence_exists,
            "hasGoal": packet.contains("Goal: "),
            "hasCaptureArtifactsSection": packet.contains("## Capture Artifacts"),
            "hasEvidenceBoundarySection": packet.contains("## Evidence Boundary"),
            "hasRefinementInstructions": packet.contains("## Refinement Instructions"),
            "hasTemporalAlignmentSummary": packet.contains("## Temporal Alignment Summary"),
            "hasTranscriptReviewBoundarySection": packet.contains("## Transcript Review Boundary"),
            "rawTranscriptEmbeddingAvoided": !packet.contains("## Timestamped Transcript"),
            "transcriptSegments": transcript_segments,
            "transcriptWordCount": quality.word_count,
            "transcriptCharCount": quality.char_count,
            "recordReplayEventCount": quality.event_count,
            "narrationDensityStatus": quality.status,
            "alignedSegments": aligned_segments,
            "conflictWarnings": conflict_warnings
        },
        "checkedSurfaces": [
            "packet includes goal, artifact pointers, evidence boundary, refinement instructions, temporal summary, and transcript review boundary sections",
            "timestamped notes, thought-process boundary, temporal context, and evidence boundary report artifacts exist",
            "evidence counts remain machine-readable for later real-packet review",
            "conflict warnings remain blockers for operator review",
            "raw transcript text remains in local-private transcript artifacts instead of generated review packets"
        ],
        "blockers": blockers(
            packet_exists,
            notes_exists,
            thought_exists,
            temporal_exists,
            evidence_exists,
            transcript_segments,
            &quality,
            aligned_segments,
            conflict_warnings,
        )
    })
}

fn blockers(
    packet_exists: bool,
    notes_exists: bool,
    thought_exists: bool,
    temporal_exists: bool,
    evidence_exists: bool,
    transcript_segments: u64,
    quality: &NarrationQuality,
    aligned_segments: u64,
    conflict_warnings: usize,
) -> Vec<&'static str> {
    let mut blockers = Vec::new();
    if !packet_exists {
        blockers.push("skill-refinement-packet.md is missing");
    }
    if !notes_exists {
        blockers.push("timestamped-notes.md is missing");
    }
    if !thought_exists {
        blockers.push("thought-process.md is missing");
    }
    if !temporal_exists {
        blockers.push("temporal-context.json is missing");
    }
    if !evidence_exists {
        blockers.push("evidence-boundary-report.json is missing");
    }
    if transcript_segments == 0 {
        blockers.push("packet has no transcript segments");
    }
    if quality.status == "too-sparse-for-non-toy-replay" {
        blockers.push("narration is too sparse for confident non-toy replay refinement");
    }
    if aligned_segments == 0 {
        blockers.push("packet has no aligned segments");
    }
    if conflict_warnings > 0 {
        blockers.push("conflict warnings need operator review");
    }
    blockers.push("real non-toy packet usefulness review still owed");
    blockers
}

struct NarrationQuality {
    word_count: usize,
    char_count: usize,
    event_count: usize,
    status: &'static str,
}

fn narration_quality(temporal: &Value) -> NarrationQuality {
    let segments =
        if temporal.get("schema").and_then(Value::as_str) == Some(TEMPORAL_CONTEXT_SCHEMA) {
            temporal
                .get("transcriptSegments")
                .and_then(Value::as_array)
                .map(Vec::as_slice)
                .unwrap_or_default()
        } else {
            &[]
        };
    let event_count = temporal
        .get("recordReplayEvents")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0);
    let transcript_text = segments
        .iter()
        .filter_map(consume_transcript_segment_text)
        .collect::<Vec<_>>();
    let text = transcript_text.join(" ");
    let word_count = text.split_whitespace().count();
    let char_count = text.chars().count();
    let sparse_for_ui = event_count >= 10 && (transcript_text.len() < 2 || word_count < 30);
    let status = if sparse_for_ui || word_count < 10 {
        "too-sparse-for-non-toy-replay"
    } else if word_count < 50 {
        "needs-operator-distillation"
    } else {
        "sufficient-for-operator-review"
    };
    NarrationQuality {
        word_count,
        char_count,
        event_count,
        status,
    }
}

fn regular_file_exists(path: &Path) -> bool {
    regular_file_metadata(path).is_ok()
}

fn read_regular_text(path: &Path) -> String {
    if !regular_file_exists(path) {
        return String::new();
    }
    read_regular_text_bounded(path, MAX_USEFULNESS_TEXT_BYTES).unwrap_or_default()
}
