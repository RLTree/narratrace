use super::{RuntimeConfig, capture_inner_with_runtime};
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

#[cfg(unix)]
#[tokio::test(flavor = "current_thread")]
async fn final_commit_without_post_send_completion_fails_closed() {
    let _guard = ENV_LOCK.lock().unwrap();
    let root = unique_tmp("nrr-test/nrr-final-commit-unacknowledged");
    fs::create_dir_all(&root).unwrap();
    let fake_ffmpeg = root.join("fake-ffmpeg");
    fs::write(
        &fake_ffmpeg,
        "#!/bin/sh\ndd if=/dev/zero bs=8192 count=1 2>/dev/null\nsleep 6\ndd if=/dev/zero bs=8192 count=1 2>/dev/null\nsleep 1\ndd if=/dev/zero bs=8192 count=1 2>/dev/null\nsleep 2\n",
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
        let mut commits_received = 0_u64;
        while let Some(message) = socket.next().await {
            let Message::Text(text) = message.unwrap() else {
                continue;
            };
            if !text.contains("\"input_audio_buffer.commit\"") {
                continue;
            }
            commits_received += 1;
            if commits_received == 1 {
                socket
                    .send(Message::Text(
                        serde_json::json!({
                            "type": "conversation.item.input_audio_transcription.completed",
                            "transcript": "periodic commit completed"
                        })
                        .to_string()
                        .into(),
                    ))
                    .await
                    .unwrap();
            } else {
                socket.close(None).await.unwrap();
                break;
            }
        }
        commits_received
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
        "12",
        "--i-consent-to-microphone-capture",
        "--i-consent-to-custom-runtime-paths",
    ])
    .unwrap();

    let runtime = RuntimeConfig::for_test(&url, &fake_ffmpeg);
    let error = capture_inner_with_runtime(&args, &root, runtime)
        .await
        .unwrap_err()
        .to_string();
    let commits_received = server.await.unwrap();
    unsafe {
        std::env::remove_var("OPENAI_API_KEY");
    }

    let drain: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(root.join("post-commit-drain.json")).unwrap())
            .unwrap();
    let status: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(root.join("status.json")).unwrap()).unwrap();

    assert!(error.contains("final audio commit was not acknowledged"));
    assert_eq!(commits_received, 2);
    assert_eq!(
        drain["captureStats"]["realtimeCompletedSegmentsObserved"],
        1
    );
    assert_eq!(drain["captureStats"]["audioCommitsSent"], 1);
    assert_eq!(drain["captureStats"]["audioBytesPendingFinalCommit"], 8192);
    assert_eq!(drain["errors"].as_array().unwrap().len(), 1);
    assert_eq!(drain["finalCommit"]["status"], "unacknowledged");
    assert_eq!(status["state"], "failed");
}

fn unique_tmp(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    PathBuf::from("/private/tmp").join(format!("{prefix}-{nanos}"))
}
