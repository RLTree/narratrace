use super::{
    CaptureQuota, MAX_REALTIME_MESSAGE_BYTES, MODEL, REALTIME_ENDPOINT_INTENT, SAMPLE_RATE,
};
use crate::audio_input::ResolvedAudioInput;
use crate::packet::update_thought_process;
use crate::private_fs::{append_private, write_private};
use crate::timeline;
use anyhow::Result;
use serde_json::{Value, json};

pub(super) fn session_update(delay: &str) -> Value {
    json!({
        "type": "session.update",
        "session": {
            "type": "transcription",
            "audio": {
                "input": {
                    "format": { "type": "audio/pcm", "rate": SAMPLE_RATE },
                    "transcription": { "model": MODEL, "language": "en", "delay": delay },
                    "turn_detection": null
                }
            }
        }
    })
}

pub(super) fn should_commit_audio_buffer(
    buffered_audio_bytes: u64,
    elapsed_since_commit: std::time::Duration,
    commit_interval: std::time::Duration,
    minimum_commit_bytes: u64,
) -> bool {
    buffered_audio_bytes >= minimum_commit_bytes && elapsed_since_commit >= commit_interval
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum EventKind {
    Completed,
    Delta,
    Error,
    Other,
}

pub(super) fn handle_event(
    session_dir: &std::path::Path,
    text: &str,
    monotonic_offset_ms: u64,
    quota: &mut CaptureQuota,
) -> Result<EventKind> {
    if text.len() > MAX_REALTIME_MESSAGE_BYTES {
        quota.reserve_realtime_event(text)?;
        unreachable!("oversized realtime events are always rejected")
    }
    let event: Value = serde_json::from_str(text)?;
    quota.reserve_realtime_event(text)?;
    if event.get("type").and_then(Value::as_str) == Some("error") {
        write_status(
            session_dir,
            "failed",
            "unknown",
            None,
            None,
            Some("realtime service returned an error event; payload omitted"),
        )?;
        return Ok(EventKind::Error);
    }
    append(
        session_dir.join("transcript.events.jsonl"),
        &format!("{text}\n"),
    )?;
    match event.get("type").and_then(Value::as_str) {
        Some("conversation.item.input_audio_transcription.delta") => {
            if let Some(delta) = event.get("delta").and_then(Value::as_str) {
                timeline::record_transcript_event_with_monotonic_offset(
                    session_dir,
                    &event,
                    "delta",
                    delta,
                    Some(monotonic_offset_ms),
                )?;
                append(session_dir.join("transcript.live.txt"), delta)?;
                return Ok(EventKind::Delta);
            }
            Ok(EventKind::Other)
        }
        Some("conversation.item.input_audio_transcription.completed") => {
            if let Some(transcript) = event.get("transcript").and_then(Value::as_str) {
                if transcript.trim().is_empty() {
                    return Ok(EventKind::Other);
                }
                timeline::record_transcript_event_with_monotonic_offset(
                    session_dir,
                    &event,
                    "completed",
                    transcript,
                    Some(monotonic_offset_ms),
                )?;
                append(
                    session_dir.join("transcript.final.txt"),
                    &format!("{transcript}\n"),
                )?;
                update_thought_process(session_dir)?;
            }
            Ok(EventKind::Completed)
        }
        _ => Ok(EventKind::Other),
    }
}

pub(super) fn append(path: impl AsRef<std::path::Path>, text: &str) -> Result<()> {
    append_private(path, text)
}

pub(super) fn write_status(
    session_dir: &std::path::Path,
    state: &str,
    delay: &str,
    max_seconds: Option<u64>,
    input: Option<&ResolvedAudioInput>,
    error: Option<&str>,
) -> Result<()> {
    let audio_input = input.map(|input| {
        json!({
            "requested": &input.requested,
            "ffmpegInput": &input.ffmpeg_input,
            "deviceName": &input.device_name,
            "source": &input.source,
        })
    });
    let status = json!({
        "state": state,
        "sessionDir": session_dir,
        "model": MODEL,
        "realtimeEndpointIntent": REALTIME_ENDPOINT_INTENT,
        "delay": delay,
        "maxSeconds": max_seconds,
        "audioInput": audio_input,
        "error": error,
    });
    write_private(
        session_dir.join("status.json"),
        serde_json::to_string_pretty(&status)?,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn empty_completed_transcript_event_does_not_count_as_segment() {
        let session_dir = unique_tmp("nrr-empty-completed");
        fs::create_dir_all(&session_dir).unwrap();
        let event = json!({
            "type": "conversation.item.input_audio_transcription.completed",
            "transcript": ""
        });

        let kind = handle_event(
            &session_dir,
            &event.to_string(),
            42,
            &mut CaptureQuota::default(),
        )
        .unwrap();

        assert_eq!(kind, EventKind::Other);
        assert!(!session_dir.join("transcript.timeline.jsonl").exists());
        assert!(!session_dir.join("transcript.final.txt").exists());
    }

    #[test]
    fn session_update_disables_server_vad_for_realtime_whisper() {
        let update = session_update("low");

        assert!(update["session"]["audio"]["input"]["turn_detection"].is_null());
    }

    #[test]
    fn periodic_commit_requires_enough_audio_and_elapsed_time() {
        let interval = std::time::Duration::from_secs(5);
        let minimum = 4_800;

        assert!(!should_commit_audio_buffer(
            minimum - 1,
            std::time::Duration::from_secs(6),
            interval,
            minimum
        ));
        assert!(!should_commit_audio_buffer(
            minimum,
            std::time::Duration::from_secs(4),
            interval,
            minimum
        ));
        assert!(should_commit_audio_buffer(
            minimum,
            std::time::Duration::from_secs(5),
            interval,
            minimum
        ));
    }

    #[test]
    fn delta_transcript_event_counts_as_delta() {
        let session_dir = unique_tmp("nrr-delta-event");
        fs::create_dir_all(&session_dir).unwrap();
        let event = json!({
            "type": "conversation.item.input_audio_transcription.delta",
            "delta": "checking"
        });

        let kind = handle_event(
            &session_dir,
            &event.to_string(),
            42,
            &mut CaptureQuota::default(),
        )
        .unwrap();

        assert_eq!(kind, EventKind::Delta);
        assert!(session_dir.join("transcript.timeline.jsonl").exists());
        assert_eq!(
            fs::read_to_string(session_dir.join("transcript.live.txt")).unwrap(),
            "checking"
        );
    }

    fn unique_tmp(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        PathBuf::from("/private/tmp").join(format!("{prefix}-{nanos}"))
    }
}
