#[cfg(test)]
mod prompt_tests {
    use super::*;
    use crate::config::parse_args_from;
    use serde_json::json;
    use std::fs;
    use std::path::PathBuf;
    #[test]
    fn cleanup_text_reads_responses_output_content() {
        let value = json!({
            "output": [
                {
                    "content": [
                        {"text": " First sentence."},
                        {"text": " Second sentence. "}
                    ]
                }
            ]
        });

        assert_eq!(cleanup_text(&value), "First sentence. Second sentence.");
    }

    #[test]
    fn cleanup_text_prefers_explicit_cleaned_text_then_output_text() {
        assert_eq!(
            cleanup_text(&json!({
                "cleanedText": " Cleaned ",
                "output_text": "Ignored"
            })),
            "Cleaned"
        );
        assert_eq!(
            cleanup_text(&json!({
                "output_text": " Output text "
            })),
            "Output text"
        );
    }

    #[test]
    fn cleanup_model_candidates_use_default_fallback_policy() {
        assert_eq!(
            cleanup_model_candidates(""),
            vec![DEFAULT_CLEANUP_MODEL, DEFAULT_CLEANUP_FALLBACK_MODEL]
        );
        assert_eq!(
            cleanup_model_candidates("custom-cleanup-model"),
            vec!["custom-cleanup-model"]
        );
    }

    #[test]
    fn ensure_cleaned_transcript_uses_fixture_without_api_call() {
        let _guard = lock_cleanup_env();
        let root = PathBuf::from("/private/tmp").join(format!(
            "nrr-cleanup-fixture-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let fixture = root.join("fixture.json");
        fs::create_dir_all(&root).unwrap();
        fs::write(
            &fixture,
            r#"{"output_text":"Claude Code opened ChatGPT Atlas."}"#,
        )
        .unwrap();
        unsafe {
            std::env::set_var("NARRATED_REPLAY_CLEANUP_TRANSCRIPT_FIXTURE", &fixture);
        }
        let args = parse_args_from([
            "nrr",
            "packet",
            "--session-dir",
            root.to_str().unwrap(),
            "--i-consent-to-custom-runtime-paths",
        ])
        .unwrap();
        batch_transcribe::write_bound_batch_fixture_for_test(
            &root,
            &args.batch_transcription_model,
            "cloud code opened chat g p t atlas",
        )
        .unwrap();

        let output = ensure_cleaned_transcript(&args, &root).unwrap().unwrap();
        unsafe {
            std::env::remove_var("NARRATED_REPLAY_CLEANUP_TRANSCRIPT_FIXTURE");
        }
        let artifact: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();

        assert_eq!(artifact["source"], "fixture");
        assert_eq!(artifact["cleanedText"], "Claude Code opened ChatGPT Atlas.");
        assert_eq!(artifact["privacy"]["localPrivate"], true);
    }

    #[test]
    fn ensure_cleaned_transcript_returns_existing_regular_artifact() {
        let root = cleanup_test_unique_tmp("nrr-cleanup-existing");
        fs::create_dir_all(&root).unwrap();
        let existing = root.join("cleaned-transcript.json");
        fs::write(&existing, r#"{"cleanedText":"already cleaned"}"#).unwrap();
        let args = parse_args_from([
            "nrr",
            "packet",
            "--session-dir",
            root.to_str().unwrap(),
            "--i-consent-to-custom-runtime-paths",
        ])
        .unwrap();

        let output = ensure_cleaned_transcript(&args, &root).unwrap();

        assert!(output.is_none());
        assert_eq!(
            cleanup_test_read_json(&root.join("cleanup-receipt.json"))["reason"],
            "missing-batch-transcript"
        );
    }

    #[test]
    fn ensure_cleaned_transcript_disables_when_config_or_batch_missing() {
        let disabled_root = cleanup_test_unique_tmp("nrr-cleanup-disabled-config");
        fs::create_dir_all(&disabled_root).unwrap();
        fs::write(
            disabled_root.join("batch-transcript.json"),
            r#"{"transcription":{"text":"raw words"}}"#,
        )
        .unwrap();
        let disabled_args = parse_args_from([
            "nrr",
            "packet",
            "--session-dir",
            disabled_root.to_str().unwrap(),
            "--disable-cleanup",
            "--i-consent-to-custom-runtime-paths",
        ])
        .unwrap();

        assert!(
            ensure_cleaned_transcript(&disabled_args, &disabled_root)
                .unwrap()
                .is_none()
        );
        assert_eq!(
            cleanup_test_read_json(&disabled_root.join("cleanup-receipt.json"))["reason"],
            "disabled-by-config"
        );

        let missing_root = cleanup_test_unique_tmp("nrr-cleanup-missing-batch");
        fs::create_dir_all(&missing_root).unwrap();
        let missing_args = parse_args_from([
            "nrr",
            "packet",
            "--session-dir",
            missing_root.to_str().unwrap(),
            "--i-consent-to-custom-runtime-paths",
        ])
        .unwrap();
        assert!(
            ensure_cleaned_transcript(&missing_args, &missing_root)
                .unwrap()
                .is_none()
        );
        assert_eq!(
            cleanup_test_read_json(&missing_root.join("cleanup-receipt.json"))["reason"],
            "missing-batch-transcript"
        );
    }

    #[test]
    fn ensure_cleaned_transcript_requires_fixture_to_be_regular_file() {
        let _guard = lock_cleanup_env();
        let root = cleanup_test_unique_tmp("nrr-cleanup-bad-fixture");
        fs::create_dir_all(&root).unwrap();
        let missing_fixture = root.join("missing-fixture.json");
        unsafe {
            std::env::set_var(
                "NARRATED_REPLAY_CLEANUP_TRANSCRIPT_FIXTURE",
                &missing_fixture,
            );
        }
        let args = parse_args_from([
            "nrr",
            "packet",
            "--session-dir",
            root.to_str().unwrap(),
            "--i-consent-to-custom-runtime-paths",
        ])
        .unwrap();
        batch_transcribe::write_bound_batch_fixture_for_test(
            &root,
            &args.batch_transcription_model,
            "raw",
        )
        .unwrap();

        let error = ensure_cleaned_transcript(&args, &root).unwrap_err();
        unsafe {
            std::env::remove_var("NARRATED_REPLAY_CLEANUP_TRANSCRIPT_FIXTURE");
        }

        assert!(
            error
                .to_string()
                .contains("cleanup transcript fixture not readable")
        );
    }

    #[test]
    fn ensure_cleaned_transcript_requires_openai_key_for_api_cleanup() {
        let _guard = lock_cleanup_env();
        unsafe {
            std::env::remove_var("NARRATED_REPLAY_CLEANUP_TRANSCRIPT_FIXTURE");
            std::env::remove_var("OPENAI_API_KEY");
        }
        let root = cleanup_test_unique_tmp("nrr-cleanup-missing-key");
        fs::create_dir_all(&root).unwrap();
        let args = parse_args_from([
            "nrr",
            "packet",
            "--session-dir",
            root.to_str().unwrap(),
            "--i-consent-to-openai-postprocessing",
            "--i-consent-to-custom-runtime-paths",
        ])
        .unwrap();
        batch_transcribe::write_bound_batch_fixture_for_test(
            &root,
            &args.batch_transcription_model,
            "raw words",
        )
        .unwrap();

        let error = ensure_cleaned_transcript(&args, &root).unwrap_err();

        assert!(error.to_string().contains("OPENAI_API_KEY is required"));
    }
}
