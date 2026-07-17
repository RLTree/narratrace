use super::*;

#[test]
fn default_realtime_delay_is_high() {
    let args = parse_args_from(["nrr", "preflight"]).unwrap();

    assert_eq!(args.delay, "high");
}

#[test]
fn default_max_seconds_is_thirty_minutes() {
    let args = parse_args_from(["nrr", "preflight"]).unwrap();

    assert_eq!(args.max_seconds, Some(DEFAULT_MAX_SECONDS));
}

#[test]
fn parses_batch_cleanup_and_retention_flags() {
    let args = parse_args_from([
        "nrr",
        "packet",
        "--session-dir",
        "/private/tmp/narrated-record-replay/nrr-config-test",
        "--delay",
        "xhigh",
        "--batch-transcription-model",
        "gpt-4o-mini-transcribe",
        "--cleanup-model",
        "gpt-5.4-mini",
        "--i-consent-to-openai-postprocessing",
        "--disable-cleanup",
        "--audio-retention-mode",
        "disabled",
    ])
    .unwrap();

    assert_eq!(args.delay, "xhigh");
    assert_eq!(args.batch_transcription_model, "gpt-4o-mini-transcribe");
    assert_eq!(args.cleanup_model, "gpt-5.4-mini");
    assert!(args.openai_postprocessing_consent);
    assert!(!args.cleanup_enabled);
    assert_eq!(args.audio_retention_mode, "disabled");
}

#[test]
fn default_audio_filter_limits_clipping() {
    let args = parse_args_from(["nrr", "preflight"]).unwrap();

    assert_eq!(
        args.audio_filter,
        "highpass=f=80,lowpass=f=9000,volume=9dB,alimiter=limit=0.95"
    );
}

#[test]
fn rejects_audio_filter_override_without_explicit_consent() {
    let result = parse_args_from([
        "nrr",
        "capture",
        "--session-dir",
        "/private/tmp/narrated-record-replay/nrr-config-test",
        "--audio-filter",
        "highpass=f=100,volume=6dB,alimiter=limit=0.90",
    ]);

    assert!(result.is_err());
}

#[test]
fn parses_audio_filter_override_with_explicit_consent() {
    let args = parse_args_from([
        "nrr",
        "capture",
        "--session-dir",
        "/private/tmp/narrated-record-replay/nrr-config-test",
        "--audio-filter",
        "highpass=f=100,volume=6dB,alimiter=limit=0.90",
        "--i-consent-to-custom-audio-filter",
    ])
    .unwrap();

    assert_eq!(
        args.audio_filter,
        "highpass=f=100,volume=6dB,alimiter=limit=0.90"
    );
}

#[test]
fn parses_named_avfoundation_input_override() {
    let args = parse_args_from([
        "nrr",
        "capture",
        "--session-dir",
        "/private/tmp/narrated-record-replay/nrr-config-test",
        "--input",
        ":MacBook Pro Microphone",
    ])
    .unwrap();

    assert_eq!(args.input, ":MacBook Pro Microphone");
}

#[test]
fn rejects_custom_root_without_explicit_consent() {
    let result = parse_args_from(["nrr", "preflight", "--root", "/private/tmp/other-nrr-root"]);

    assert!(result.is_err());
}

#[test]
fn accepts_custom_root_with_explicit_consent() {
    let args = parse_args_from([
        "nrr",
        "preflight",
        "--root",
        "/private/tmp/other-nrr-root",
        "--i-consent-to-custom-runtime-paths",
    ])
    .unwrap();

    assert_eq!(
        args.root,
        std::path::PathBuf::from("/private/tmp/other-nrr-root")
    );
}

#[test]
fn rejects_custom_session_dir_without_explicit_consent() {
    let result = parse_args_from([
        "nrr",
        "packet",
        "--session-dir",
        "/private/tmp/not-narrated/session",
    ]);

    assert!(result.is_err());
}

#[test]
fn rejects_custom_audio_retention_path_without_explicit_consent() {
    let result = parse_args_from([
        "nrr",
        "capture",
        "--session-dir",
        "/private/tmp/narrated-record-replay/nrr-config-test",
        "--audio-retention-path",
        "/Users/terrynoblin/retained-audio.wav",
    ]);

    assert!(result.is_err());
}
