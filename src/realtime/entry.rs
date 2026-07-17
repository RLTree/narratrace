use crate::audio_input::{ensure_not_iphone_input, resolve_avfoundation_input};
use crate::audio_retention::AudioRetentionWriter;
use crate::config::{Args, MODEL, REALTIME_ENDPOINT_INTENT, SAMPLE_RATE, required_session_dir};
use crate::packet::update_thought_process;
use crate::private_fs::{create_private_dir_all, write_private};
use crate::timeline;
use anyhow::Result;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use std::process::Stdio;
use tokio::io::AsyncReadExt;
use tokio::process::Command;
use tokio::time::{Duration, Instant, sleep, timeout};
use tokio_tungstenite::connect_async_with_config;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;

mod helpers;
use helpers::{EventKind, handle_event, session_update, should_commit_audio_buffer, write_status};

const PERIODIC_COMMIT_INTERVAL: Duration = Duration::from_secs(5);
const MINIMUM_COMMIT_BYTES: u64 = (SAMPLE_RATE as u64 * 2) / 10;

pub async fn capture(args: &Args) -> Result<()> {
    let session_dir = required_session_dir(args)?;
    if !args.microphone_capture_consent {
        anyhow::bail!(
            "--i-consent-to-microphone-capture is required before opening the microphone"
        );
    }
    if args.record_replay_status.as_deref() != Some("idle") {
        anyhow::bail!("--record-replay-status idle is required before microphone capture");
    }
    validate_capture_duration(args)?;
    create_private_dir_all(&session_dir)?;
    match capture_inner(args, &session_dir).await {
        Ok(()) => Ok(()),
        Err(error) => {
            let message = error.to_string();
            if !session_dir.join("status.json").exists() {
                let _ = write_status(
                    &session_dir,
                    "failed",
                    &args.delay,
                    args.max_seconds,
                    None,
                    Some(&message),
                );
            }
            Err(error)
        }
    }
}

#[cfg(test)]
mod entry_tests {
    use super::capture;
    use crate::config::parse_args_from;

    #[tokio::test]
    async fn capture_rejects_missing_microphone_consent_before_external_resources() {
        let args = parse_args_from([
            "nrr",
            "capture",
            "--session-dir",
            "/private/tmp/narrated-record-replay/nrr-capture-no-consent",
            "--record-replay-status",
            "idle",
        ])
        .unwrap();

        let error = capture(&args).await.unwrap_err().to_string();

        assert!(error.contains("--i-consent-to-microphone-capture is required"));
    }

    #[tokio::test]
    async fn capture_rejects_non_idle_record_replay_before_external_resources() {
        let args = parse_args_from([
            "nrr",
            "capture",
            "--session-dir",
            "/private/tmp/narrated-record-replay/nrr-capture-non-idle",
            "--record-replay-status",
            "recording",
            "--i-consent-to-microphone-capture",
        ])
        .unwrap();

        let error = capture(&args).await.unwrap_err().to_string();

        assert!(error.contains("--record-replay-status idle is required"));
    }

    #[tokio::test]
    async fn capture_writes_failed_status_when_inner_capture_errors() {
        let session_dir = format!(
            "/private/tmp/narrated-record-replay/nrr-capture-inner-error-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let args = parse_args_from([
            "nrr",
            "capture",
            "--session-dir",
            &session_dir,
            "--record-replay-status",
            "idle",
            "--input",
            ":999999",
            "--i-consent-to-microphone-capture",
        ])
        .unwrap();

        let error = capture(&args).await.unwrap_err().to_string();
        let status: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(format!("{session_dir}/status.json")).unwrap(),
        )
        .unwrap();

        assert!(error.contains("AVFoundation audio input index"));
        assert_eq!(status["state"], "failed");
        assert!(status["error"].as_str().unwrap().contains("AVFoundation"));
    }
}
