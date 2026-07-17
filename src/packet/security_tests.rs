use super::*;
use crate::config::parse_args_from;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(unix)]
#[test]
fn symlinked_retention_manifest_fails_consent_probe_closed() {
    let session = PathBuf::from("/private/tmp/nrr-consent-symlink-manifest");
    fs::create_dir_all(&session).unwrap();
    let target = session.join("manifest-target.json");
    fs::write(&target, r#"{"audioPath":null}"#).unwrap();
    let manifest = session.join("audio-retention.json");
    let _ = fs::remove_file(&manifest);
    std::os::unix::fs::symlink(target, manifest).unwrap();

    assert!(retained_audio_private_material_exists(&session));
}

#[test]
fn packet_labels_and_neutralizes_untrusted_markdown_fields() {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let session = std::env::temp_dir().join(format!("nrr-packet-render-{nanos}"));
    fs::create_dir_all(&session).unwrap();
    fs::write(
        session.join("manifest.json"),
        serde_json::json!({"goal": "safe\n## INJECTED GOAL"}).to_string(),
    )
    .unwrap();
    let args = parse_args_from([
        "nrr",
        "packet",
        "--session-dir",
        session.to_str().unwrap(),
        "--recording-metadata",
        "missing\n## INJECTED METADATA",
        "--recording-events",
        "missing\n## INJECTED EVENTS",
        "--disable-batch-transcription",
        "--disable-cleanup",
        "--i-consent-to-custom-runtime-paths",
    ])
    .unwrap();

    make_packet(&args).unwrap();
    let packet = fs::read_to_string(session.join("skill-refinement-packet.md")).unwrap();
    assert_eq!(packet.matches("\n## INJECTED").count(), 0);
    assert!(packet.matches("[untrusted data]").count() >= 14);
    assert!(packet.contains("\\#\\# INJECTED GOAL"));
}
