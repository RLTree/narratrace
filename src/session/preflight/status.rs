use super::shell_command::{join_tokens, quote_token};
use super::{ffmpeg_available, narration_quality_targets};
use crate::audio_input::auto_input_preview;
use crate::config::{Args, DEFAULT_MAX_SECONDS};
use anyhow::Result;
use serde_json::Value;
use serde_json::json;

const APP_HELPER_MANIFEST: &str = "/Users/terrynoblin/.codex/plugins/narrated-record-replay/skills/narrated-record-replay/Cargo.toml";
const APP_HELPER_CARGO_TARGET: &str = "/private/tmp/narrated-record-replay-cargo-target";

pub async fn preflight(args: &Args) -> Result<()> {
    let ffmpeg = ffmpeg_available();
    let has_openai_key = std::env::var("OPENAI_API_KEY").is_ok();
    let audio_input_preview = auto_input_preview();
    let payload = preflight_payload(args, ffmpeg, has_openai_key, audio_input_preview);
    let max_seconds = args.max_seconds.unwrap_or(DEFAULT_MAX_SECONDS);
    let local_ready = payload["localPrerequisitesReady"]
        .as_bool()
        .unwrap_or(false);
    let prepare_command = prepare_command(args, max_seconds);
    if args.json {
        println!("{}", serde_json::to_string_pretty(&payload)?);
    } else {
        println!("Ready for live narrated capture: {}", "no");
        println!(
            "Local prerequisites: {}",
            if local_ready { "ready" } else { "not ready" }
        );
        println!("ffmpeg: {}", if ffmpeg { "found" } else { "missing" });
        println!(
            "OpenAI key: {}",
            if has_openai_key { "present" } else { "missing" }
        );
        println!(
            "Goal: {}",
            if args.goal.is_some() {
                "provided"
            } else {
                "using default"
            }
        );
        println!("Recommended max seconds: {max_seconds}");
        println!(
            "Audio input preview: {}",
            payload["audioInputPreview"]["deviceName"]
                .as_str()
                .unwrap_or("unresolved")
        );
        println!("Recommended command: {prepare_command}");
    }
    Ok(())
}

fn preflight_payload(
    args: &Args,
    ffmpeg: bool,
    has_openai_key: bool,
    audio_input_preview: Value,
) -> Value {
    let max_seconds = args.max_seconds.unwrap_or(DEFAULT_MAX_SECONDS);
    let local_ready = ffmpeg && has_openai_key;
    let record_replay_ready = args.record_replay_status.as_deref() == Some("idle");
    let audio_input_ready = audio_preview_resolved(&audio_input_preview);
    let prepare_command = prepare_command(args, max_seconds);
    let dogfood_plan = live_dogfood_plan(&prepare_command);
    let blockers = preflight_blockers(
        ffmpeg,
        has_openai_key,
        args.goal.is_some(),
        args.record_replay_status.as_deref(),
    );
    json!({
        "schema": "narrated-record-replay.preflight.v1",
        "readyForLiveNarratedCapture": local_ready && record_replay_ready && audio_input_ready,
        "localPrerequisitesReady": local_ready,
        "recordReplayReady": record_replay_ready,
        "doesNotStartRecordReplay": true,
        "opensMicrophone": false,
        "callsOpenAI": false,
        "requiredConsentFlag": "--i-consent-to-microphone-capture",
        "localChecks": {
            "ffmpeg": ffmpeg,
            "hasOpenAIKey": has_openai_key,
            "goalProvided": args.goal.is_some(),
            "defaultGoalUsedWhenMissing": !args.goal.is_some()
        },
        "recordReplayStatus": {
            "source": "operator-provided-from-event-stream-status",
            "status": args.record_replay_status.as_deref().unwrap_or("not-confirmed"),
            "confirmed": args.record_replay_status.is_some()
        },
        "audioInputPreview": audio_input_preview,
        "transcriptionQualityPipeline": {
            "realtimeTimingSpine": {
                "model": "gpt-realtime-whisper",
                "delay": args.delay,
                "turnDetection": null,
                "wordAuthority": "timing-only"
            },
            "audioRetention": {
                "mode": args.audio_retention_mode,
                "localPrivate": true,
                "copiedIntoGeneratedPacketsByDefault": false
            },
            "batchTranscription": {
                "enabled": args.batch_transcription_enabled,
                "model": args.batch_transcription_model,
                "language": "en",
                "temperature": 0
            },
            "cleanup": {
                "enabled": args.cleanup_enabled,
                "model": args.cleanup_model,
                "dictionaryEntryCap": 100
            },
            "finalAlignment": "cleaned batch words mapped back to realtime timing windows"
        },
        "operatorActionsRequired": [
            "Confirm Record & Replay event-stream status before starting capture.",
            "Treat explicit narrated plugin or skill invocation as approval to add the helper consent flags for this bounded run.",
            "Use --max-seconds for bounded dogfood capture; default is 1800 seconds.",
            "Narrate enough workflow reasoning to meet the non-toy narration density target.",
            "Stop Record & Replay and narration explicitly before packet generation."
        ],
        "narrationQualityTargets": narration_quality_targets(),
        "liveDogfoodPlan": dogfood_plan,
        "recommendedCommand": prepare_command,
        "recommendedCommandOpensMicrophone": false,
        "recommendedCommandRequiresSeparateConsentQuestion": false,
        "coordinatedCaptureConsentSource": "explicit narrated plugin or skill invocation for this bounded run",
        "recommendedCommandAfterInvocation": "use liveDogfoodPlan step 3 inside the same orchestrated Record & Replay plus microphone start operation",
        "root": args.root,
        "blockers": blockers,
        "claimCeiling": "preflight only; does not prove live Record & Replay, microphone transcription, packet usefulness, or replay behavior"
    })
}

fn audio_preview_resolved(preview: &serde_json::Value) -> bool {
    preview.get("status").and_then(serde_json::Value::as_str) == Some("resolved")
        && preview
            .get("rejectsIphoneOrVirtualInput")
            .and_then(serde_json::Value::as_bool)
            == Some(false)
}

fn prepare_command(args: &Args, max_seconds: u64) -> String {
    let goal = args
        .goal
        .as_deref()
        .unwrap_or("narrated record and replay capture");
    let suffix = join_tokens(&[
        "--".into(),
        "prepare-coordinated-session".into(),
        "--goal".into(),
        goal.into(),
        "--root".into(),
        args.root.to_string_lossy().into_owned(),
        "--delay".into(),
        args.delay.clone(),
        "--input".into(),
        args.input.clone(),
        "--max-seconds".into(),
        max_seconds.to_string(),
    ]);
    format!("{} {suffix}", app_helper_command_prefix())
}
