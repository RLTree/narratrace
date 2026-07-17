use super::*;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn generated_notes_neutralize_redacted_event_fields() {
    let root = unique_tmp("nrr-timeline-safe-notes");
    let session = root.join("session");
    fs::create_dir_all(&session).unwrap();
    fs::write(
        session.join("capture-clock.json"),
        r#"{"audioStartedAtUnixMs":1781833105000}"#,
    )
    .unwrap();
    fs::write(
        session.join("transcript.timeline.jsonl"),
        r#"{"kind":"completed","audioOffsetMs":1000,"text":"normal narration"}"#,
    )
    .unwrap();
    let events = root.join("events.jsonl");
    fs::write(
        &events,
        serde_json::json!({
            "kind":"password: kind-secret\n## INJECTED KIND",
            "timestamp":"2026-06-19T01:38:26Z",
            "app":{"name":"token: app-secret\n> INJECTED APP"},
            "window":{"title":"/Users/tree/private.txt\n- INJECTED WINDOW"},
            "selection":{"selectedText":"password: ui-secret\n* INJECTED UI"}
        })
        .to_string(),
    )
    .unwrap();

    build_temporal_context(&session, None, Some(events.to_str().unwrap())).unwrap();
    let notes = fs::read_to_string(session.join("timestamped-notes.md")).unwrap();
    let context = fs::read_to_string(session.join("temporal-context.json")).unwrap();

    for secret in ["kind-secret", "app-secret", "private.txt", "ui-secret"] {
        assert!(!notes.contains(secret));
        assert!(!context.contains(secret));
    }
    for active in [
        "\n## INJECTED",
        "\n> INJECTED",
        "\n- INJECTED",
        "\n* INJECTED",
    ] {
        assert!(!notes.contains(active));
        assert!(!context.contains(active));
    }
    assert!(notes.contains("Nearby UI:"));
    for label in ["Event kind", "Event app", "Event window"] {
        assert!(notes.contains(&format!("{label}: [untrusted data]")));
    }
}

#[test]
fn generated_notes_preserve_benign_event_fields_as_labeled_data() {
    let root = unique_tmp("nrr-timeline-benign-notes");
    let session = root.join("session");
    fs::create_dir_all(&session).unwrap();
    fs::write(
        session.join("capture-clock.json"),
        r#"{"audioStartedAtUnixMs":1781833105000}"#,
    )
    .unwrap();
    fs::write(
        session.join("transcript.timeline.jsonl"),
        r#"{"kind":"completed","audioOffsetMs":1000,"text":"normal narration"}"#,
    )
    .unwrap();
    let events = root.join("events.jsonl");
    fs::write(
        &events,
        r#"{"kind":"click","timestamp":"2026-06-19T01:38:26Z","app":{"name":"Codex"},"window":{"title":"Review"}}"#,
    )
    .unwrap();

    build_temporal_context(&session, None, Some(events.to_str().unwrap())).unwrap();
    let notes = fs::read_to_string(session.join("timestamped-notes.md")).unwrap();

    assert!(notes.contains("Event kind: [untrusted data] click"));
    assert!(notes.contains("Event app: [untrusted data] Codex"));
    assert!(notes.contains("Event window: [untrusted data] Review"));
}

#[test]
fn generated_notes_label_and_neutralize_untrusted_transcript_markdown() {
    let root = unique_tmp("nrr-timeline-safe-transcript-notes");
    let session = root.join("session");
    fs::create_dir_all(&session).unwrap();
    fs::write(
        session.join("transcript.timeline.jsonl"),
        format!(
            "{}\n",
            serde_json::json!({
                "kind": "completed",
                "audioOffsetMs": 1000,
                "text": "normal narration\n## INJECTED password=short"
            })
        ),
    )
    .unwrap();

    build_temporal_context(&session, None, None).unwrap();
    let notes = fs::read_to_string(session.join("timestamped-notes.md")).unwrap();

    assert!(notes.contains("Transcript segment: [untrusted data]"));
    assert!(notes.contains("\\#\\# INJECTED"));
    assert!(!notes.contains("\n## INJECTED"));
    assert!(!notes.contains("short"));
}

fn unique_tmp(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    PathBuf::from("/private/tmp").join(format!("{prefix}-{nanos}"))
}
