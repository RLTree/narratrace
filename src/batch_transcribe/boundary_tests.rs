#[cfg(test)]
mod boundary_tests {
    use super::*;
    use crate::config::parse_args_from;
    use serde_json::json;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn fixture_outside_current_test_session_is_rejected() {
        let _guard = lock_batch_env();
        let root = unique_tmp("nrr-batch-session");
        let outside = unique_tmp("nrr-batch-unrelated.json");
        fs::create_dir_all(&root).unwrap();
        fs::write(&outside, r#"{"text":"unrelated local material"}"#).unwrap();
        unsafe {
            std::env::set_var("NARRATED_REPLAY_BATCH_TRANSCRIPT_FIXTURE", &outside);
        }
        let args = args_for(&root);

        let error = ensure_batch_transcript(&args, &root, None, None).unwrap_err();
        clear_env();

        assert!(error.to_string().contains("current test session"));
        assert!(!root.join("batch-transcript.json").exists());
    }

    #[test]
    fn openai_batch_call_requires_current_call_consent() {
        let _guard = lock_batch_env();
        clear_env();
        let root = unique_tmp("nrr-batch-no-consent");
        fs::create_dir_all(&root).unwrap();
        let audio = root.join("retained-audio.wav");
        fs::write(&audio, wav_bytes(64)).unwrap();
        fs::write(
            root.join("audio-retention.json"),
            serde_json::to_string(&json!({"audioPath": audio})).unwrap(),
        )
        .unwrap();
        unsafe {
            std::env::set_var("OPENAI_API_KEY", "must-not-be-used");
        }
        let args = args_for(&root);

        let error = ensure_batch_transcript(&args, &root, None, None).unwrap_err();
        clear_env();

        assert!(
            error
                .to_string()
                .contains("--i-consent-to-openai-postprocessing")
        );
        assert!(!root.join("batch-transcript.json").exists());
    }

    fn args_for(root: &Path) -> crate::config::Args {
        parse_args_from([
            "nrr",
            "packet",
            "--session-dir",
            root.to_str().unwrap(),
            "--i-consent-to-custom-runtime-paths",
        ])
        .unwrap()
    }

    fn clear_env() {
        unsafe {
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
