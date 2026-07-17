use super::*;
use serde_json::json;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn comparison_is_inconclusive_when_required_metrics_are_missing() {
    let baseline = json!({"alignmentMetrics": {"unresolvedMismatches": 0}});
    let candidate = json!({"alignmentMetrics": {"unresolvedMismatches": 0}});

    let summary = compare_delay_evaluations(&baseline, &candidate);

    assert_eq!(summary.status, "inconclusive");
    assert_eq!(summary.recommendation, "keep-current-default");
    assert_eq!(summary.reason, "required comparison metrics are missing");
}

#[test]
fn first_event_latency_prefers_monotonic_then_audio_offset() {
    let events = vec![
        json!({"kind": "delta", "audioOffsetMs": 12}),
        json!({"kind": "completed", "monotonicOffsetMs": 34, "audioOffsetMs": 99}),
    ];

    let delta = first_event_latency(&events, "delta");
    let completed = first_event_latency(&events, "completed");
    let missing = first_event_latency(&events, "error");

    assert_eq!(delta.latency_ms, Some(12));
    assert_eq!(delta.source, "audio-wall-clock-offset");
    assert_eq!(completed.latency_ms, Some(34));
    assert_eq!(completed.source, "process-local-monotonic-offset");
    assert_eq!(missing.latency_ms, None);
    assert_eq!(missing.source, "missing");
}

#[test]
fn read_helpers_are_conservative_for_missing_and_malformed_inputs() {
    let root = unique_tmp("nrr-delay-read-extra");
    fs::create_dir_all(&root).unwrap();
    let bad_jsonl = root.join("bad.jsonl");
    fs::write(&bad_jsonl, "{bad}\n{\"ok\":true}\n").unwrap();
    let missing = root.join("missing.json");

    assert!(read_json(&missing).is_err());
    assert_eq!(read_json_lines(&missing).unwrap().len(), 0);
    assert!(read_json_lines(&bad_jsonl).is_err());
    assert!(line_count(&missing).is_err());
}

#[test]
fn delay_readers_reject_byte_and_row_budget_exhaustion() {
    let root = unique_tmp("nrr-delay-read-bounds");
    fs::create_dir_all(&root).unwrap();
    let oversized = root.join("oversized.json");
    fs::File::create(&oversized)
        .unwrap()
        .set_len(MAX_DELAY_JSON_BYTES + 1)
        .unwrap();
    assert!(read_json(&oversized).is_err());

    let rows = root.join("rows.jsonl");
    fs::write(&rows, "{}\n".repeat(MAX_DELAY_JSONL_ROWS + 1)).unwrap();
    assert!(
        read_json_lines(&rows)
            .unwrap_err()
            .to_string()
            .contains("row limit")
    );
}

#[test]
fn utc_timestamp_parser_rejects_malformed_values() {
    for value in [
        "2026-06-22 22:34:02Z",
        "2026-06-22T22:34Z",
        "2026-06-22T22:34:02:99Z",
        "2026-06-22T22:xx:02Z",
    ] {
        assert!(parse_utc_timestamp_ms(value).is_err(), "{value}");
    }
}

#[test]
fn utc_timestamp_parser_enforces_calendar_clock_and_fraction_ranges() {
    for value in [
        "2026-02-29T00:00:00Z",
        "2026-04-31T00:00:00Z",
        "2026-01-01T24:00:00Z",
        "2026-01-01T00:60:00Z",
        "2026-01-01T00:00:60Z",
        "2026-01-01T00:00:00.1234567890Z",
    ] {
        assert!(parse_utc_timestamp_ms(value).is_err(), "{value}");
    }
    assert_eq!(
        parse_utc_timestamp_ms("2024-02-29T23:59:59.125Z").unwrap(),
        1_709_251_199_125
    );
}

#[test]
fn diagnostic_marker_recall_counts_marker_words_without_raw_output() {
    let alignment = json!({
        "segments": [
            {"text": "Alpha marker and bravo marker"},
            {"text": "Quebec marker closes the run"}
        ]
    });

    let recall = scripted_marker_recall(&alignment);

    assert_eq!(recall["found"], 3);
    assert_eq!(recall["diagnosticOnly"], true);
}

fn unique_tmp(prefix: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::path::PathBuf::from(format!("/private/tmp/{prefix}-{nanos}"))
}
