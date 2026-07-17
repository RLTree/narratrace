#[cfg(test)]
mod api_extra_tests {
    use super::*;
    use crate::config::parse_args_from;
    use std::fs;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::path::PathBuf;
    use std::thread;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn cleanup_api_reports_non_success_response() {
        let (url, handle) = local_response(500, r#"{"error":"nope"}"#);
        let client = Client::builder().build().unwrap();

        let input = CleanupModelInput {
            trusted_instructions: CLEANUP_SEED.to_string(),
            untrusted_data: "prompt".to_string(),
        };
        let error = call_cleanup_api_with_url(&client, &url, "test-key", "gpt-5-mini", &input)
            .unwrap_err()
            .to_string();

        handle.join().unwrap();
        assert!(error.contains("cleanup failed with 500"));
        assert!(!error.contains("nope"));
        assert!(url.starts_with("http://127.0.0.1:"));
    }

    #[test]
    fn ensure_cleaned_transcript_uses_local_api_and_writes_openai_artifact() {
        let _guard = lock_cleanup_env();
        let root = unique_tmp("nrr-cleanup-local-api");
        fs::create_dir_all(&root).unwrap();
        fs::write(
            root.join("batch-transcript.json"),
            r#"{"transcription":{"text":"cloud code opened chat g p t atlas"}}"#,
        )
        .unwrap();
        let (url, handle) = local_response(
            200,
            r#"{"output_text":"Claude Code opened ChatGPT Atlas."}"#,
        );
        unsafe {
            std::env::remove_var("NARRATED_REPLAY_CLEANUP_TRANSCRIPT_FIXTURE");
            std::env::set_var("NARRATED_REPLAY_CLEANUP_API_URL", &url);
            std::env::set_var("OPENAI_API_KEY", "test-key");
        }
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
            "cloud code opened chat g p t atlas",
        )
        .unwrap();

        let output = ensure_cleaned_transcript(&args, &root).unwrap().unwrap();
        clear_cleanup_env();
        handle.join().unwrap();
        let artifact: Value = serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();

        assert_eq!(artifact["source"], "openai-responses");
        assert_eq!(artifact["cleanedText"], "Claude Code opened ChatGPT Atlas.");
        assert_eq!(artifact["model"], DEFAULT_CLEANUP_MODEL);
        assert!(artifact["modelFallback"].is_null());
    }

    #[test]
    fn ensure_cleaned_transcript_falls_back_after_primary_model_error() {
        let _guard = lock_cleanup_env();
        let root = unique_tmp("nrr-cleanup-local-api-fallback");
        fs::create_dir_all(&root).unwrap();
        fs::write(
            root.join("batch-transcript.json"),
            r#"{"transcription":{"text":"raw words"}}"#,
        )
        .unwrap();
        let (url, handle) = local_sequence([
            (500, r#"{"error":"primary unavailable"}"#),
            (200, r#"{"cleanedText":"Raw words."}"#),
        ]);
        unsafe {
            std::env::remove_var("NARRATED_REPLAY_CLEANUP_TRANSCRIPT_FIXTURE");
            std::env::set_var("NARRATED_REPLAY_CLEANUP_API_URL", &url);
            std::env::set_var("OPENAI_API_KEY", "test-key");
        }
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

        let output = ensure_cleaned_transcript(&args, &root).unwrap().unwrap();
        clear_cleanup_env();
        handle.join().unwrap();
        let artifact: Value = serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();

        assert_eq!(artifact["model"], DEFAULT_CLEANUP_FALLBACK_MODEL);
        assert_eq!(artifact["cleanedText"], "Raw words.");
        assert!(
            artifact["modelFallback"]
                .as_str()
                .unwrap()
                .contains("requested cleanup model")
        );
    }

    #[test]
    fn dictionary_ignores_dynamic_private_inputs() {
        let root = unique_tmp("nrr-cleanup-extra-dict");
        fs::create_dir_all(&root).unwrap();
        fs::write(
            root.join("dict.txt"),
            "Claude Code\nplain lowercase\nhttps://example.com\nChatGPT Atlas\n",
        )
        .unwrap();
        let args = parse_args_from([
            "nrr",
            "packet",
            "--session-dir",
            root.to_str().unwrap(),
            "--cleanup-dictionary-source",
            root.join("dict.txt").to_str().unwrap(),
            "--i-consent-to-custom-runtime-paths",
        ])
        .unwrap();
        let entries = build_dictionary(&args, &root);
        assert!(entries.contains(&"Claude Code".to_string()));
        assert!(!entries.contains(&"ChatGPT Atlas".to_string()));
    }

    fn local_response(status: u16, body: &'static str) -> (String, thread::JoinHandle<()>) {
        local_sequence([(status, body)])
    }

    fn local_sequence<const N: usize>(
        responses: [(u16, &'static str); N],
    ) -> (String, thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let handle = thread::spawn(move || {
            for (status, body) in responses {
                let (mut stream, _) = listener.accept().unwrap();
                let mut buffer = [0_u8; 4096];
                let _ = stream.read(&mut buffer);
                let response = format!(
                    "HTTP/1.1 {status} Test\r\nContent-Length: {}\r\n\r\n{body}",
                    body.len()
                );
                stream.write_all(response.as_bytes()).unwrap();
            }
        });
        (url, handle)
    }

    fn clear_cleanup_env() {
        unsafe {
            std::env::remove_var("NARRATED_REPLAY_CLEANUP_API_URL");
            std::env::remove_var("NARRATED_REPLAY_CLEANUP_TRANSCRIPT_FIXTURE");
            std::env::remove_var("OPENAI_API_KEY");
        }
    }

    fn unique_tmp(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        PathBuf::from("/private/tmp").join(format!("{prefix}-{nanos}"))
    }
}
