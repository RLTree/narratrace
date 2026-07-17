use super::record_replay::read_rnr_events;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn read_rnr_events_parses_timestamp_states_and_redacts_ui_text() {
    let root = unique_tmp("nrr-timeline-rnr-events");
    fs::create_dir_all(&root).unwrap();
    let events_path = root.join("events.jsonl");
    let long = "x".repeat(260);
    fs::write(
        &events_path,
        format!(
            "{}\n{}\n{}\n",
            serde_json::json!({
                "id": 1,
                "kind": "click",
                "timestamp": "2026-06-19T01:38:25Z",
                "app": {"name": "Finder"},
                "window": {"title": "/Users/tree/private.txt"},
                "selection": {"selectedText": "selected /Users/tree/private.txt"}
            }),
            serde_json::json!({
                "id": 2,
                "timestamp": "bad-date",
                "selection": {"target": {"value": long}}
            }),
            serde_json::json!({
                "id": 3,
                "ax": {"mode": "AXButton"}
            })
        ),
    )
    .unwrap();

    let events = read_rnr_events(&events_path).unwrap();

    assert_eq!(events.len(), 3);
    assert_eq!(events[0].kind, "click");
    assert_eq!(events[0].timestamp_parse_status, "valid");
    assert_eq!(events[0].app.as_deref(), Some("Finder"));
    assert!(events[0].window.as_deref().unwrap().contains("REDACTED"));
    assert!(events[0].ui_hint.as_deref().unwrap().contains("REDACTED"));
    assert_eq!(events[1].kind, "unknown");
    assert_eq!(events[1].timestamp_parse_status, "invalid");
    assert!(events[1].ui_hint.as_deref().unwrap().ends_with("..."));
    assert_eq!(events[2].timestamp_parse_status, "missing");
    assert_eq!(events[2].ui_hint.as_deref(), Some("AXButton"));
}

#[test]
fn read_rnr_events_fails_closed_on_any_malformed_jsonl_row() {
    let root = unique_tmp("nrr-timeline-rnr-malformed");
    fs::create_dir_all(&root).unwrap();
    let events_path = root.join("events.jsonl");
    fs::write(
        &events_path,
        "{\"kind\":\"click\",\"timestamp\":\"2026-06-19T01:38:25Z\"}\nnot-json\n",
    )
    .unwrap();

    let error = read_rnr_events(&events_path).unwrap_err().to_string();
    assert!(error.contains("line 2 is malformed JSON"));
}

#[test]
fn read_rnr_events_marks_impossible_clock_values_invalid() {
    let root = unique_tmp("nrr-timeline-rnr-invalid-clock");
    fs::create_dir_all(&root).unwrap();
    let events_path = root.join("events.jsonl");
    fs::write(
        &events_path,
        r#"{"kind":"click","timestamp":"2026-02-31T25:61:61.999Z"}"#,
    )
    .unwrap();

    let event = read_rnr_events(&events_path).unwrap().remove(0);
    assert_eq!(event.timestamp_parse_status, "invalid");
    assert_eq!(event.unix_ms, None);
}

#[cfg(unix)]
#[test]
fn read_rnr_events_rejects_symlinked_event_stream() {
    let root = unique_tmp("nrr-timeline-rnr-symlink");
    fs::create_dir_all(&root).unwrap();
    let target = root.join("target.jsonl");
    fs::write(&target, r#"{"id":1,"kind":"click"}"#).unwrap();
    let link = root.join("events.jsonl");
    std::os::unix::fs::symlink(&target, &link).unwrap();

    assert!(read_rnr_events(&link).unwrap().is_empty());
}

#[test]
fn read_rnr_events_redacts_and_neutralizes_agent_readable_fields() {
    let root = unique_tmp("nrr-timeline-rnr-safe-fields");
    fs::create_dir_all(&root).unwrap();
    let events_path = root.join("events.jsonl");
    fs::write(
        &events_path,
        serde_json::json!({
            "kind": "password: hunter2\n## KIND",
            "timestamp": "2026-06-19T01:38:25Z",
            "app": {"name": "/Users/tree/private\n> APP"},
            "window": {"title": "password: abc123\n- WINDOW"},
            "selection": {"selectedText": "token: xyz789\n* UI"}
        })
        .to_string(),
    )
    .unwrap();

    let event = read_rnr_events(&events_path).unwrap().remove(0);
    let rendered = serde_json::to_string(&event.to_json()).unwrap();

    assert!(!rendered.contains("hunter2"));
    assert!(!rendered.contains("private"));
    assert!(!rendered.contains("abc123"));
    assert!(!rendered.contains("xyz789"));
    for value in [
        Some(event.kind.as_str()),
        event.app.as_deref(),
        event.window.as_deref(),
        event.ui_hint.as_deref(),
    ] {
        let value = value.unwrap();
        assert!(!value.contains('\n'));
        assert!(!value.contains("##"));
    }
    assert!(event.kind.contains("\\#\\# KIND"));
    assert!(event.app.as_deref().unwrap().contains("\\> APP"));
    assert!(event.window.as_deref().unwrap().contains("\\- WINDOW"));
    assert!(event.ui_hint.as_deref().unwrap().contains("\\* UI"));
}

#[test]
fn read_rnr_events_rejects_oversized_bytes_and_rows() {
    let root = unique_tmp("nrr-timeline-rnr-bounds");
    fs::create_dir_all(&root).unwrap();
    let events_path = root.join("events.jsonl");
    fs::File::create(&events_path)
        .unwrap()
        .set_len(32 * 1024 * 1024 + 1)
        .unwrap();
    assert!(
        read_rnr_events(&events_path)
            .unwrap_err()
            .to_string()
            .contains("recording event artifact exceeds 33554432 byte limit")
    );

    let rows = "{\"kind\":\"click\"}\n".repeat(20_001);
    fs::write(&events_path, rows).unwrap();
    assert!(
        read_rnr_events(&events_path)
            .unwrap_err()
            .to_string()
            .contains("recording event artifact exceeds 20000 row limit")
    );
}

#[test]
fn read_rnr_events_rejects_oversized_event_text() {
    let root = unique_tmp("nrr-timeline-rnr-text-bound");
    fs::create_dir_all(&root).unwrap();
    let events_path = root.join("events.jsonl");
    fs::write(
        &events_path,
        serde_json::json!({"kind":"click","app":{"name":"x".repeat(4097)}}).to_string(),
    )
    .unwrap();

    assert!(
        read_rnr_events(&events_path)
            .unwrap_err()
            .to_string()
            .contains("event app exceeds 4096 byte limit")
    );
}

fn unique_tmp(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    PathBuf::from("/private/tmp").join(format!("{prefix}-{nanos}"))
}
