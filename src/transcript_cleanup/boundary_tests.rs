#[cfg(test)]
mod boundary_tests {
    use super::*;
    use crate::config::parse_args_from;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn fixture_outside_current_test_session_is_rejected() {
        let _guard = lock_cleanup_env();
        let root = unique_tmp("nrr-cleanup-session");
        let outside = unique_tmp("nrr-cleanup-unrelated.json");
        fs::write(&outside, r#"{"output_text":"unrelated local material"}"#).unwrap();
        unsafe {
            std::env::set_var("NARRATED_REPLAY_CLEANUP_TRANSCRIPT_FIXTURE", &outside);
        }
        let args = args_for(&root);
        write_batch(&root, &args);

        let error = ensure_cleaned_transcript(&args, &root).unwrap_err();
        clear_env();

        assert!(error.to_string().contains("current test session"));
        assert!(!root.join("cleaned-transcript.json").exists());
    }

    #[test]
    fn openai_cleanup_call_requires_current_call_consent() {
        let _guard = lock_cleanup_env();
        clear_env();
        let root = unique_tmp("nrr-cleanup-no-consent");
        let args = args_for(&root);
        write_batch(&root, &args);

        let error = ensure_cleaned_transcript(&args, &root).unwrap_err();

        assert!(
            error
                .to_string()
                .contains("--i-consent-to-openai-postprocessing")
        );
        assert!(!root.join("cleaned-transcript.json").exists());
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

    fn write_batch(root: &Path, args: &crate::config::Args) {
        fs::create_dir_all(root).unwrap();
        batch_transcribe::write_bound_batch_fixture_for_test(
            root,
            &args.batch_transcription_model,
            "current session words",
        )
        .unwrap();
    }

    fn clear_env() {
        unsafe {
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
