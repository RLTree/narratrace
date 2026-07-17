use super::{RuntimeConfig, capture_inner, capture_inner_with_runtime, post_commit_drain_receipt};
use crate::config::parse_args_from;
use futures_util::{SinkExt, StreamExt};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn post_commit_drain_receipt_is_metadata_only() {
    let receipt = post_commit_drain_receipt(
        42,
        2,
        1,
        vec!["realtime-error-event".to_string()],
        3,
        8192,
        2,
        0,
        5,
        4,
        2,
        1,
        "highpass=f=80,lowpass=f=9000,volume=9dB,alimiter=limit=0.95",
        "acknowledged",
    );

    assert_eq!(
        receipt["schema"],
        "narrated-record-replay.post-commit-drain.v1"
    );
    assert_eq!(receipt["captureStats"]["realtimeMessagesObserved"], 7);
    assert_eq!(receipt["captureStats"]["audioBytesSent"], 8192);
    assert_eq!(receipt["finalCommit"]["status"], "acknowledged");
    assert_eq!(receipt["finalCommit"]["postSendMessagesObserved"], 2);
    assert_eq!(
        receipt["finalCommit"]["postSendCompletedSegmentsObserved"],
        1
    );
    assert!(
        receipt["claimCeiling"]
            .as_str()
            .unwrap()
            .contains("metadata-only")
    );
    assert!(receipt.get("rawTranscript").is_none());
    assert!(receipt.get("rawAudio").is_none());
}

#[cfg(unix)]
#[tokio::test(flavor = "current_thread")]
async fn capture_inner_runs_against_local_realtime_and_fake_audio() {
    let _guard = ENV_LOCK.lock().unwrap();
    let root = unique_tmp("nrr-test/nrr-local-capture-loop");
    fs::create_dir_all(&root).unwrap();
    let fake_ffmpeg = root.join("fake-ffmpeg");
    fs::write(
        &fake_ffmpeg,
        "#!/bin/sh\ndd if=/dev/zero bs=8192 count=1 2>/dev/null\n",
    )
    .unwrap();
    let mut permissions = fs::metadata(&fake_ffmpeg).unwrap().permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
    fs::set_permissions(&fake_ffmpeg, permissions).unwrap();

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("ws://{}", listener.local_addr().unwrap());
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut socket = accept_async(stream).await.unwrap();
        while let Some(message) = socket.next().await {
            let Message::Text(text) = message.unwrap() else {
                continue;
            };
            if text.contains("\"input_audio_buffer.commit\"") {
                socket
                    .send(Message::Text(
                        serde_json::json!({
                            "type": "conversation.item.input_audio_transcription.completed",
                            "transcript": "local capture loop completed"
                        })
                        .to_string()
                        .into(),
                    ))
                    .await
                    .unwrap();
                break;
            }
        }
    });

    unsafe {
        std::env::set_var("OPENAI_API_KEY", "test-key");
    }
    let args = parse_args_from([
        "nrr",
        "capture",
        "--session-dir",
        root.to_str().unwrap(),
        "--record-replay-status",
        "idle",
        "--input",
        ":MacBook Pro Microphone",
        "--max-seconds",
        "1",
        "--i-consent-to-microphone-capture",
        "--i-consent-to-custom-runtime-paths",
    ])
    .unwrap();

    let runtime = RuntimeConfig::for_test(&url, &fake_ffmpeg);
    capture_inner_with_runtime(&args, &root, runtime)
        .await
        .unwrap();
    server.await.unwrap();
    unsafe {
        std::env::remove_var("OPENAI_API_KEY");
    }

    let status: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(root.join("status.json")).unwrap()).unwrap();
    let drain: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(root.join("post-commit-drain.json")).unwrap())
            .unwrap();
    assert_eq!(status["state"], "stopped");
    assert_eq!(drain["completedSegments"], 1);
    assert_eq!(drain["captureStats"]["audioCommitsSent"], 1);
    assert_eq!(drain["captureStats"]["audioBytesPendingFinalCommit"], 0);
    assert_eq!(drain["errors"], serde_json::json!([]));
    assert_eq!(drain["finalCommit"]["status"], "acknowledged");
    assert!(root.join("narration.sync.jsonl").is_file());
    assert!(root.join("retained-audio.wav").is_file());
    assert!(root.join("transcript.timeline.jsonl").is_file());
}

#[tokio::test(flavor = "current_thread")]
async fn capture_inner_rejects_iphone_input_before_openai_or_ffmpeg() {
    let root = unique_tmp("nrr-test/nrr-capture-iphone-reject");
    fs::create_dir_all(&root).unwrap();
    let args = parse_args_from([
        "nrr",
        "capture",
        "--session-dir",
        root.to_str().unwrap(),
        "--record-replay-status",
        "idle",
        "--input",
        ":Tree's iPhone Microphone",
        "--i-consent-to-microphone-capture",
        "--i-consent-to-custom-runtime-paths",
    ])
    .unwrap();

    let error = capture_inner(&args, &root).await.unwrap_err().to_string();
    let status: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(root.join("status.json")).unwrap()).unwrap();

    assert!(error.contains("refusing to use iPhone microphone"));
    assert_eq!(status["state"], "failed");
    assert_eq!(
        status["audioInput"]["deviceName"],
        "Tree's iPhone Microphone"
    );
}

fn unique_tmp(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    PathBuf::from("/private/tmp").join(format!("{prefix}-{nanos}"))
}
