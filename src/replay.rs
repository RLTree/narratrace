use crate::config::{Args, required_session_dir};
use crate::private_fs::write_private;
use crate::safe_path::read_regular_text_bounded;
use anyhow::{Result, bail};
use serde_json::{Value, json};
#[cfg(test)]
use std::fs;
use std::path::Path;

const MAX_REPLAY_PARAMETERS_BYTES: u64 = 8 * 1024 * 1024;
const MAX_REPLAY_BINDINGS: usize = 10_000;

pub fn preview_replay_voice(args: &Args) -> Result<()> {
    let session_dir = required_session_dir(args)?;
    let voice_path = session_dir.join("replay-voice-parameters.json");
    let voice = read_json(&voice_path)?;
    let bindings = voice
        .get("segmentBindings")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow::anyhow!("replay-voice-parameters.json missing segmentBindings"))?;
    if bindings.is_empty() {
        bail!("replay-voice-parameters.json has no segmentBindings to preview");
    }
    if bindings.len() > MAX_REPLAY_BINDINGS {
        bail!("replay-voice-parameters.json exceeds segment binding limit");
    }

    let cues = bindings.iter().map(voice_cue).collect::<Result<Vec<_>>>()?;
    let plan_path = session_dir.join("replay-voice-execution-plan.json");
    write_private(
        &plan_path,
        serde_json::to_string_pretty(&json!({
            "schema": "narrated-record-replay.replay-voice-execution-plan.v1",
            "status": "dry-run-not-spoken",
            "claimCeiling": "deterministic replay voice scheduling preview only; no audio playback or live demonstration",
            "sourceArtifact": voice_path.display().to_string(),
            "cueCount": cues.len(),
            "cues": cues,
            "proofBoundary": {
                "consumesReplayVoiceParameters": true,
                "speaksAudio": false,
                "requiresLiveDemonstrationForClaim": "CLAIM-013"
            }
        }))?,
    )?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "replayVoiceExecutionPlanPath": plan_path,
            "sessionDir": session_dir,
            "status": "dry-run-not-spoken",
            "cueCount": cues.len()
        }))?
    );
    Ok(())
}

fn voice_cue(binding: &Value) -> Result<Value> {
    let voice = binding.get("voice").unwrap_or(&Value::Null);
    let style = required_text(voice, "style")?;
    let pace = required_text(voice, "pace")?;
    let emphasis = required_text(voice, "emphasis")?;
    let timeline = binding.get("timelineBinding").unwrap_or(binding);
    let start_ms = required_i64(timeline, "startMs")?;
    let end_ms = required_i64(timeline, "endMs")?;
    if end_ms < start_ms {
        bail!("replay voice binding endMs is before startMs");
    }
    let base_duration = end_ms.saturating_sub(start_ms).max(1);
    let scheduled_duration = apply_pace(base_duration, pace);
    Ok(json!({
        "transcriptSegmentId": binding.get("transcriptSegmentId").cloned().unwrap_or(Value::Null),
        "scheduledAtMs": start_ms,
        "sourceWindowMs": [start_ms, end_ms],
        "plannedDurationMs": scheduled_duration,
        "voice": {
            "style": style,
            "pace": pace,
            "emphasis": emphasis
        },
        "previewInstruction": {
            "tone": style_tone(style),
            "paceMultiplier": pace_multiplier(pace),
            "emphasisGain": emphasis_gain(emphasis)
        },
        "executionStatus": "dry-run-not-spoken"
    }))
}

fn required_text<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("replay voice binding missing {key}"))
}

fn required_i64(value: &Value, key: &str) -> Result<i64> {
    value
        .get(key)
        .and_then(Value::as_i64)
        .ok_or_else(|| anyhow::anyhow!("replay voice binding missing {key}"))
}

fn apply_pace(base_duration: i64, pace: &str) -> i64 {
    match pace {
        "slow" => base_duration.saturating_mul(5) / 4,
        "fast" => base_duration.saturating_mul(3) / 4,
        _ => base_duration,
    }
}

