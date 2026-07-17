mod preflight;
mod shell_command;

use crate::config::{
    Args, DEFAULT_AUDIO_FILTER, DEFAULT_BATCH_TRANSCRIPTION_MODEL, DEFAULT_CLEANUP_MODEL,
    DEFAULT_MAX_SECONDS, MODEL, REALTIME_ENDPOINT_INTENT, required_session_dir, slugify,
};
use crate::private_fs::{create_private_dir_all, create_private_file, write_private};
use crate::safe_path::open_regular_file;
use anyhow::{Context, Result, bail};
use serde_json::Value;
use serde_json::json;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::sleep;

#[cfg(unix)]
use std::os::unix::fs::DirBuilderExt;

pub use preflight::preflight;

pub(super) const MIN_TRANSCRIPT_WORDS_FOR_NON_TOY_REPLAY: u64 = 30;
pub(super) const RECOMMENDED_TRANSCRIPT_WORDS_FOR_NON_TOY_REPLAY: u64 = 80;
pub(super) const RECOMMENDED_TRANSCRIPT_SEGMENTS_FOR_NON_TOY_REPLAY: u64 = 3;
pub(super) const NARRATION_CHECKLIST: [&str; 6] = [
    "State the workflow intent before manipulating the UI.",
    "Name variable inputs, defaults, and decision criteria as they appear.",
    "Say what changed on screen and why it matters for replay.",
    "Call out brittle points, confusion, or recovery steps immediately.",
    "State the replay success condition in operator-verifiable terms.",
    "Distinguish reusable workflow guidance from private context.",
];

pub(super) fn narration_quality_targets() -> Value {
    json!({
        "schema": "narrated-record-replay.narration-quality-targets.v1",
        "minimumTranscriptWordsForNonToyReplay": MIN_TRANSCRIPT_WORDS_FOR_NON_TOY_REPLAY,
        "recommendedTranscriptWordsForNonToyReplay": RECOMMENDED_TRANSCRIPT_WORDS_FOR_NON_TOY_REPLAY,
        "recommendedTranscriptSegmentsForNonToyReplay": RECOMMENDED_TRANSCRIPT_SEGMENTS_FOR_NON_TOY_REPLAY,
        "densityGate": "packet-inspection.narrationDensityStatus must not be too-sparse-for-non-toy-replay before confident non-toy replay reuse",
        "checklist": NARRATION_CHECKLIST,
        "claimCeiling": "operator guidance only; packet inspection and review contract remain the authority after capture"
    })
}

pub async fn validate(args: &Args) -> Result<()> {
    let ffmpeg = ffmpeg_available();
    let payload = validate_payload(args, ffmpeg);
    if args.json {
        println!("{}", serde_json::to_string_pretty(&payload)?);
    } else {
        println!("ffmpeg: {}", if ffmpeg { "found" } else { "missing" });
        println!(
            "OpenAI key: {}",
            if payload["hasOpenAIKey"].as_bool().unwrap_or(false) {
                "present"
            } else {
                "missing"
            }
        );
        println!("Model: {MODEL}");
        println!("Realtime endpoint intent: {REALTIME_ENDPOINT_INTENT}");
    }
    if ffmpeg {
        Ok(())
    } else {
        bail!("ffmpeg is required")
    }
}

fn validate_payload(args: &Args, ffmpeg: bool) -> Value {
    json!({
        "ok": ffmpeg,
        "ffmpeg": ffmpeg,
        "model": MODEL,
        "realtimeEndpointIntent": REALTIME_ENDPOINT_INTENT,
        "defaultRealtimeDelay": args.delay,
        "defaultAudioFilter": DEFAULT_AUDIO_FILTER,
        "audioFilter": args.audio_filter,
        "batchTranscription": {
            "defaultEnabled": true,
            "defaultModel": DEFAULT_BATCH_TRANSCRIPTION_MODEL
        },
        "cleanup": {
            "defaultEnabled": true,
            "defaultModel": DEFAULT_CLEANUP_MODEL,
            "dictionaryEntryCap": 100
        },
        "hasOpenAIKey": std::env::var("OPENAI_API_KEY").is_ok(),
        "root": args.root,
    })
}

pub(super) fn ffmpeg_available() -> bool {
    ffmpeg_available_at(crate::realtime::ffmpeg_binary())
}

