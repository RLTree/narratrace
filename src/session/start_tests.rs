use super::*;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn validate_payload_records_runtime_defaults_without_opening_resources() {
    let _guard = ENV_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    unsafe {
        std::env::remove_var("OPENAI_API_KEY");
    }
    let args = crate::config::parse_args_from(["nrr", "validate"]).unwrap();

    let payload = validate_payload(&args, false);

    assert_eq!(payload["ok"], false);
    assert_eq!(payload["model"], MODEL);
    assert_eq!(payload["realtimeEndpointIntent"], REALTIME_ENDPOINT_INTENT);
    assert_eq!(payload["defaultRealtimeDelay"], "high");
    assert_eq!(payload["batchTranscription"]["defaultEnabled"], true);
    assert_eq!(payload["cleanup"]["dictionaryEntryCap"], 100);
    assert!(payload["hasOpenAIKey"].is_boolean());
}

#[test]
fn ffmpeg_probe_rejects_relative_executable_names() {
    assert!(!ffmpeg_available_at(Ok(std::path::Path::new("ffmpeg"))));
}

#[test]
fn start_requires_microphone_consent_before_other_checks() {
    let args = crate::config::parse_args_from(["nrr", "start"]).unwrap();

    let error = start(&args).unwrap_err().to_string();

    assert!(error.contains("--i-consent-to-microphone-capture"));
}

#[test]
fn start_rejects_non_idle_record_replay_before_openai_key() {
    let _guard = ENV_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    unsafe {
        std::env::remove_var("OPENAI_API_KEY");
    }
    let args = crate::config::parse_args_from([
        "nrr",
        "start",
        "--record-replay-status",
        "recording",
        "--i-consent-to-microphone-capture",
    ])
    .unwrap();

    let error = start(&args).unwrap_err().to_string();

    assert!(error.contains("already recording"));
}

#[test]
fn start_requires_openai_key_after_idle_status() {
    let _guard = ENV_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    unsafe {
        std::env::remove_var("OPENAI_API_KEY");
    }
    let args = crate::config::parse_args_from([
        "nrr",
        "start",
        "--record-replay-status",
        "idle",
        "--i-consent-to-microphone-capture",
    ])
    .unwrap();

    let error = start(&args).unwrap_err().to_string();

    assert!(error.contains("OPENAI_API_KEY is required"));
}

#[test]
fn start_writes_private_status_and_helper_logs_after_spawn() {
    let _guard = ENV_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let root = unique_tmp("nrr-test/nrr-start-success");
    unsafe {
        std::env::set_var("OPENAI_API_KEY", "test-key");
    }
    let args = crate::config::parse_args_from([
        "nrr",
        "start",
        "--root",
        root.to_str().unwrap(),
        "--record-replay-status",
        "idle",
        "--max-seconds",
        "1",
        "--i-consent-to-custom-runtime-paths",
        "--i-consent-to-microphone-capture",
    ])
    .unwrap();

    start(&args).unwrap();
    unsafe {
        std::env::remove_var("OPENAI_API_KEY");
    }

    let session_dir = std::fs::read_dir(&root)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| path.is_dir())
        .unwrap();
    let status: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(session_dir.join("status.json")).unwrap())
            .unwrap();

    assert_eq!(status["state"], "starting");
    assert!(status["pid"].as_u64().unwrap() > 0);
    assert!(session_dir.join("capture.stdout.log").is_file());
    assert!(session_dir.join("capture.stderr.log").is_file());
}

fn unique_tmp(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    PathBuf::from("/private/tmp").join(format!("{prefix}-{nanos}"))
}
