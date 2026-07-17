#[cfg(test)]
mod lifecycle_tests {
    use super::*;
    use crate::config::parse_args_from;
    use serde_json::{Value, json};
    use std::fs;
    use std::io::Write;
    use std::net::TcpListener;
    use std::path::{Path, PathBuf};
    use std::thread;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn ensure_batch_transcript_returns_existing_regular_artifact() {
        let _guard = lock_batch_env();
        let root = unique_tmp("nrr-batch-existing");
        fs::create_dir_all(&root).unwrap();
        let existing = root.join("batch-transcript.json");
        fs::write(&existing, r#"{"transcription":{"text":"already done"}}"#).unwrap();
        let args = args_for(&root, []);

        let output = ensure_batch_transcript(&args, &root, None, None).unwrap();

        assert!(output.is_none());
        assert!(
            read_json(&root.join("batch-transcription-receipt.json"))["reason"]
                .as_str()
                .unwrap()
                .starts_with("missing-audio-retention")
        );
    }

    #[test]
    fn ensure_batch_transcript_records_disabled_config_and_missing_audio() {
        let disabled_root = unique_tmp("nrr-batch-disabled");
        fs::create_dir_all(&disabled_root).unwrap();
        let disabled_args = args_for(&disabled_root, ["--disable-batch-transcription"]);

        assert!(
            ensure_batch_transcript(&disabled_args, &disabled_root, None, None)
                .unwrap()
                .is_none()
        );
        assert_eq!(
            read_json(&disabled_root.join("batch-transcription-receipt.json"))["reason"],
            "disabled-by-config"
        );

        let missing_root = unique_tmp("nrr-batch-missing-audio");
        fs::create_dir_all(&missing_root).unwrap();
        let missing_args = args_for(&missing_root, []);
        assert!(
            ensure_batch_transcript(&missing_args, &missing_root, None, None)
                .unwrap()
                .is_none()
        );
        assert!(
            read_json(&missing_root.join("batch-transcription-receipt.json"))["reason"]
                .as_str()
                .unwrap()
                .contains("missing-audio-retention")
        );
    }

    #[test]
    fn ensure_batch_transcript_requires_fixture_to_be_regular_file() {
        let _guard = lock_batch_env();
        let root = unique_tmp("nrr-batch-bad-fixture");
        fs::create_dir_all(&root).unwrap();
        unsafe {
            std::env::set_var(
                "NARRATED_REPLAY_BATCH_TRANSCRIPT_FIXTURE",
                root.join("missing.json"),
            );
        }
        let args = args_for(&root, []);

        let error = ensure_batch_transcript(&args, &root, None, None).unwrap_err();
        unsafe {
            std::env::remove_var("NARRATED_REPLAY_BATCH_TRANSCRIPT_FIXTURE");
        }

        assert!(
            error
                .to_string()
                .contains("batch transcript fixture not readable")
        );
    }

    #[test]
    fn ensure_batch_transcript_requires_openai_key_after_audio_is_available() {
        let _guard = lock_batch_env();
        unsafe {
            std::env::remove_var("NARRATED_REPLAY_BATCH_TRANSCRIPT_FIXTURE");
            std::env::remove_var("OPENAI_API_KEY");
        }
        let root = unique_tmp("nrr-batch-missing-key");
        fs::create_dir_all(&root).unwrap();
        let audio = root.join("retained-audio.wav");
        fs::write(&audio, wav_bytes(64)).unwrap();
        fs::write(
            root.join("audio-retention.json"),
            serde_json::to_string(&json!({"audioPath": audio})).unwrap(),
        )
        .unwrap();
        let args = args_for(&root, ["--i-consent-to-openai-postprocessing"]);

        let error = ensure_batch_transcript(&args, &root, None, None).unwrap_err();

        assert!(error.to_string().contains("OPENAI_API_KEY is required"));
    }

    #[test]
    fn ensure_batch_transcript_uses_local_api_and_writes_private_artifact() {
        let _guard = lock_batch_env();
        clear_batch_env();
        let root = unique_tmp("nrr-batch-local-api");
        fs::create_dir_all(&root).unwrap();
        let audio = root.join("retained-audio.wav");
        fs::write(&audio, wav_bytes(64)).unwrap();
        fs::write(
            root.join("audio-retention.json"),
            serde_json::to_string(&json!({"audioPath": audio})).unwrap(),
        )
        .unwrap();
        let (url, handle) = local_response(200, r#"{"text":"Batch transcript words."}"#);
        unsafe {
            std::env::set_var("NARRATED_REPLAY_BATCH_API_URL", &url);
            std::env::set_var("OPENAI_API_KEY", "test-key");
        }
        let args = args_for(&root, ["--i-consent-to-openai-postprocessing"]);

        let output = ensure_batch_transcript(&args, &root, None, None)
            .unwrap()
            .unwrap();
        clear_batch_env();
        handle.join().unwrap();
        let artifact = read_json(&output);

        assert_eq!(artifact["source"], "openai-audio-transcriptions");
        assert_eq!(artifact["transcription"]["text"], "Batch transcript words.");
        assert_eq!(artifact["privacy"]["localPrivate"], true);
    }

    #[test]
    fn transcription_api_non_success_is_reported() {
        let _guard = lock_batch_env();
        clear_batch_env();
        let root = unique_tmp("nrr-batch-api-error");
        fs::create_dir_all(&root).unwrap();
        let audio = root.join("retained-audio.wav");
        fs::write(&audio, wav_bytes(64)).unwrap();
        let (url, handle) = local_response(503, r#"{"error":"try later"}"#);
        let client = reqwest::blocking::Client::builder().build().unwrap();
        unsafe {
            std::env::set_var("NARRATED_REPLAY_BATCH_API_URL", &url);
        }

        let error =
            call_transcription_api(&client, "test-key", &audio, "gpt-4o-transcribe", "prompt")
                .unwrap_err()
                .to_string();
        clear_batch_env();
        handle.join().unwrap();

        assert!(error.contains("batch transcription failed with 503"));
        assert!(!error.contains("try later"));
    }

    #[test]
    fn audio_path_reads_regular_manifest_target() {
        let root = unique_tmp("nrr-batch-audio-path");
        fs::create_dir_all(&root).unwrap();
        let audio = root.join("retained-audio.wav");
        fs::write(&audio, wav_bytes(2)).unwrap();
        fs::write(
            root.join("audio-retention.json"),
            serde_json::to_string(&json!({"audioPath": audio})).unwrap(),
        )
        .unwrap();

        assert_eq!(audio_path(&root).unwrap(), audio);
    }

    fn args_for<const N: usize>(root: &Path, extra: [&str; N]) -> crate::config::Args {
        let mut argv = vec![
            "nrr",
            "packet",
            "--session-dir",
            root.to_str().unwrap(),
            "--i-consent-to-custom-runtime-paths",
        ];
        argv.extend(extra);
        parse_args_from(argv).unwrap()
    }

    fn read_json(path: &Path) -> Value {
        serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
    }

    fn local_response(status: u16, body: &'static str) -> (String, thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            drain_test_http_request(&mut stream);
            let response = format!(
                "HTTP/1.1 {status} Test\r\nContent-Length: {}\r\n\r\n{body}",
                body.len()
            );
            stream.write_all(response.as_bytes()).unwrap();
        });
        (url, handle)
    }

    fn clear_batch_env() {
        unsafe {
            std::env::remove_var("NARRATED_REPLAY_BATCH_API_URL");
            std::env::remove_var("NARRATED_REPLAY_BATCH_TRANSCRIPT_FIXTURE");
            std::env::remove_var("OPENAI_API_KEY");
        }
    }

    fn wav_bytes(payload_len: usize) -> Vec<u8> {
        let mut bytes = b"RIFF".to_vec();
        let data_len = payload_len as u32;
        bytes.extend_from_slice(&(36 + data_len).to_le_bytes());
        bytes.extend_from_slice(b"WAVEfmt ");
        bytes.extend_from_slice(&16_u32.to_le_bytes());
        bytes.extend_from_slice(&1_u16.to_le_bytes());
        bytes.extend_from_slice(&1_u16.to_le_bytes());
        bytes.extend_from_slice(&24_000_u32.to_le_bytes());
        bytes.extend_from_slice(&(24_000_u32 * 2).to_le_bytes());
        bytes.extend_from_slice(&2_u16.to_le_bytes());
        bytes.extend_from_slice(&16_u16.to_le_bytes());
        bytes.extend_from_slice(b"data");
        bytes.extend_from_slice(&data_len.to_le_bytes());
        bytes.extend(std::iter::repeat_n(0_u8, payload_len));
        bytes
    }

    fn unique_tmp(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        PathBuf::from("/private/tmp").join(format!("{prefix}-{nanos}"))
    }
}
