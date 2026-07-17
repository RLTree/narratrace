use super::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
static ENV_LOCK: Mutex<()> = Mutex::new(());
#[cfg(unix)]
#[test]
fn status_read_rejects_symlinked_status_file() {
    let root = unique_tmp("nrr-status-symlink");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("target.json"), r#"{"state":"stopped"}"#).unwrap();
    std::os::unix::fs::symlink(root.join("target.json"), root.join("status.json")).unwrap();

    assert!(read_regular_text(&root.join("status.json")).is_err());
}

#[test]
fn mark_stop_requested_preserves_status_context() {
    let root = unique_tmp("nrr-stop-requested");
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("status.json"),
        r#"{"state":"recording","audioInput":{"deviceName":"MacBook Pro Microphone"}}"#,
    )
    .unwrap();
    mark_stop_requested(&root).unwrap();
    let status: serde_json::Value =
        serde_json::from_str(&read_regular_text(&root.join("status.json")).unwrap()).unwrap();

    assert_eq!(status["state"], "stop-requested");
    assert_eq!(status["audioInput"]["deviceName"], "MacBook Pro Microphone");
    assert!(status["stopRequestedAt"].as_str().is_some());
}

#[test]
fn mark_stop_requested_does_not_regress_terminal_state() {
    let root = unique_tmp("nrr-stop-terminal");
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("status.json"),
        r#"{"state":"stopped","audioInput":{"deviceName":"MacBook Pro Microphone"}}"#,
    )
    .unwrap();
    mark_stop_requested(&root).unwrap();
    let status: serde_json::Value =
        serde_json::from_str(&read_regular_text(&root.join("status.json")).unwrap()).unwrap();

    assert_eq!(status["state"], "stopped");
    assert!(status.get("stopRequestedAt").is_none());
}

#[test]
fn narration_quality_targets_are_machine_readable() {
    let targets = narration_quality_targets();

    assert_eq!(
        targets["minimumTranscriptWordsForNonToyReplay"],
        MIN_TRANSCRIPT_WORDS_FOR_NON_TOY_REPLAY
    );
    assert_eq!(
        targets["recommendedTranscriptWordsForNonToyReplay"],
        RECOMMENDED_TRANSCRIPT_WORDS_FOR_NON_TOY_REPLAY
    );
    assert_eq!(
        targets["recommendedTranscriptSegmentsForNonToyReplay"],
        RECOMMENDED_TRANSCRIPT_SEGMENTS_FOR_NON_TOY_REPLAY
    );
    assert!(
        targets["densityGate"]
            .as_str()
            .unwrap()
            .contains("too-sparse-for-non-toy-replay")
    );
    assert_eq!(targets["checklist"].as_array().unwrap().len(), 6);
}

#[test]
fn manifest_and_status_record_coordination_contract() {
    let root = unique_tmp("nrr-session-manifest");
    fs::create_dir_all(&root).unwrap();

    write_manifest(
        &root,
        "feature walkthrough",
        "coordinated-orchestrator",
        true,
        "explicit-plugin-invocation",
        DEFAULT_AUDIO_FILTER,
    )
    .unwrap();
    write_status(&root, "prepared", Some(1234)).unwrap();

    let manifest: serde_json::Value =
        serde_json::from_str(&read_regular_text(&root.join("manifest.json")).unwrap()).unwrap();
    let status: serde_json::Value =
        serde_json::from_str(&read_regular_text(&root.join("status.json")).unwrap()).unwrap();

    assert_eq!(manifest["goal"], "feature walkthrough");
    assert_eq!(manifest["microphoneConsent"], "explicit-plugin-invocation");
    assert_eq!(
        manifest["startCoordination"]["recordReplayAndMicrophoneSameOperation"],
        true
    );
    assert_eq!(
        manifest["postStopQualityPipeline"]["rawAudioCopiedIntoGeneratedPacketsByDefault"],
        false
    );
    assert_eq!(status["state"], "prepared");
    assert_eq!(status["pid"], 1234);
}

