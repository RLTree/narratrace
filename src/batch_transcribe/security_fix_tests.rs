#[cfg(test)]
mod security_fix_tests {
    use super::*;
    use crate::config::parse_args_from;

    #[test]
    fn stale_unbound_batch_artifact_is_not_reused() {
        let root = test_root("stale-batch");
        fs::create_dir_all(&root).unwrap();
        fs::write(
            root.join("batch-transcript.json"),
            r#"{"schema":"narrated-record-replay.batch-transcript.v1","text":"stale"}"#,
        )
        .unwrap();
        let binding = BatchBinding::for_fixture(
            &root,
            "gpt-4o-transcribe",
            &build_prompt(None, None),
            "source",
            1,
        )
        .unwrap();
        assert!(verified_cached_batch(&root, &binding).unwrap().is_none());
    }

    #[test]
    fn retained_audio_handle_survives_path_replacement() {
        let root = test_root("audio-handle");
        fs::create_dir_all(&root).unwrap();
        let audio = root.join("retained-audio.wav");
        fs::write(&audio, b"original-audio-bytes").unwrap();
        fs::write(
            root.join("audio-retention.json"),
            serde_json::to_vec(&json!({"audioPath": audio})).unwrap(),
        )
        .unwrap();
        let mut opened = open_current_audio(&root).unwrap();
        fs::rename(&audio, root.join("replaced-away.wav")).unwrap();
        fs::write(&audio, b"replacement-bytes").unwrap();
        let mut retained = String::new();
        opened.file.read_to_string(&mut retained).unwrap();
        assert_eq!(retained, "original-audio-bytes");
        assert_ne!(retained.as_bytes(), fs::read(&audio).unwrap());
    }

    #[test]
    fn fresh_bound_fixture_is_reusable_only_with_matching_binding() {
        let root = test_root("bound-batch");
        fs::create_dir_all(&root).unwrap();
        write_bound_batch_fixture_for_test(&root, "gpt-4o-transcribe", "current words").unwrap();
        let receipt: Value = serde_json::from_slice(
            &fs::read(root.join("batch-transcription-receipt.json")).unwrap(),
        )
        .unwrap();
        let binding = BatchBinding::for_fixture(
            &root,
            "gpt-4o-transcribe",
            &build_prompt(None, None),
            receipt["sourceSha256"].as_str().unwrap(),
            receipt["sourceBytes"].as_u64().unwrap(),
        )
        .unwrap();
        assert_eq!(
            verified_cached_batch(&root, &binding)
                .unwrap()
                .unwrap()
                .text,
            "current words"
        );
    }

    fn test_root(label: &str) -> PathBuf {
        let args = parse_args_from(["nrr", "packet"]).unwrap();
        drop(args);
        PathBuf::from("/private/tmp").join(format!(
            "nrr-{label}-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }
}
