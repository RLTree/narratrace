use super::*;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn rejects_unparsed_or_unbound_event_evidence() {
    let fixture = fixture("nrr-parent-bad-events");
    for events in [
        "not-json",
        "{\"kind\":\"click\",\"timestamp\":\"2026-06-19T01:38:26Z\"}\nnot-json",
        r#"{"kind":"click"}"#,
        r#"{"kind":"click","timestamp":"2026-02-31T25:61:61Z"}"#,
        r#"{"kind":"click","timestamp":"2026-06-18T01:38:26Z"}"#,
    ] {
        fs::write(&fixture.events, events).unwrap();
        assert!(evaluate(&fixture).is_err(), "{events}");
    }
}

#[test]
fn rejects_event_evidence_above_row_limit() {
    let fixture = fixture("nrr-parent-many-events");
    let row = r#"{"kind":"click","timestamp":"2026-06-19T01:38:26Z"}"#;
    fs::write(
        &fixture.events,
        format!("{row}\n").repeat(PARENT_EVENT_MAX_ROWS + 1),
    )
    .unwrap();

    let error = evaluate(&fixture).unwrap_err().to_string();
    assert!(error.contains("20000 row limit"));
}

#[test]
fn rejects_oversized_event_evidence() {
    let fixture = fixture("nrr-parent-large-events");
    fs::File::create(&fixture.events)
        .unwrap()
        .set_len(PARENT_EVENT_MAX_BYTES + 1)
        .unwrap();

    let error = evaluate(&fixture).unwrap_err().to_string();
    assert!(error.contains("33554432 byte limit"));
}

#[test]
fn checks_start_delta_arithmetic() {
    let fixture = fixture("nrr-parent-delta-overflow");
    fs::write(
        fixture.session.join("capture-clock.json"),
        format!(
            r#"{{"audioStartedAtUnixMs":0,"firstAudioChunkAtUnixMs":{}}}"#,
            i64::MIN
        ),
    )
    .unwrap();

    assert!(
        evaluate(&fixture)
            .unwrap_err()
            .to_string()
            .contains("delta overflow")
    );
}

struct Fixture {
    session: PathBuf,
    metadata: PathBuf,
    events: PathBuf,
}

fn fixture(prefix: &str) -> Fixture {
    let root = unique_tmp(prefix);
    let session = root.join("session");
    let rnr = root.join("rnr");
    fs::create_dir_all(&session).unwrap();
    fs::create_dir_all(&rnr).unwrap();
    let metadata = rnr.join("session.json");
    let events = rnr.join("events.jsonl");
    fs::write(
        &metadata,
        r#"{"id":"rnr-1","startedAt":"2026-06-19T01:38:25Z"}"#,
    )
    .unwrap();
    fs::write(
        &events,
        r#"{"kind":"click","timestamp":"2026-06-19T01:38:26Z"}"#,
    )
    .unwrap();
    fs::write(session.join("status.json"), r#"{"state":"stopped"}"#).unwrap();
    fs::write(
        session.join("capture-clock.json"),
        r#"{"audioStartedAtUnixMs":1781833105000,"firstAudioChunkAtUnixMs":1781833106000}"#,
    )
    .unwrap();
    fs::write(
        session.join("post-commit-drain.json"),
        r#"{"completedSegments":1}"#,
    )
    .unwrap();
    Fixture {
        session,
        metadata,
        events,
    }
}

fn evaluate(fixture: &Fixture) -> Result<ParentOperationEvaluation> {
    evaluate_parent_operation(
        &fixture.session,
        fixture.metadata.to_str().unwrap(),
        fixture.events.to_str().unwrap(),
    )
}

fn unique_tmp(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    PathBuf::from("/private/tmp").join(format!("{prefix}-{nanos}"))
}
