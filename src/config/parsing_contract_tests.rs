use super::*;
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn parses_boolean_environment_aliases_without_cli_side_effects() {
    let _guard = ENV_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let truthy = ["1", "true", "TRUE", "yes", "on"];
    let falsy = ["0", "false", "FALSE", "no", "off"];

    for value in truthy {
        unsafe {
            std::env::set_var("NARRATED_REPLAY_TEST_BOOL", value);
        }
        assert!(parse_bool_env("NARRATED_REPLAY_TEST_BOOL", false));
    }
    for value in falsy {
        unsafe {
            std::env::set_var("NARRATED_REPLAY_TEST_BOOL", value);
        }
        assert!(!parse_bool_env("NARRATED_REPLAY_TEST_BOOL", true));
    }
    unsafe {
        std::env::set_var("NARRATED_REPLAY_TEST_BOOL", "maybe");
    }
    assert!(parse_bool_env("NARRATED_REPLAY_TEST_BOOL", true));
    unsafe {
        std::env::remove_var("NARRATED_REPLAY_TEST_BOOL");
    }
}

#[test]
fn usage_advertises_operational_commands_and_claim_gates() {
    let text = usage();

    for expected in [
        "prepare-coordinated-session",
        "coverage-receipt",
        "check-coverage-policy",
        "parent-operation-receipt",
        "--i-consent-to-openai-postprocessing",
    ] {
        assert!(text.contains(expected), "usage should mention {expected}");
    }
}

#[test]
fn parses_default_aliases_and_slugifies_goals() {
    let args = parse_args_from([
        "nrr",
        "preflight",
        "--input",
        ":default",
        "--audio-filter",
        "voice-default",
    ])
    .unwrap();

    assert_eq!(args.input, crate::audio_input::AUTO_INPUT);
    assert_eq!(args.audio_filter, DEFAULT_AUDIO_FILTER);
    assert_eq!(
        slugify("  Feature Demo: Claude Code + ChatGPT Atlas!!!  "),
        "feature-demo-claude-code-chatgpt-atlas"
    );
}

#[test]
fn rejects_multiline_or_nul_audio_filters_and_malformed_inputs() {
    assert!(parse_args_from(["nrr", "preflight", "--audio-filter", "volume=1\nanull"]).is_err());
    assert!(parse_args_from(["nrr", "preflight", "--audio-filter", "bad\0filter"]).is_err());
    assert!(parse_args_from(["nrr", "preflight", "--input", ":"]).is_err());
    assert!(parse_args_from(["nrr", "preflight", "--input", ":Mic:Extra"]).is_err());
    assert!(parse_args_from(["nrr", "preflight", "--input", ":Mic\nExtra"]).is_err());
}
