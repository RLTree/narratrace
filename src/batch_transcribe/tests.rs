#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn prompt_contains_canonical_terms_and_caps_entries() {
        let prompt = build_prompt(None, None);

        assert!(prompt.contains("Claude Code"));
        assert!(prompt.contains("ChatGPT Atlas"));
        assert!(prompt.contains("Record and Replay"));
    }

    #[test]
    fn batch_text_reads_wrapped_transcription() {
        let value = json!({"transcription": {"text": "hello world"}});

        assert_eq!(batch_text(&value), "hello world");
    }

    #[test]
    fn prompt_context_does_not_harvest_private_event_strings() {
        let root = PathBuf::from("/private/tmp/nrr-batch-prompt-privacy-test");
        fs::create_dir_all(&root).unwrap();
        let metadata_path = root.join("metadata.json");
        let events_path = root.join("events.jsonl");
        fs::write(
            &metadata_path,
            r#"{
              "goal": "Review /Users/tree/private/report.docx with tree@example.com",
              "appName": "ChatGPT Atlas",
              "windowTitle": "secret client plan",
              "model": "gpt-realtime-whisper"
            }"#,
        )
        .unwrap();
        fs::write(
            &events_path,
            r#"{"appName":"Finder","windowTitle":"password hunter2","path":"/Users/tree/private/file.txt","text":"raw visible private text"}"#,
        )
        .unwrap();

        let prompt = build_prompt(
            Some(metadata_path.to_str().unwrap()),
            Some(events_path.to_str().unwrap()),
        );

        assert!(prompt.contains("ChatGPT Atlas"));
        assert!(!prompt.contains("gpt-realtime-whisper"));
        assert!(!prompt.contains("Finder"));
        assert!(!prompt.contains("/Users/tree/private"));
        assert!(!prompt.contains("tree@example.com"));
        assert!(!prompt.contains("secret client plan"));
        assert!(!prompt.contains("hunter2"));
        assert!(!prompt.contains("raw visible private text"));
    }

    #[cfg(unix)]
    #[test]
    fn prompt_context_ignores_symlink_inputs() {
        let root = PathBuf::from("/private/tmp/nrr-batch-prompt-symlink-test");
        fs::create_dir_all(&root).unwrap();
        let target_path = root.join("target.json");
        let link_path = root.join("metadata-link.json");
        fs::write(&target_path, r#"{"appName":"Secret Symlink App"}"#).unwrap();
        let _ = fs::remove_file(&link_path);
        std::os::unix::fs::symlink(&target_path, &link_path).unwrap();

        let prompt = build_prompt(Some(link_path.to_str().unwrap()), None);

        assert!(!prompt.contains("Secret Symlink App"));
    }

    #[test]
    fn disabled_batch_transcription_writes_receipt() {
        let _guard = lock_batch_env();
        let root = unique_tmp("nrr-batch-disabled-test");
        fs::create_dir_all(&root).unwrap();
        unsafe {
            std::env::remove_var("NARRATED_REPLAY_BATCH_TRANSCRIPT_FIXTURE");
        }
        let args = crate::config::parse_args_from([
            "nrr",
            "packet",
            "--session-dir",
            root.to_str().unwrap(),
            "--disable-batch-transcription",
            "--i-consent-to-custom-runtime-paths",
        ])
        .unwrap();

        let path = ensure_batch_transcript(&args, &root, None, None).unwrap();
        let receipt: Value = serde_json::from_str(
            &fs::read_to_string(root.join("batch-transcription-receipt.json")).unwrap(),
        )
        .unwrap();

        assert!(path.is_none());
        assert_eq!(receipt["status"], "disabled");
        assert_eq!(receipt["reason"], "disabled-by-config");
    }

    #[test]
    fn missing_retained_audio_disables_batch_without_api_call() {
        let _guard = lock_batch_env();
        let root = unique_tmp("nrr-batch-missing-audio-test");
        fs::create_dir_all(&root).unwrap();
        unsafe {
            std::env::remove_var("NARRATED_REPLAY_BATCH_TRANSCRIPT_FIXTURE");
        }
        let args = crate::config::parse_args_from([
            "nrr",
            "packet",
            "--session-dir",
            root.to_str().unwrap(),
            "--i-consent-to-custom-runtime-paths",
        ])
        .unwrap();

        let path = ensure_batch_transcript(&args, &root, None, None).unwrap();
        let receipt: Value = serde_json::from_str(
            &fs::read_to_string(root.join("batch-transcription-receipt.json")).unwrap(),
        )
        .unwrap();

        assert!(path.is_none());
        assert_eq!(receipt["status"], "disabled");
        assert!(
            receipt["reason"]
                .as_str()
                .unwrap()
                .contains("missing-audio-retention")
        );
    }

    #[test]
    fn fixture_batch_transcript_writes_private_artifact_without_api_call() {
        let _guard = lock_batch_env();
        let root = unique_tmp("nrr-batch-fixture-test");
        let fixture = root.join("fixture.json");
        fs::create_dir_all(&root).unwrap();
        fs::write(&fixture, r#"{"text":"Claude Code narration"}"#).unwrap();
        unsafe {
            std::env::set_var("NARRATED_REPLAY_BATCH_TRANSCRIPT_FIXTURE", &fixture);
        }
        let args = crate::config::parse_args_from([
            "nrr",
            "packet",
            "--session-dir",
            root.to_str().unwrap(),
            "--i-consent-to-custom-runtime-paths",
        ])
        .unwrap();

        let output = ensure_batch_transcript(&args, &root, None, None)
            .unwrap()
            .unwrap();
        unsafe {
            std::env::remove_var("NARRATED_REPLAY_BATCH_TRANSCRIPT_FIXTURE");
        }
        let artifact: Value = serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();

        assert_eq!(artifact["source"], "fixture");
        assert_eq!(artifact["transcription"]["text"], "Claude Code narration");
        assert_eq!(
            artifact["privacy"]["copyIntoGeneratedPacketsByDefault"],
            false
        );
    }

    #[test]
    fn write_batch_artifact_records_private_boundary() {
        let root = PathBuf::from("/private/tmp/nrr-batch-artifact-test");
        fs::create_dir_all(&root).unwrap();
        let output = root.join("batch-transcript.json");

        let prompt = build_prompt(None, None);
        let binding =
            BatchBinding::for_fixture(&root, "gpt-4o-transcribe", &prompt, "fixture-digest", 1)
                .unwrap();
        write_batch_artifact(
            &root,
            &output,
            "gpt-4o-transcribe",
            &prompt,
            "fixture",
            json!({"text":"Claude Code narration"}),
            &binding,
        )
        .unwrap();

        let artifact: Value = serde_json::from_str(&fs::read_to_string(&output).unwrap()).unwrap();
        let receipt: Value = serde_json::from_str(
            &fs::read_to_string(root.join("batch-transcription-receipt.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(artifact["transcription"]["text"], "Claude Code narration");
        assert_eq!(artifact["privacy"]["rawAudioCopied"], false);
        assert_eq!(receipt["status"], "completed");
    }

    #[test]
    fn audio_path_requires_regular_retained_audio_file() {
        let root = PathBuf::from("/private/tmp/nrr-batch-audio-path-test");
        fs::create_dir_all(&root).unwrap();
        let wav = root.join("retained-audio.wav");
        fs::write(&wav, b"RIFF....WAVE").unwrap();
        fs::write(
            root.join("audio-retention.json"),
            serde_json::to_string(&json!({"audioPath": wav})).unwrap(),
        )
        .unwrap();

        assert_eq!(audio_path(&root).unwrap(), wav);
    }

    #[test]
    fn combine_chunk_transcripts_preserves_chunk_metadata() {
        let combined =
            combine_chunk_transcripts(&[json!({"text":"alpha"}), json!({"text":"bravo"})], 2);

        assert_eq!(combined["text"], "alpha\nbravo");
        assert_eq!(combined["chunked"], true);
        assert_eq!(combined["chunkCount"], 2);
    }

    #[test]
    fn chunk_wav_bytes_rewrites_header_for_private_chunk() {
        let source_payload = vec![1_u8, 2, 3, 4, 5, 6];

        let chunk = chunk_wav_bytes(&source_payload, 2, 6).unwrap();

        assert_eq!(&chunk[0..4], b"RIFF");
        assert_eq!(&chunk[8..12], b"WAVE");
        assert_eq!(&chunk[36..40], b"data");
        assert_eq!(&chunk[44..], &[3_u8, 4, 5, 6]);
    }

    fn unique_tmp(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        PathBuf::from("/private/tmp").join(format!("{prefix}-{nanos}"))
    }
}