fn pace_multiplier(pace: &str) -> f64 {
    match pace {
        "slow" => 0.8,
        "fast" => 1.25,
        _ => 1.0,
    }
}

fn emphasis_gain(emphasis: &str) -> f64 {
    match emphasis {
        "low" => 0.85,
        "high" => 1.2,
        _ => 1.0,
    }
}

fn style_tone(style: &str) -> &'static str {
    match style {
        "calm" => "steady and low-variance",
        "focused" => "concise and task-forward",
        "energetic" => "brighter and higher-variance",
        _ => "neutral and literal",
    }
}

fn read_json(path: &Path) -> Result<Value> {
    let contents = read_regular_text_bounded(path, MAX_REPLAY_PARAMETERS_BYTES)?;
    Ok(serde_json::from_str(&contents)?)
}

#[cfg(test)]
#[path = "replay_extra_tests.rs"]
mod replay_extra_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[cfg(unix)]
    #[test]
    fn replay_voice_preview_rejects_symlinked_parameters() {
        let root = unique_tmp("nrr-replay-voice-symlink");
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("private.json"), r#"{"segmentBindings":[]}"#).unwrap();
        std::os::unix::fs::symlink(
            root.join("private.json"),
            root.join("replay-voice-parameters.json"),
        )
        .unwrap();

        assert!(read_json(&root.join("replay-voice-parameters.json")).is_err());
    }

    #[test]
    fn voice_cue_applies_pace_and_instruction_metadata() {
        let cue = voice_cue(&json!({
            "transcriptSegmentId": "seg-1",
            "timelineBinding": { "startMs": 100, "endMs": 500 },
            "voice": { "style": "focused", "pace": "fast", "emphasis": "high" }
        }))
        .unwrap();

        assert_eq!(cue["transcriptSegmentId"], "seg-1");
        assert_eq!(cue["scheduledAtMs"], 100);
        assert_eq!(cue["plannedDurationMs"], 300);
        assert_eq!(
            cue["previewInstruction"]["tone"],
            "concise and task-forward"
        );
        assert_eq!(cue["previewInstruction"]["paceMultiplier"], 1.25);
        assert_eq!(cue["previewInstruction"]["emphasisGain"], 1.2);
    }

    #[test]
    fn voice_cue_rejects_inverted_window() {
        let error = voice_cue(&json!({
            "timelineBinding": { "startMs": 500, "endMs": 100 },
            "voice": { "style": "calm", "pace": "slow", "emphasis": "low" }
        }))
        .unwrap_err()
        .to_string();

        assert!(error.contains("endMs is before startMs"));
    }

    #[test]
    fn preview_replay_voice_writes_dry_run_plan_without_speaking_audio() {
        let root = unique_tmp("nrr-replay-voice-preview");
        fs::create_dir_all(&root).unwrap();
        fs::write(
            root.join("replay-voice-parameters.json"),
            r#"{
              "segmentBindings":[{
                "transcriptSegmentId":"seg-1",
                "timelineBinding":{"startMs":0,"endMs":1000},
                "voice":{"style":"neutral","pace":"normal","emphasis":"balanced"}
              }]
            }"#,
        )
        .unwrap();
        let args = crate::config::parse_args_from([
            "nrr",
            "replay-voice-preview",
            "--session-dir",
            root.to_str().unwrap(),
            "--i-consent-to-custom-runtime-paths",
        ])
        .unwrap();

        preview_replay_voice(&args).unwrap();

        let plan: Value = serde_json::from_str(
            &fs::read_to_string(root.join("replay-voice-execution-plan.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(plan["status"], "dry-run-not-spoken");
        assert_eq!(plan["proofBoundary"]["speaksAudio"], false);
        assert_eq!(plan["cueCount"], 1);
    }

    #[cfg(unix)]
    fn unique_tmp(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        PathBuf::from("/private/tmp").join(format!("{prefix}-{nanos}"))
    }
}
