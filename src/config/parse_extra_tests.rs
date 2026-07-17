use super::*;
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn parse_args_uses_help_when_no_command_is_supplied() {
    let args = parse_args_from(["nrr"]).unwrap();

    assert_eq!(args.command, "help");
}

#[test]
fn parses_path_flags_and_receipt_metadata() {
    let args = parse_args_from([
        "nrr",
        "delay-compare",
        "--skill-dir",
        "/private/tmp/narrated-record-replay/skill",
        "--session-dir",
        "/private/tmp/narrated-record-replay/session",
        "--recording-metadata",
        "/private/tmp/narrated-record-replay/recording.json",
        "--recording-events",
        "/private/tmp/narrated-record-replay/events.jsonl",
        "--baseline-delay-evaluation",
        "/private/tmp/narrated-record-replay/high.json",
        "--candidate-delay-evaluation",
        "/private/tmp/narrated-record-replay/low.json",
        "--coverage-json",
        "/private/tmp/narrated-record-replay/coverage.json",
        "--coverage-receipt",
        "/private/tmp/narrated-record-replay/coverage-receipt.json",
        "--cleanup-dictionary-source",
        "/private/tmp/narrated-record-replay/dictionary.txt",
        "--receipt-run-id",
        "coverage-001",
        "--receipt-generated-at",
        "2026-06-24T00:00:00Z",
        "--i-consent-to-custom-runtime-paths",
        "--json",
    ])
    .unwrap();

    assert_eq!(args.command, "delay-compare");
    assert!(args.skill_dir.is_some());
    assert!(args.session_dir.is_some());
    assert!(args.baseline_delay_evaluation.is_some());
    assert!(args.candidate_delay_evaluation.is_some());
    assert!(args.coverage_json.is_some());
    assert!(args.coverage_receipt.is_some());
    assert!(args.cleanup_dictionary_source.is_some());
    assert_eq!(args.receipt_run_id.as_deref(), Some("coverage-001"));
    assert!(args.json);
}

#[test]
fn parses_runtime_toggles_and_replay_voice_options() {
    let args = parse_args_from([
        "nrr",
        "packet",
        "--session-dir",
        "/private/tmp/narrated-record-replay/nrr-config-test",
        "--enable-batch-transcription",
        "--enable-cleanup",
        "--replay-voice-style",
        "focused",
        "--replay-voice-pace",
        "fast",
        "--replay-voice-emphasis",
        "high",
    ])
    .unwrap();

    assert!(args.batch_transcription_enabled);
    assert!(args.cleanup_enabled);
    assert_eq!(args.replay_voice_style, "focused");
    assert_eq!(args.replay_voice_pace, "fast");
    assert_eq!(args.replay_voice_emphasis, "high");
}

#[test]
fn rejects_missing_values_and_unknown_arguments() {
    assert!(parse_args_from(["nrr", "preflight", "--max-seconds"]).is_err());
    assert!(parse_args_from(["nrr", "preflight", "--unknown"]).is_err());
}

#[test]
fn rejects_invalid_boundary_values() {
    assert!(parse_args_from(["nrr", "preflight", "--delay", "instant"]).is_err());
    assert!(parse_args_from(["nrr", "preflight", "--max-seconds", "0"]).is_err());
    assert!(parse_args_from(["nrr", "preflight", "--record-replay-status", "paused"]).is_err());
    assert!(
        parse_args_from([
            "nrr",
            "preflight",
            "--batch-transcription-model",
            "whisper-1",
        ])
        .is_err()
    );
    assert!(parse_args_from(["nrr", "preflight", "--audio-retention-mode", "public"]).is_err());
    assert!(parse_args_from(["nrr", "preflight", "--input", "MacBook Pro Microphone"]).is_err());
    assert!(parse_args_from(["nrr", "preflight", "--audio-filter", ""]).is_err());
}

