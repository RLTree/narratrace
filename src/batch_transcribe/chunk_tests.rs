#[cfg(test)]
mod chunk_tests {
    use super::*;
    use serde_json::json;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn opened_audio_rejects_total_bound_before_upload() {
        let root = unique_tmp("nrr-audio-total-bound");
        fs::create_dir_all(&root).unwrap();
        let audio = root.join("retained-audio.wav");
        fs::File::create(&audio)
            .unwrap()
            .set_len(MAX_RETAINED_AUDIO_BYTES + 1)
            .unwrap();
        fs::write(
            root.join("audio-retention.json"),
            json!({"audioPath": audio}).to_string(),
        )
        .unwrap();

        assert!(
            open_current_audio(&root)
                .unwrap_err()
                .to_string()
                .contains("exceeds")
        );
        assert!(!root.join("batch-audio-chunks.json").exists());
    }

    #[test]
    fn chunk_builder_preserves_even_pcm_payload() {
        let chunk = chunk_wav_bytes(&[1, 2, 3, 4], 0, 4).unwrap();

        assert_eq!(&chunk[0..4], b"RIFF");
        assert_eq!(&chunk[44..], &[1, 2, 3, 4]);
    }

    fn unique_tmp(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        PathBuf::from("/private/tmp").join(format!("{prefix}-{nanos}"))
    }
}
