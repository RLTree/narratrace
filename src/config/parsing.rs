use crate::audio_input::AUTO_INPUT;
use crate::config::DEFAULT_AUDIO_FILTER;
use crate::safe_path::validate_cli_path;
use anyhow::{Result, bail};
use std::env;
use std::path::{Path, PathBuf};

pub const MAX_CAPTURE_SECONDS: u64 = 1_800;

pub fn usage() -> &'static str {
    "Usage:
  narrated-record-replay preflight [--json] [--goal <text>] [--root <dir>] [--max-seconds <1..=1800>] [--record-replay-status idle|recording|unavailable]
  narrated-record-replay validate [--json]
  narrated-record-replay validate-bundle --skill-dir <dir> [--receipt-run-id <id>] [--receipt-generated-at <iso-time>]
  narrated-record-replay refresh-bundle-receipt --skill-dir <dir> [--receipt-run-id <id>]
  narrated-record-replay check-coverage-policy --skill-dir <dir>
  narrated-record-replay coverage-receipt --skill-dir <dir> --coverage-json <path> --coverage-receipt <path> --receipt-run-id <coverage-command> --receipt-generated-at <iso-time>
  narrated-record-replay prepare-coordinated-session [--goal <text>] [--root <dir>] [--delay minimal|low|medium|high|xhigh] [--input auto|:<audio-index>|:<audio-device-name>] [--max-seconds <1..=1800>] [--audio-retention-mode private-wav|disabled] [--audio-retention-path <path>] [--audio-filter default|voice-default|<custom-with-explicit-consent>]
  narrated-record-replay start [--goal <text>] --i-consent-to-microphone-capture --record-replay-status idle [--root <dir>] [--delay minimal|low|medium|high|xhigh] [--input auto|:<audio-index>|:<audio-device-name>] [--max-seconds <1..=1800>] [--audio-retention-mode private-wav|disabled] [--audio-retention-path <path>] [--audio-filter default|voice-default|<custom-with-explicit-consent>]
  narrated-record-replay status --session-dir <dir>
  narrated-record-replay stop --session-dir <dir>
  narrated-record-replay packet --session-dir <dir> [--recording-metadata <path>] [--recording-events <path>] [--i-consent-to-openai-postprocessing] [--disable-batch-transcription] [--disable-cleanup] [--batch-transcription-model gpt-4o-transcribe|gpt-4o-mini-transcribe] [--cleanup-model <model>] [--cleanup-dictionary-source <path>] [--replay-voice-style neutral|calm|focused|energetic] [--replay-voice-pace slow|normal|fast] [--replay-voice-emphasis low|balanced|high]
  narrated-record-replay parent-operation-receipt --session-dir <dir> --recording-metadata <path> --recording-events <path>
  narrated-record-replay receipt --session-dir <dir> [--recording-metadata <path>] [--recording-events <path>]
  narrated-record-replay delay-eval --session-dir <dir> [--recording-metadata <path>] [--recording-events <path>]
  narrated-record-replay delay-compare --session-dir <dir> --baseline-delay-evaluation <path> --candidate-delay-evaluation <path>
  narrated-record-replay inspect --session-dir <dir>
  narrated-record-replay review --session-dir <dir>
  narrated-record-replay replay-voice-preview --session-dir <dir>"
}

pub(super) fn parse_bool_env(name: &str, default: bool) -> bool {
    match env::var(name).ok().as_deref() {
        Some("1") | Some("true") | Some("TRUE") | Some("yes") | Some("on") => true,
        Some("0") | Some("false") | Some("FALSE") | Some("no") | Some("off") => false,
        _ => default,
    }
}

pub(super) fn required_value(flag: &str, value: Option<String>) -> Result<String> {
    match value {
        Some(value) if !value.starts_with("--") => Ok(value),
        _ => bail!("{flag} requires a value"),
    }
}

pub(super) fn parse_path(flag: &str, value: &str) -> Result<PathBuf> {
    let path = normalize_cli_path(Path::new(value));
    validate_cli_path(flag, &path)?;
    Ok(path)
}

pub(super) fn normalize_cli_path(path: &Path) -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        if let Ok(stripped) = path.strip_prefix("/tmp") {
            return Path::new("/private/tmp").join(stripped);
        }
    }
    path.to_path_buf()
}

pub(super) fn parse_capture_seconds(flag: &str, value: &str) -> Result<u64> {
    let parsed = value
        .parse::<u64>()
        .map_err(|_| anyhow::anyhow!("{flag} must be a positive integer"))?;
    if parsed == 0 {
        bail!("{flag} must be a positive integer");
    }
    if parsed > MAX_CAPTURE_SECONDS {
        bail!("{flag} must not exceed {MAX_CAPTURE_SECONDS} seconds");
    }
    Ok(parsed)
}

pub(super) fn parse_record_replay_status(value: &str) -> Result<String> {
    match value {
        "idle" | "recording" | "unavailable" => Ok(value.to_string()),
        _ => bail!("--record-replay-status must be one of idle, recording, unavailable"),
    }
}

pub(super) fn parse_delay(value: &str) -> Result<String> {
    match value {
        "minimal" | "low" | "medium" | "high" | "xhigh" => Ok(value.to_string()),
        _ => bail!("--delay must be one of minimal, low, medium, high, xhigh"),
    }
}

pub(super) fn parse_batch_model(value: &str) -> Result<String> {
    match value {
        "gpt-4o-transcribe" | "gpt-4o-mini-transcribe" => Ok(value.to_string()),
        _ => {
            bail!("--batch-transcription-model must be gpt-4o-transcribe or gpt-4o-mini-transcribe")
        }
    }
}

pub(super) fn parse_audio_retention_mode(value: &str) -> Result<String> {
    match value {
        "private-wav" | "disabled" => Ok(value.to_string()),
        _ => bail!("--audio-retention-mode must be private-wav or disabled"),
    }
}

pub(super) fn parse_audio_filter(value: &str) -> Result<String> {
    let trimmed = value.trim();
    if matches!(trimmed, "default" | "voice-default") {
        return Ok(DEFAULT_AUDIO_FILTER.to_string());
    }
    if trimmed.is_empty() || trimmed.contains('\0') || trimmed.contains('\n') {
        bail!(
            "--audio-filter must be default, voice-default, or a non-empty single-line custom filtergraph with explicit consent"
        );
    }
    Ok(trimmed.to_string())
}

pub(super) fn parse_avfoundation_input_arg(value: &str) -> Result<String> {
    match value {
        "" | AUTO_INPUT | "default" | ":default" => Ok(AUTO_INPUT.to_string()),
        _ => {
            let Some(device) = value.strip_prefix(':') else {
                bail!(
                    "--input must be auto, an AVFoundation audio index like :4, or a device name like ':MacBook Pro Microphone'"
                );
            };
            if device.is_empty()
                || device.contains(':')
                || device.contains('\0')
                || device.contains('\n')
                || device.contains('\r')
            {
                bail!(
                    "--input must be auto, an AVFoundation audio index like :4, or a device name like ':MacBook Pro Microphone'"
                );
            }
            Ok(value.to_string())
        }
    }
}

pub fn slugify(value: &str) -> String {
    let mut out = String::new();
    let mut last_dash = false;
    for ch in value.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            last_dash = false;
        } else if !last_dash && !out.is_empty() {
            out.push('-');
            last_dash = true;
        }
        if out.len() >= 48 {
            break;
        }
    }
    out.trim_matches('-').to_string()
}