#[test]
fn capture_duration_accepts_documented_maximum_and_rejects_larger_values() {
    let maximum = MAX_CAPTURE_SECONDS.to_string();
    let too_large = (MAX_CAPTURE_SECONDS + 1).to_string();

    let args = parse_args_from(["nrr", "preflight", "--max-seconds", &maximum]).unwrap();

    assert_eq!(args.max_seconds, Some(MAX_CAPTURE_SECONDS));
    assert!(parse_args_from(["nrr", "preflight", "--max-seconds", &too_large]).is_err());
    assert!(parse_args_from(["nrr", "preflight", "--max-seconds", &u64::MAX.to_string()]).is_err());
    assert!(usage().contains("--max-seconds <1..=1800>"));
}

#[test]
fn parses_environment_defaults_and_normalizes_optional_paths() {
    let _guard = ENV_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    clear_env_defaults();
    unsafe {
        std::env::set_var(
            "NARRATED_REPLAY_ROOT",
            "/private/tmp/narrated-record-replay/root",
        );
        std::env::set_var("NARRATED_REPLAY_REALTIME_DELAY", "xhigh");
        std::env::set_var("NARRATED_REPLAY_AVFOUNDATION_INPUT", "auto");
        std::env::set_var("NARRATED_REPLAY_BATCH_TRANSCRIPTION", "false");
        std::env::set_var("NARRATED_REPLAY_CLEANUP", "false");
        std::env::set_var("NARRATED_REPLAY_BATCH_MODEL", "gpt-4o-mini-transcribe");
        std::env::set_var("NARRATED_REPLAY_CLEANUP_MODEL", "gpt-5-mini");
        std::env::set_var("NARRATED_REPLAY_AUDIO_RETENTION_MODE", "private-wav");
        std::env::set_var(
            "NARRATED_REPLAY_AUDIO_RETENTION_PATH",
            "/private/tmp/narrated-record-replay/audio/retained.wav",
        );
        std::env::set_var(
            "NARRATED_REPLAY_CLEANUP_DICTIONARY",
            "/private/tmp/narrated-record-replay/dict/terms.txt",
        );
    }

    let args = parse_args_from(["nrr", "preflight"]).unwrap();

    clear_env_defaults();

    assert_eq!(args.delay, "xhigh");
    assert!(!args.batch_transcription_enabled);
    assert!(!args.cleanup_enabled);
    assert_eq!(args.batch_transcription_model, "gpt-4o-mini-transcribe");
    assert_eq!(args.cleanup_model, "gpt-5-mini");
    assert!(args.root.ends_with("root"));
    assert!(
        args.audio_retention_path
            .unwrap()
            .ends_with("audio/retained.wav")
    );
    assert!(
        args.cleanup_dictionary_source
            .unwrap()
            .ends_with("dict/terms.txt")
    );
}

#[test]
fn rejects_invalid_environment_defaults_after_cli_parse() {
    let _guard = ENV_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    clear_env_defaults();
    unsafe {
        std::env::set_var("NARRATED_REPLAY_REALTIME_DELAY", "turbo");
        std::env::set_var("NARRATED_REPLAY_BATCH_TRANSCRIPTION", "not-bool");
    }

    let error = parse_args_from(["nrr", "preflight"]).unwrap_err();

    clear_env_defaults();

    assert!(
        error
            .to_string()
            .contains("--delay must be one of minimal, low, medium, high, xhigh")
    );
}

fn clear_env_defaults() {
    unsafe {
        for key in [
            "NARRATED_REPLAY_ROOT",
            "NARRATED_REPLAY_REALTIME_DELAY",
            "NARRATED_REPLAY_AVFOUNDATION_INPUT",
            "NARRATED_REPLAY_BATCH_TRANSCRIPTION",
            "NARRATED_REPLAY_CLEANUP",
            "NARRATED_REPLAY_BATCH_MODEL",
            "NARRATED_REPLAY_CLEANUP_MODEL",
            "NARRATED_REPLAY_AUDIO_RETENTION_MODE",
            "NARRATED_REPLAY_AUDIO_RETENTION_PATH",
            "NARRATED_REPLAY_AUDIO_FILTER",
            "NARRATED_REPLAY_CLEANUP_DICTIONARY",
        ] {
            std::env::remove_var(key);
        }
    }
}