fn ffmpeg_available_at(binary: std::io::Result<&Path>) -> bool {
    let Ok(binary) = binary else { return false };
    binary.is_absolute()
        && Command::new(binary)
            .arg("-version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok()
}

pub fn prepare_coordinated_session(args: &Args) -> Result<()> {
    let goal = args
        .goal
        .as_deref()
        .unwrap_or("narrated record and replay capture");
    if std::env::var("OPENAI_API_KEY").is_err() {
        bail!("OPENAI_API_KEY is required for realtime narration capture");
    }
    create_private_dir_all(&args.root)?;
    let session_dir = allocate_session_dir(&args.root, goal)?;
    write_manifest(
        &session_dir,
        goal,
        "coordinated-orchestrator",
        true,
        "pending-fresh-approval",
        &args.audio_filter,
    )?;
    write_status(&session_dir, "prepared", None)?;
    let max_seconds = args.max_seconds.unwrap_or(DEFAULT_MAX_SECONDS);
    let helper_exe = std::env::current_exe()?;
    let capture_command =
        shell_command::capture_template(&helper_exe, &session_dir, args, max_seconds);
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "sessionDir": session_dir,
            "model": MODEL,
            "realtimeEndpointIntent": REALTIME_ENDPOINT_INTENT,
            "status": "prepared",
            "postStopQualityPipeline": {
                "batchTranscriptionEnabledByDefault": true,
                "batchTranscriptionModel": args.batch_transcription_model,
                "cleanupEnabledByDefault": true,
                "cleanupModel": args.cleanup_model,
                "audioRetentionMode": args.audio_retention_mode,
                "audioFilter": args.audio_filter,
                "requiresPacketTimeOpenAIConsentFlag": "--i-consent-to-openai-postprocessing",
                "normalPacketPath": "packet runs batch transcription, cleanup, and final alignment only when existing local artifacts, fixtures, or explicit OpenAI postprocessing consent allow it"
            },
            "startCoordination": {
                "mode": "coordinated-orchestrator",
                "recordReplayAndMicrophoneSameOperation": true,
                "manualSequentialStartAllowedForLiveProof": false
            },
            "narrationQualityTargets": narration_quality_targets(),
            "parallelStart": {
                "recordReplayTool": "mcp__event_stream.event_stream_start",
                "captureCommandTemplate": capture_command,
                "requiredConsentFlag": "--i-consent-to-microphone-capture",
                "consentSource": "explicit narrated plugin or skill invocation for this bounded run"
            }
        }))?
    );
    Ok(())
}

pub fn start(args: &Args) -> Result<()> {
    let goal = args
        .goal
        .as_deref()
        .unwrap_or("narrated record and replay capture");
    if !args.microphone_capture_consent {
        bail!("--i-consent-to-microphone-capture is required before opening the microphone");
    }
    match args.record_replay_status.as_deref() {
        Some("idle") => {}
        Some("recording") => {
            bail!("Record & Replay is already recording; stop it before narrated capture")
        }
        Some("unavailable") => bail!("Record & Replay event-stream status is unavailable"),
        _ => bail!("--record-replay-status idle is required before narrated capture"),
    }
    if std::env::var("OPENAI_API_KEY").is_err() {
        bail!("OPENAI_API_KEY is required for realtime narration capture");
    }
    create_private_dir_all(&args.root)?;
    let session_dir = allocate_session_dir(&args.root, goal)?;
    write_manifest(
        &session_dir,
        goal,
        "narration-helper-only",
        false,
        "explicit-cli-flag-observed",
        &args.audio_filter,
    )?;
    let exe = std::env::current_exe()?;
    let mut child_command = Command::new(exe);
    child_command
        .arg("capture")
        .arg("--session-dir")
        .arg(&session_dir)
        .arg("--delay")
        .arg(&args.delay)
        .arg("--input")
        .arg(&args.input)
        .arg("--record-replay-status")
        .arg("idle")
        .arg("--audio-retention-mode")
        .arg(&args.audio_retention_mode)
        .arg("--audio-filter")
        .arg(&args.audio_filter)
        .arg("--i-consent-to-microphone-capture");
    if let Some(path) = &args.audio_retention_path {
        child_command.arg("--audio-retention-path").arg(path);
    }
    if let Some(max_seconds) = args.max_seconds {
        child_command
            .arg("--max-seconds")
            .arg(max_seconds.to_string());
    }
    let child_stdout = create_private_file(session_dir.join("capture.stdout.log"))?;
    let child_stderr = create_private_file(session_dir.join("capture.stderr.log"))?;
    let child = child_command
        .stdin(Stdio::null())
        .stdout(Stdio::from(child_stdout))
        .stderr(Stdio::from(child_stderr))
        .spawn()?;
    write_status(&session_dir, "starting", Some(child.id()))?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "sessionDir": session_dir,
            "pid": child.id(),
            "model": MODEL,
            "realtimeEndpointIntent": REALTIME_ENDPOINT_INTENT
        }))?
    );
    Ok(())
}