#[test]
fn prepare_coordinated_session_requires_openai_key_before_writing() {
    let _guard = ENV_LOCK.lock().unwrap();
    let root = unique_tmp("nrr-test/nrr-prepare-missing-key");
    unsafe {
        std::env::remove_var("OPENAI_API_KEY");
    }
    let args = crate::config::parse_args_from([
        "nrr",
        "prepare-coordinated-session",
        "--root",
        root.to_str().unwrap(),
        "--i-consent-to-custom-runtime-paths",
    ])
    .unwrap();

    let error = prepare_coordinated_session(&args).unwrap_err().to_string();

    assert!(error.contains("OPENAI_API_KEY is required"));
    assert!(!root.exists());
}

#[test]
fn prepare_coordinated_session_writes_private_manifest_and_status() {
    let _guard = ENV_LOCK.lock().unwrap();
    let root = unique_tmp("nrr-test/nrr-prepare-success");
    unsafe {
        std::env::set_var("OPENAI_API_KEY", "test-key");
    }
    let args = crate::config::parse_args_from([
        "nrr",
        "prepare-coordinated-session",
        "--root",
        root.to_str().unwrap(),
        "--goal",
        "Feature Demo",
        "--max-seconds",
        "30",
        "--audio-retention-path",
        root.join("retained-audio.wav").to_str().unwrap(),
        "--i-consent-to-custom-runtime-paths",
    ])
    .unwrap();

    prepare_coordinated_session(&args).unwrap();
    unsafe {
        std::env::remove_var("OPENAI_API_KEY");
    }

    let session_dir = fs::read_dir(&root)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| path.is_dir())
        .unwrap();
    let manifest: serde_json::Value =
        serde_json::from_str(&read_regular_text(&session_dir.join("manifest.json")).unwrap())
            .unwrap();
    let status: serde_json::Value =
        serde_json::from_str(&read_regular_text(&session_dir.join("status.json")).unwrap())
            .unwrap();

    assert_eq!(manifest["goal"], "Feature Demo");
    assert_eq!(
        manifest["startCoordination"]["recordReplayAndMicrophoneSameOperation"],
        true
    );
    assert_eq!(status["state"], "prepared");
}

#[tokio::test]
async fn stop_returns_existing_terminal_status_without_timeout() {
    let root = unique_tmp("nrr-test/nrr-stop-terminal-command");
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("status.json"),
        r#"{"state":"stopped","sessionDir":"synthetic"}"#,
    )
    .unwrap();
    let args = custom_session_args("stop", &root);

    stop(&args).await.unwrap();

    assert!(root.join(".stop").is_file());
    let status: serde_json::Value =
        serde_json::from_str(&read_regular_text(&root.join("status.json")).unwrap()).unwrap();
    assert_eq!(status["state"], "stopped");
    assert!(status.get("stopRequestedAt").is_none());
}

#[test]
fn status_command_reads_regular_status_file() {
    let root = unique_tmp("nrr-test/nrr-status-command");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("status.json"), r#"{"state":"recording"}"#).unwrap();
    let args = custom_session_args("status", &root);

    status(&args).unwrap();
}

#[tokio::test]
async fn stop_timeout_writes_receipt_without_waiting() {
    let root = unique_tmp("nrr-test/nrr-stop-timeout");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("status.json"), r#"{"state":"recording"}"#).unwrap();
    let args = custom_session_args("stop", &root);

    let error = stop_with_poll(&args, 0, Duration::from_millis(0))
        .await
        .unwrap_err()
        .to_string();
    let receipt: serde_json::Value =
        serde_json::from_str(&read_regular_text(&root.join("stop-timeout.json")).unwrap()).unwrap();

    assert!(error.contains("stop timed out"));
    assert_eq!(receipt["status"], "timeout");
    assert_eq!(receipt["waitedMs"], 0);
    assert_eq!(receipt["stopFilePresent"], true);
    assert_eq!(receipt["lastStatus"]["state"], "stop-requested");
}

fn unique_tmp(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    PathBuf::from("/private/tmp").join(format!("{prefix}-{nanos}"))
}

fn custom_session_args(command: &str, root: &Path) -> crate::config::Args {
    crate::config::parse_args_from([
        "nrr",
        command,
        "--session-dir",
        root.to_str().unwrap(),
        "--i-consent-to-custom-runtime-paths",
    ])
    .unwrap()
}
