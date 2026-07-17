use crate::audio_input::AUTO_INPUT;
use anyhow::{Result, bail};
use std::env;
use std::path::PathBuf;

mod parsing;
mod types;
mod validation;

pub use parsing::{MAX_CAPTURE_SECONDS, slugify, usage};
use parsing::{
    normalize_cli_path, parse_audio_filter, parse_audio_retention_mode,
    parse_avfoundation_input_arg, parse_batch_model, parse_bool_env, parse_capture_seconds,
    parse_delay, parse_path, parse_record_replay_status, required_value,
};
pub use types::*;
pub use validation::required_session_dir;
use validation::validate_args;

#[cfg(test)]
mod parse_extra_tests;
#[cfg(test)]
mod parsing_contract_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod value_flag_tests;

pub fn parse_args() -> Result<Args> {
    parse_args_from(env::args())
}

pub fn parse_args_from<I, S>(input: I) -> Result<Args>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut raw = input.into_iter().map(Into::into).skip(1);
    let command = raw.next().unwrap_or_else(|| "help".to_string());
    let mut args = Args {
        command,
        goal: None,
        root: PathBuf::from(
            env::var("NARRATED_REPLAY_ROOT").unwrap_or_else(|_| DEFAULT_ROOT.into()),
        ),
        skill_dir: None,
        session_dir: None,
        recording_metadata: None,
        recording_events: None,
        baseline_delay_evaluation: None,
        candidate_delay_evaluation: None,
        coverage_json: None,
        coverage_receipt: None,
        delay: env::var("NARRATED_REPLAY_REALTIME_DELAY")
            .unwrap_or_else(|_| DEFAULT_REALTIME_DELAY.to_string()),
        input: env::var("NARRATED_REPLAY_AVFOUNDATION_INPUT")
            .unwrap_or_else(|_| AUTO_INPUT.to_string()),
        max_seconds: Some(DEFAULT_MAX_SECONDS),
        record_replay_status: None,
        microphone_capture_consent: false,
        openai_postprocessing_consent: false,
        custom_runtime_path_consent: false,
        custom_audio_filter_consent: false,
        batch_transcription_enabled: parse_bool_env("NARRATED_REPLAY_BATCH_TRANSCRIPTION", true),
        cleanup_enabled: parse_bool_env("NARRATED_REPLAY_CLEANUP", true),
        batch_transcription_model: env::var("NARRATED_REPLAY_BATCH_MODEL")
            .unwrap_or_else(|_| DEFAULT_BATCH_TRANSCRIPTION_MODEL.to_string()),
        cleanup_model: env::var("NARRATED_REPLAY_CLEANUP_MODEL")
            .unwrap_or_else(|_| DEFAULT_CLEANUP_MODEL.to_string()),
        audio_retention_mode: env::var("NARRATED_REPLAY_AUDIO_RETENTION_MODE")
            .unwrap_or_else(|_| "private-wav".to_string()),
        audio_retention_path: env::var("NARRATED_REPLAY_AUDIO_RETENTION_PATH")
            .ok()
            .map(PathBuf::from),
        audio_filter: env::var("NARRATED_REPLAY_AUDIO_FILTER")
            .unwrap_or_else(|_| DEFAULT_AUDIO_FILTER.to_string()),
        cleanup_dictionary_source: env::var("NARRATED_REPLAY_CLEANUP_DICTIONARY")
            .ok()
            .map(PathBuf::from),
        replay_voice_style: "neutral".to_string(),
        replay_voice_pace: "normal".to_string(),
        replay_voice_emphasis: "balanced".to_string(),
        receipt_run_id: None,
        receipt_generated_at: None,
        json: false,
    };
    while let Some(token) = raw.next() {
        match token.as_str() {
            "--goal" => args.goal = raw.next(),
            "--root" => args.root = parse_path("--root", &required_value("--root", raw.next())?)?,
            "--skill-dir" => {
                args.skill_dir = Some(parse_path(
                    "--skill-dir",
                    &required_value("--skill-dir", raw.next())?,
                )?)
            }
            "--session-dir" => {
                args.session_dir = Some(parse_path(
                    "--session-dir",
                    &required_value("--session-dir", raw.next())?,
                )?)
            }
            "--recording-metadata" => args.recording_metadata = raw.next(),
            "--recording-events" => args.recording_events = raw.next(),
            "--baseline-delay-evaluation" => {
                args.baseline_delay_evaluation = Some(parse_path(
                    "--baseline-delay-evaluation",
                    &required_value("--baseline-delay-evaluation", raw.next())?,
                )?)
            }
            "--candidate-delay-evaluation" => {
                args.candidate_delay_evaluation = Some(parse_path(
                    "--candidate-delay-evaluation",
                    &required_value("--candidate-delay-evaluation", raw.next())?,
                )?)
            }
            "--coverage-json" => {
                args.coverage_json = Some(parse_path(
                    "--coverage-json",
                    &required_value("--coverage-json", raw.next())?,
                )?)
            }
            "--coverage-receipt" => {
                args.coverage_receipt = Some(parse_path(
                    "--coverage-receipt",
                    &required_value("--coverage-receipt", raw.next())?,
                )?)
            }
            "--delay" => args.delay = required_value("--delay", raw.next())?,
            "--input" => args.input = required_value("--input", raw.next())?,
            "--max-seconds" => {
                args.max_seconds = Some(parse_capture_seconds(
                    "--max-seconds",
                    &required_value("--max-seconds", raw.next())?,
                )?)
            }
            "--record-replay-status" => {
                args.record_replay_status = Some(parse_record_replay_status(&required_value(
                    "--record-replay-status",
                    raw.next(),
                )?)?)
            }
            "--i-consent-to-microphone-capture" => args.microphone_capture_consent = true,
            "--i-consent-to-openai-postprocessing" => args.openai_postprocessing_consent = true,
            "--i-consent-to-custom-runtime-paths" => args.custom_runtime_path_consent = true,
            "--i-consent-to-custom-audio-filter" => args.custom_audio_filter_consent = true,
            "--disable-batch-transcription" => args.batch_transcription_enabled = false,
            "--enable-batch-transcription" => args.batch_transcription_enabled = true,
            "--disable-cleanup" => args.cleanup_enabled = false,
            "--enable-cleanup" => args.cleanup_enabled = true,
            "--batch-transcription-model" => {
                args.batch_transcription_model =
                    parse_batch_model(&required_value("--batch-transcription-model", raw.next())?)?
            }
            "--cleanup-model" => {
                args.cleanup_model = required_value("--cleanup-model", raw.next())?
            }
            "--audio-retention-mode" => {
                args.audio_retention_mode = parse_audio_retention_mode(&required_value(
                    "--audio-retention-mode",
                    raw.next(),
                )?)?
            }
            "--audio-retention-path" => {
                args.audio_retention_path = Some(parse_path(
                    "--audio-retention-path",
                    &required_value("--audio-retention-path", raw.next())?,
                )?)
            }
            "--audio-filter" => {
                args.audio_filter =
                    parse_audio_filter(&required_value("--audio-filter", raw.next())?)?
            }
            "--cleanup-dictionary-source" => {
                args.cleanup_dictionary_source = Some(parse_path(
                    "--cleanup-dictionary-source",
                    &required_value("--cleanup-dictionary-source", raw.next())?,
                )?)
            }
            "--replay-voice-style" => {
                args.replay_voice_style = required_value("--replay-voice-style", raw.next())?
            }
            "--replay-voice-pace" => {
                args.replay_voice_pace = required_value("--replay-voice-pace", raw.next())?
            }
            "--replay-voice-emphasis" => {
                args.replay_voice_emphasis = required_value("--replay-voice-emphasis", raw.next())?
            }
            "--receipt-run-id" => {
                args.receipt_run_id = Some(required_value("--receipt-run-id", raw.next())?)
            }
            "--receipt-generated-at" => {
                args.receipt_generated_at =
                    Some(required_value("--receipt-generated-at", raw.next())?)
            }
            "--json" => args.json = true,
            "--help" | "-h" => args.command = "help".to_string(),
            other => bail!("unknown argument: {other}"),
        }
    }
    args.delay = parse_delay(&args.delay)?;
    args.input = parse_avfoundation_input_arg(&args.input)?;
    args.batch_transcription_model = parse_batch_model(&args.batch_transcription_model)?;
    args.audio_retention_mode = parse_audio_retention_mode(&args.audio_retention_mode)?;
    args.audio_filter = parse_audio_filter(&args.audio_filter)?;
    args.root = normalize_cli_path(&args.root);
    if let Some(session_dir) = &args.session_dir {
        args.session_dir = Some(normalize_cli_path(session_dir));
    }
    if let Some(path) = &args.audio_retention_path {
        args.audio_retention_path = Some(normalize_cli_path(path));
    }
    if let Some(path) = &args.cleanup_dictionary_source {
        args.cleanup_dictionary_source = Some(normalize_cli_path(path));
    }
    if let Some(path) = &args.coverage_json {
        args.coverage_json = Some(normalize_cli_path(path));
    }
    if let Some(path) = &args.coverage_receipt {
        args.coverage_receipt = Some(normalize_cli_path(path));
    }
    validate_args(&args)?;
    Ok(args)
}
