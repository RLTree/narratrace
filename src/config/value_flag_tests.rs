use super::*;

#[test]
fn parses_all_path_and_value_flags_through_validation() {
    let args = parse_args_from([
        "nrr",
        "packet",
        "--root",
        "/private/tmp/nrr-test/root",
        "--skill-dir",
        "/private/tmp/nrr-test/skill",
        "--session-dir",
        "/private/tmp/nrr-test/session",
        "--recording-metadata",
        "/private/tmp/nrr-test/recording.json",
        "--recording-events",
        "/private/tmp/nrr-test/events.jsonl",
        "--audio-retention-path",
        "/private/tmp/nrr-test/session/retained.wav",
        "--audio-filter",
        "highpass=f=90,lowpass=f=8000,volume=6dB,alimiter=limit=0.90",
        "--record-replay-status",
        "recording",
        "--max-seconds",
        "42",
        "--i-consent-to-custom-runtime-paths",
        "--i-consent-to-custom-audio-filter",
    ])
    .unwrap();

    assert_eq!(
        args.root,
        std::path::PathBuf::from("/private/tmp/nrr-test/root")
    );
    assert_eq!(args.record_replay_status.as_deref(), Some("recording"));
    assert_eq!(args.max_seconds, Some(42));
    assert!(
        args.recording_metadata
            .as_deref()
            .unwrap()
            .ends_with(".json")
    );
    assert!(
        args.recording_events
            .as_deref()
            .unwrap()
            .ends_with(".jsonl")
    );
    assert!(args.audio_retention_path.unwrap().ends_with("retained.wav"));
}

#[test]
fn missing_required_values_are_rejected_for_every_value_flag() {
    for flag in [
        "--root",
        "--skill-dir",
        "--session-dir",
        "--baseline-delay-evaluation",
        "--candidate-delay-evaluation",
        "--coverage-json",
        "--coverage-receipt",
        "--delay",
        "--input",
        "--record-replay-status",
        "--batch-transcription-model",
        "--cleanup-model",
        "--audio-retention-mode",
        "--audio-retention-path",
        "--audio-filter",
        "--cleanup-dictionary-source",
        "--replay-voice-style",
        "--replay-voice-pace",
        "--replay-voice-emphasis",
        "--receipt-run-id",
        "--receipt-generated-at",
    ] {
        let result = parse_args_from(["nrr", "preflight", flag]);
        assert!(result.is_err(), "{flag} should require a value");
    }
}
