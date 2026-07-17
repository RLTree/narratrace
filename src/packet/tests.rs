#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::parse_args_from;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[cfg(unix)]
    #[test]
    fn thought_process_ignores_symlinked_raw_transcript_file() {
        let root = unique_tmp("nrr-packet-symlink");
        let session_dir = root.join("session");
        fs::create_dir_all(&session_dir).unwrap();
        let outside = root.join("outside.jsonl");
        fs::write(
            &outside,
            r#"{"kind":"completed","audioOffsetMs":10,"text":"private path /Users/tree/secret.txt"}"#,
        )
        .unwrap();
        std::os::unix::fs::symlink(&outside, session_dir.join("transcript.timeline.jsonl"))
            .unwrap();

        update_thought_process(&session_dir).unwrap();

        let thought = fs::read_to_string(session_dir.join("thought-process.md")).unwrap();
        assert!(!thought.contains("secret"));
        assert!(!thought.contains("/Users/tree/secret.txt"));
        assert!(thought.contains("Transcript segments: 0"));
    }

    #[test]
    fn thought_process_does_not_embed_raw_transcript_text() {
        let root = unique_tmp("nrr-packet-private-text");
        let session_dir = root.join("session");
        fs::create_dir_all(&session_dir).unwrap();
        fs::write(
            session_dir.join("transcript.timeline.jsonl"),
            r#"{"kind":"completed","audioOffsetMs":10,"text":"private path /Users/tree/secret.txt"}"#,
        )
        .unwrap();

        update_thought_process(&session_dir).unwrap();

        let thought = fs::read_to_string(session_dir.join("thought-process.md")).unwrap();
        assert!(!thought.contains("secret"));
        assert!(!thought.contains("/Users/tree/secret.txt"));
        assert!(thought.contains("Transcript segments: 1"));
        assert!(thought.contains("Do not copy raw transcript lines"));
    }

    #[test]
    fn packet_fixture_builds_cleaned_final_alignment_without_api_calls() {
        let root = unique_tmp("nrr-packet-quality-pipeline");
        let session_dir = root.join("session");
        fs::create_dir_all(&session_dir).unwrap();
        let metadata_path = root.join("metadata.json");
        let events_path = root.join("events.jsonl");
        fs::write(
            session_dir.join("manifest.json"),
            r#"{"schema":"narrated-record-replay.session.v1","goal":"fixture quality pipeline","postStopQualityPipeline":{"batchTranscriptionEnabledByDefault":true}}"#,
        )
        .unwrap();
        fs::write(
            session_dir.join("capture-clock.json"),
            r#"{"schema":"narrated-record-replay.capture-clock.v1","audioStartedAtUnixMs":1700000000000,"delay":"high"}"#,
        )
        .unwrap();
        fs::write(
            session_dir.join("transcript.timeline.jsonl"),
            concat!(
                "{\"kind\":\"completed\",\"audioOffsetMs\":1000,\"monotonicOffsetMs\":1000,\"text\":\"cloud code\"}\n",
                "{\"kind\":\"completed\",\"audioOffsetMs\":2500,\"monotonicOffsetMs\":2500,\"text\":\"chat g p t atlas web browser\"}\n"
            ),
        )
        .unwrap();
        fs::write(
            &metadata_path,
            r#"{"startedAt":"2023-11-14T22:13:20.000Z"}"#,
        )
        .unwrap();
        fs::write(
            &events_path,
            r#"{"id":1,"kind":"window","timestamp":"2023-11-14T22:13:21.000Z","app":{"name":"ChatGPT Atlas"},"window":{"title":"Codex"}}"#,
        )
        .unwrap();
        let args = parse_args_from([
            "nrr",
            "packet",
            "--session-dir",
            session_dir.to_str().unwrap(),
            "--recording-metadata",
            metadata_path.to_str().unwrap(),
            "--recording-events",
            events_path.to_str().unwrap(),
            "--i-consent-to-custom-runtime-paths",
        ])
        .unwrap();
        let _batch_env_guard = crate::batch_transcribe::lock_batch_env();
        let batch_text = "Claude Code. ChatGPT Atlas web browser.";
        let batch_fixture = session_dir.join("batch-fixture.json");
        fs::write(
            &batch_fixture,
            serde_json::to_vec(&serde_json::json!({"text": batch_text})).unwrap(),
        )
        .unwrap();
        crate::transcript_cleanup::write_bound_cleanup_fixture_for_test(
            &args,
            &session_dir,
            batch_text,
            "Claude Code ChatGPT Atlas web browser",
        )
        .unwrap();
        unsafe {
            std::env::set_var("NARRATED_REPLAY_BATCH_TRANSCRIPT_FIXTURE", &batch_fixture);
        }

        let result = make_packet(&args);
        unsafe {
            std::env::remove_var("NARRATED_REPLAY_BATCH_TRANSCRIPT_FIXTURE");
        }
        result.unwrap();

        let final_alignment =
            fs::read_to_string(session_dir.join("final-transcript-alignment.json")).unwrap();
        assert!(
            final_alignment.contains("\"wordAuthority\": \"verified-cleaned-batch-transcript\"")
        );
        assert!(final_alignment.contains("Claude Code"));
        let evidence =
            fs::read_to_string(session_dir.join("evidence-boundary-report.json")).unwrap();
        assert!(evidence.contains("batchRaw"));
        assert!(evidence.contains("alignedFinal"));
        let temporal = fs::read_to_string(session_dir.join("temporal-context.json")).unwrap();
        assert!(temporal.contains("aligned-cleaned-batch-text-with-realtime-window"));
    }

    #[test]
    fn packet_openai_postprocessing_requires_explicit_consent() {
        let root = unique_tmp("nrr-packet-openai-consent");
        let session_dir = root.join("session");
        fs::create_dir_all(&session_dir).unwrap();
        fs::write(session_dir.join("retained-audio.wav"), b"RIFFprivate audio").unwrap();
        let args = parse_args_from([
            "nrr",
            "packet",
            "--session-dir",
            session_dir.to_str().unwrap(),
            "--i-consent-to-custom-runtime-paths",
        ])
        .unwrap();

        let error = make_packet(&args).unwrap_err().to_string();

        assert!(error.contains("--i-consent-to-openai-postprocessing"));
    }

    #[test]
    fn packet_openai_postprocessing_requires_explicit_consent_for_custom_retained_audio() {
        let root = unique_tmp("nrr-packet-openai-custom-consent");
        let session_dir = root.join("session");
        let custom_audio = root.join("custom-retained-audio.wav");
        fs::create_dir_all(&session_dir).unwrap();
        fs::write(&custom_audio, b"RIFFprivate custom audio").unwrap();
        fs::write(
            session_dir.join("audio-retention.json"),
            serde_json::to_string_pretty(&serde_json::json!({
                "schema": "narrated-record-replay.audio-retention.v1",
                "mode": "private-wav",
                "audioPath": custom_audio.display().to_string(),
                "privacy": {
                    "localPrivate": true,
                    "containsRawMicrophoneAudio": true
                }
            }))
            .unwrap(),
        )
        .unwrap();
        let args = parse_args_from([
            "nrr",
            "packet",
            "--session-dir",
            session_dir.to_str().unwrap(),
            "--i-consent-to-custom-runtime-paths",
        ])
        .unwrap();

        let error = make_packet(&args).unwrap_err().to_string();

        assert!(error.contains("--i-consent-to-openai-postprocessing"));
    }

    #[test]
    fn packet_openai_postprocessing_allows_local_only_mode_without_consent() {
        let root = unique_tmp("nrr-packet-local-only-consent");
        let session_dir = root.join("session");
        fs::create_dir_all(&session_dir).unwrap();
        let args = parse_args_from([
            "nrr",
            "packet",
            "--session-dir",
            session_dir.to_str().unwrap(),
            "--disable-batch-transcription",
            "--disable-cleanup",
            "--i-consent-to-custom-runtime-paths",
        ])
        .unwrap();

        assert!(!openai_postprocessing_would_send_private_material(
            &args,
            &session_dir,
            false,
            false
        ));
    }

    #[cfg(unix)]
    fn unique_tmp(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        PathBuf::from("/private/tmp").join(format!("{prefix}-{nanos}"))
    }
}
