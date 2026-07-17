use super::*;
use crate::config::parse_args_from;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn packet_and_thought_process_render_all_untrusted_markdown_fields() {
    let root = unique_tmp("nrr-packet-all-fields");
    let session = root.join("session\n## INJECTED PATH");
    fs::create_dir_all(&session).unwrap();
    fs::write(
        session.join("status.json"),
        serde_json::json!({"model": "safe-model\n## INJECTED MODEL"}).to_string(),
    )
    .unwrap();
    fs::write(
        session.join("manifest.json"),
        serde_json::json!({"goal": "ordinary goal"}).to_string(),
    )
    .unwrap();
    let args = parse_args_from([
        "nrr",
        "packet",
        "--session-dir",
        session.to_str().unwrap(),
        "--disable-batch-transcription",
        "--disable-cleanup",
        "--i-consent-to-custom-runtime-paths",
    ])
    .unwrap();

    make_packet(&args).unwrap();
    let packet = fs::read_to_string(session.join("skill-refinement-packet.md")).unwrap();
    let thought = fs::read_to_string(session.join("thought-process.md")).unwrap();

    assert!(!packet.contains("\n## INJECTED"));
    assert!(!thought.contains("\n## INJECTED"));
    for label in [
        "Narration session: [untrusted data]",
        "Narration model: [untrusted data]",
        "Transcript events: [untrusted data]",
        "Timestamped transcript: [untrusted data]",
        "Temporal context packet: [untrusted data]",
        "Thought process: [untrusted data]",
        "Replay voice parameters: [untrusted data]",
        "Evidence boundary report: [untrusted data]",
        "Batch transcript: [untrusted data]",
        "Cleaned transcript: [untrusted data]",
        "Final aligned transcript: [untrusted data]",
    ] {
        assert!(packet.contains(label), "missing safe field {label}");
    }
    for label in [
        "Transcript timeline: [untrusted data]",
        "Transcript events: [untrusted data]",
        "Final transcript: [untrusted data]",
        "Live transcript: [untrusted data]",
    ] {
        assert!(thought.contains(label), "missing safe field {label}");
    }
    assert!(packet.contains("ordinary goal"));
    assert!(packet.contains("safe\\-model \\#\\# INJECTED MODEL"));

    fs::remove_dir_all(root).unwrap();
}

fn unique_tmp(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    PathBuf::from("/private/tmp").join(format!("{prefix}-{nanos}"))
}
