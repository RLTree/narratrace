#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::parse_args_from;

    #[test]
    fn cleanup_prompt_keeps_instructions_out_of_untrusted_data() {
        let prompt = cleanup_prompt(
            &["Claude Code".to_string(), "ChatGPT Atlas".to_string()],
            "cloud code in chat g p t atlas",
        );

        assert!(!prompt.contains(CLEANUP_SEED));
        assert!(prompt.contains("Claude Code"));
        assert!(prompt.contains("ChatGPT Atlas"));
        assert!(prompt.contains("untrusted-transcript-data"));
    }

    #[test]
    fn cleanup_prompt_preserves_spoken_digit_sequence_policy() {
        let prompt = cleanup_prompt(&[], "the sequence is 942, not 942 and not 400");

        assert!(CLEANUP_SEED.contains("digit-by-digit sequences"));
        assert!(CLEANUP_SEED.contains("nine, four, two"));
        assert!(prompt.contains("942"));
    }

    #[test]
    fn cleanup_prompt_preserves_disfluencies_and_self_corrections() {
        let prompt = cleanup_prompt(&[], "the blue parrot was, was certain; forty, no, fourteen");

        assert!(CLEANUP_SEED.contains("intentional repetitions"));
        assert!(prompt.contains("was, was"));
        assert!(prompt.contains("forty, no, fourteen"));
    }

    #[test]
    fn dictionary_is_capped_to_one_hundred_entries() {
        let root = PathBuf::from("/private/tmp/nrr-cleanup-dict-test");
        let dictionary_path = root.join("dict.txt");
        fs::create_dir_all(&root).unwrap();
        let text = (0..150)
            .map(|index| format!("Term {index}"))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(&dictionary_path, text).unwrap();
        let args = parse_args_from([
            "nrr",
            "packet",
            "--session-dir",
            "/private/tmp/nrr-cleanup-dict-test",
            "--cleanup-dictionary-source",
            dictionary_path.to_str().unwrap(),
            "--i-consent-to-custom-runtime-paths",
        ])
        .unwrap();

        let dictionary = build_dictionary(&args, &root);

        assert_eq!(dictionary.len(), 11);
        assert!(!dictionary.iter().any(|entry| entry.starts_with("Term ")));
    }

    #[test]
    fn dictionary_rejects_private_manifest_status_strings() {
        let root = PathBuf::from("/private/tmp/nrr-cleanup-dict-privacy-test");
        fs::create_dir_all(&root).unwrap();
        fs::write(
            root.join("manifest.json"),
            r#"{
              "goal": "Review /Users/tree/private/report.docx for tree@example.com",
              "appName": "ChatGPT Atlas",
              "windowTitle": "private client sentence",
              "model": "gpt-realtime-whisper"
            }"#,
        )
        .unwrap();
        fs::write(
            root.join("status.json"),
            r#"{
              "state": "stopped",
              "secret": "hunter2",
              "appName": "Finder",
              "path": "/Users/tree/private/file.txt"
            }"#,
        )
        .unwrap();
        let args = parse_args_from([
            "nrr",
            "packet",
            "--session-dir",
            root.to_str().unwrap(),
            "--i-consent-to-custom-runtime-paths",
        ])
        .unwrap();

        let dictionary = build_dictionary(&args, &root);
        let joined = dictionary.join("\n");

        assert!(!joined.contains("ChatGPT Atlas"));
        assert!(!joined.contains("gpt-realtime-whisper"));
        assert!(!joined.contains("Finder"));
        assert!(!joined.contains("/Users/tree/private"));
        assert!(!joined.contains("tree@example.com"));
        assert!(!joined.contains("private client sentence"));
        assert!(!joined.contains("hunter2"));
    }

    #[test]
    fn dictionary_source_is_sanitized_before_cleanup_prompt() {
        let root = PathBuf::from("/private/tmp/nrr-cleanup-dict-source-privacy-test");
        let dictionary_path = root.join("dict.txt");
        fs::create_dir_all(&root).unwrap();
        fs::write(
            &dictionary_path,
            "Claude Code\n/Users/tree/private/project\noperator@example.com\npassword hunter2\nChatGPT Atlas\n",
        )
        .unwrap();
        let args = parse_args_from([
            "nrr",
            "packet",
            "--session-dir",
            root.to_str().unwrap(),
            "--cleanup-dictionary-source",
            dictionary_path.to_str().unwrap(),
            "--i-consent-to-custom-runtime-paths",
        ])
        .unwrap();

        let dictionary = build_dictionary(&args, &root);
        let prompt = cleanup_prompt(&dictionary, "raw transcript");

        assert!(prompt.contains("Claude Code"));
        assert!(!prompt.contains("ChatGPT Atlas"));
        assert!(!prompt.contains("/Users/tree/private"));
        assert!(!prompt.contains("operator@example.com"));
        assert!(!prompt.contains("hunter2"));
    }

    #[test]
    fn disabled_cleanup_writes_receipt() {
        let root = PathBuf::from("/private/tmp/nrr-cleanup-disabled-test");
        fs::create_dir_all(&root).unwrap();
        let args = parse_args_from([
            "nrr",
            "packet",
            "--session-dir",
            root.to_str().unwrap(),
            "--disable-cleanup",
            "--i-consent-to-custom-runtime-paths",
        ])
        .unwrap();

        let path = ensure_cleaned_transcript(&args, &root).unwrap();
        let receipt: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(root.join("cleanup-receipt.json")).unwrap())
                .unwrap();

        assert!(path.is_none());
        assert_eq!(receipt["status"], "disabled");
        assert_eq!(receipt["reason"], "disabled-by-config");
    }

    #[test]
    fn missing_batch_transcript_disables_cleanup_without_api_call() {
        let root = PathBuf::from("/private/tmp/nrr-cleanup-missing-batch-test");
        fs::create_dir_all(&root).unwrap();
        let args = parse_args_from([
            "nrr",
            "packet",
            "--session-dir",
            root.to_str().unwrap(),
            "--i-consent-to-custom-runtime-paths",
        ])
        .unwrap();

        let path = ensure_cleaned_transcript(&args, &root).unwrap();
        let receipt: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(root.join("cleanup-receipt.json")).unwrap())
                .unwrap();

        assert!(path.is_none());
        assert_eq!(receipt["status"], "disabled");
        assert_eq!(receipt["reason"], "missing-batch-transcript");
    }

    #[test]
    fn cleanup_artifact_records_conservative_private_boundary() {
        let root = PathBuf::from("/private/tmp/nrr-cleanup-artifact-test");
        fs::create_dir_all(&root).unwrap();
        let output = root.join("cleaned-transcript.json");

        let batch = batch_transcribe::VerifiedBatchTranscript {
            text: "cloud code".to_string(),
            artifact_sha256: "batch-artifact".to_string(),
            receipt_sha256: "batch-receipt".to_string(),
            session_identity: "session".to_string(),
        };
        let dictionary = vec!["Claude Code".to_string()];
        let input = cleanup_model_input(&dictionary, &batch.text).unwrap();
        let validation = validate_cleanup_output(&batch.text, "Claude Code", &dictionary);
        let binding = CleanupBinding {
            session_identity: "session".to_string(),
            requested_model: "gpt-5.4-mini".to_string(),
            used_model: "gpt-5.4-mini".to_string(),
            config_sha256: "config".to_string(),
            batch_artifact_sha256: "batch-artifact".to_string(),
            batch_receipt_sha256: "batch-receipt".to_string(),
            dictionary_sha256: dictionary_sha256(&dictionary),
            source: "fixture".to_string(),
            response_sha256: "response".to_string(),
            consent_scope: CLEANUP_FIXTURE_SCOPE.to_string(),
            validation_status: validation.status.clone(),
        };
        write_cleanup_artifact(
            &root,
            &output,
            &batch,
            &dictionary,
            &input,
            "Claude Code".to_string(),
            json!({"output_text":"Claude Code"}),
            None,
            &binding,
            &validation,
        )
        .unwrap();

        let artifact: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&output).unwrap()).unwrap();
        let receipt: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(root.join("cleanup-receipt.json")).unwrap())
                .unwrap();
        assert_eq!(artifact["cleanedText"], "Claude Code");
        assert_eq!(artifact["privacy"]["conservativeCleanupOnly"], true);
        assert_eq!(artifact["store"], false);
        assert_eq!(receipt["status"], "completed");
    }

    #[test]
    fn cleanup_model_fallback_note_names_effective_model() {
        assert!(fallback_note(DEFAULT_CLEANUP_MODEL, DEFAULT_CLEANUP_MODEL).is_none());
        let note = fallback_note("gpt-5.4-mini", "gpt-5-mini").unwrap();

        assert!(note.contains("gpt-5.4-mini"));
        assert!(note.contains("gpt-5-mini"));
    }
}
