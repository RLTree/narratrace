use super::*;
use crate::config::parse_args_from;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn postprocessing_material_check_honors_existing_artifacts_and_fixtures() {
    let root = unique_tmp("nrr-packet-existing-artifacts");
    let session_dir = root.join("session");
    fs::create_dir_all(&session_dir).unwrap();
    fs::write(session_dir.join("retained-audio.wav"), b"RIFFprivate").unwrap();
    fs::write(
        session_dir.join("batch-transcript.json"),
        r#"{"text":"done"}"#,
    )
    .unwrap();
    fs::write(
        session_dir.join("cleaned-transcript.json"),
        r#"{"cleanedText":"done"}"#,
    )
    .unwrap();
    let args = parse_args_from([
        "nrr",
        "packet",
        "--session-dir",
        session_dir.to_str().unwrap(),
        "--i-consent-to-custom-runtime-paths",
    ])
    .unwrap();

    assert!(!openai_postprocessing_would_send_private_material(
        &args,
        &session_dir,
        false,
        false
    ));
    fs::remove_file(session_dir.join("cleaned-transcript.json")).unwrap();
    assert!(openai_postprocessing_would_send_private_material(
        &args,
        &session_dir,
        false,
        false
    ));
    assert!(!openai_postprocessing_would_send_private_material(
        &args,
        &session_dir,
        false,
        true
    ));
}

#[test]
fn retained_audio_manifest_requires_regular_audio_path() {
    let root = unique_tmp("nrr-packet-retained-audio-manifest");
    let session_dir = root.join("session");
    let audio = root.join("custom.wav");
    fs::create_dir_all(&session_dir).unwrap();

    assert!(!retained_audio_private_material_exists(&session_dir));
    fs::write(
        session_dir.join("audio-retention.json"),
        r#"{"audioPath":42}"#,
    )
    .unwrap();
    assert!(!retained_audio_private_material_exists(&session_dir));
    fs::write(&audio, b"RIFFprivate").unwrap();
    fs::write(
        session_dir.join("audio-retention.json"),
        serde_json::json!({ "audioPath": audio }).to_string(),
    )
    .unwrap();
    assert!(retained_audio_private_material_exists(&session_dir));
    assert_eq!(retained_audio_artifact_path(&session_dir), audio);
}

fn unique_tmp(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    PathBuf::from("/private/tmp").join(format!("{prefix}-{nanos}"))
}
