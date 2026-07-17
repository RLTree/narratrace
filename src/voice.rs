use crate::config::Args;
use crate::private_fs::write_private;
use anyhow::{Result, bail};
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};

pub struct ReplayVoiceArtifact {
    pub path: PathBuf,
    pub segment_binding_count: usize,
}

pub fn write_replay_voice_parameters(
    session_dir: &Path,
    temporal_context_path: &Path,
    args: &Args,
) -> Result<ReplayVoiceArtifact> {
    validate_voice_args(args)?;
    let context = read_json(temporal_context_path).unwrap_or(Value::Null);
    let segments = context
        .get("transcriptSegments")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let bindings = segments
        .iter()
        .enumerate()
        .map(|(index, segment)| {
            let start_ms = segment.get("startMs").cloned().unwrap_or(Value::Null);
            let end_ms = segment.get("endMs").cloned().unwrap_or(Value::Null);
            json!({
                "transcriptSegmentId": segment.get("id").cloned().unwrap_or(Value::Null),
                "transcriptWindowMs": [
                    start_ms,
                    end_ms
                ],
                "timelineBinding": {
                    "source": format!("temporal-context.transcriptSegments[{index}]"),
                    "clockDomain": "transcript-audio-offset-ms",
                    "startMs": segment.get("startMs").cloned().unwrap_or(Value::Null),
                    "endMs": segment.get("endMs").cloned().unwrap_or(Value::Null),
                    "bindingStatus": "ready-for-replay-engine"
                },
                "voice": {
                    "style": args.replay_voice_style,
                    "pace": args.replay_voice_pace,
                    "emphasis": args.replay_voice_emphasis
                },
                "source": "packet-generation-parameter",
                "replayStatus": "planned-not-executed"
            })
        })
        .collect::<Vec<_>>();
    let path = session_dir.join("replay-voice-parameters.json");
    write_private(
        &path,
        serde_json::to_string_pretty(&json!({
            "schema": "narrated-record-replay.replay-voice-parameters.v1",
            "status": "planned-not-executed",
            "claimCeiling": "typed replay-time voice parameters only; replay behavior and live demonstration still owed",
            "defaults": {
                "style": args.replay_voice_style,
                "pace": args.replay_voice_pace,
                "emphasis": args.replay_voice_emphasis
            },
            "allowedValues": {
                "style": ["neutral", "calm", "focused", "energetic"],
                "pace": ["slow", "normal", "fast"],
                "emphasis": ["low", "balanced", "high"]
            },
            "timelineBindingContract": {
                "sourceArtifact": "temporal-context.json",
                "sourceFields": [
                    "transcriptSegments[].id",
                    "transcriptSegments[].startMs",
                    "transcriptSegments[].endMs"
                ],
                "clockDomain": "transcript-audio-offset-ms",
                "executionStatus": "not-connected-to-replay-engine",
                "enginePreconditions": [
                    "replay engine reads this artifact",
                    "replay engine maps transcript-audio-offset-ms to playback time using temporal-context alignmentPolicy",
                    "replay engine records a voice execution receipt before CLAIM-013 can close"
                ],
                "unsupportedClaims": [
                    "replay voice behavior executed",
                    "style, pace, or emphasis changed playback",
                    "live demonstration used these parameters"
                ]
            },
            "proofObligations": [
                "typed timeline contract stays valid for every transcript segment binding",
                "replay behavior tests prove a replay engine consumes style, pace, and emphasis",
                "live demonstration proves replay-time voice parameters affect an actual replay"
            ],
            "segmentBindings": bindings,
        }))?,
    )?;
    Ok(ReplayVoiceArtifact {
        path,
        segment_binding_count: bindings.len(),
    })
}

fn validate_voice_args(args: &Args) -> Result<()> {
    if !matches!(
        args.replay_voice_style.as_str(),
        "neutral" | "calm" | "focused" | "energetic"
    ) {
        bail!("--replay-voice-style must be one of: neutral, calm, focused, energetic");
    }
    if !matches!(args.replay_voice_pace.as_str(), "slow" | "normal" | "fast") {
        bail!("--replay-voice-pace must be one of: slow, normal, fast");
    }
    if !matches!(
        args.replay_voice_emphasis.as_str(),
        "low" | "balanced" | "high"
    ) {
        bail!("--replay-voice-emphasis must be one of: low, balanced, high");
    }
    Ok(())
}

fn read_json(path: &Path) -> Result<Value> {
    Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn replay_voice_parameters_fall_back_to_empty_segments_for_bad_context() {
        let root = unique_tmp("nrr-voice-bad-context");
        fs::create_dir_all(&root).unwrap();
        let context = root.join("temporal-context.json");
        fs::write(&context, "{not-json").unwrap();
        let args = args_with_voice("calm", "slow", "low");

        let artifact = write_replay_voice_parameters(&root, &context, &args).unwrap();
        let written: Value =
            serde_json::from_str(&fs::read_to_string(&artifact.path).unwrap()).unwrap();

        assert_eq!(artifact.segment_binding_count, 0);
        assert_eq!(written["segmentBindings"].as_array().unwrap().len(), 0);
        assert_eq!(written["defaults"]["style"], "calm");
    }

    #[test]
    fn replay_voice_parameters_reject_invalid_voice_dimensions() {
        for (style, pace, emphasis, expected) in [
            ("loud", "normal", "balanced", "--replay-voice-style"),
            ("neutral", "rushed", "balanced", "--replay-voice-pace"),
            ("neutral", "normal", "extreme", "--replay-voice-emphasis"),
        ] {
            let args = args_with_voice(style, pace, emphasis);
            let error = validate_voice_args(&args).unwrap_err().to_string();

            assert!(error.contains(expected));
        }
    }

    fn args_with_voice(style: &str, pace: &str, emphasis: &str) -> Args {
        let mut args = crate::config::parse_args_from(["nrr", "packet"]).unwrap();
        args.replay_voice_style = style.to_string();
        args.replay_voice_pace = pace.to_string();
        args.replay_voice_emphasis = emphasis.to_string();
        args
    }

    fn unique_tmp(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        PathBuf::from("/private/tmp").join(format!("{prefix}-{nanos}"))
    }
}
