#[cfg(test)]
mod audio_extra_tests {
    use super::*;
    use serde_json::json;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn audio_path_requires_manifest_audio_path_and_regular_file() {
        let root = unique_tmp("nrr-audio-path-required");
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("audio-retention.json"), "{}").unwrap();

        let error = audio_path(&root).unwrap_err().to_string();

        assert!(error.contains("audio retention artifact is required"));
    }

    #[test]
    fn audio_path_rejects_manifest_target_outside_current_session() {
        let root = unique_tmp("nrr-audio-path-session-bound");
        fs::create_dir_all(&root).unwrap();
        let outside = root.with_extension("wav");
        fs::write(&outside, wav_bytes(2)).unwrap();
        fs::write(
            root.join("audio-retention.json"),
            json!({"audioPath": outside}).to_string(),
        )
        .unwrap();

        assert!(
            audio_path(&root)
                .unwrap_err()
                .to_string()
                .contains("current session")
        );
    }

    #[cfg(unix)]
    #[test]
    fn audio_path_rejects_symlinked_retention_manifest() {
        let root = unique_tmp("nrr-audio-manifest-symlink");
        fs::create_dir_all(&root).unwrap();
        let target = root.join("manifest-target.json");
        fs::write(
            &target,
            json!({"audioPath": root.join("retained-audio.wav")}).to_string(),
        )
        .unwrap();
        std::os::unix::fs::symlink(&target, root.join("audio-retention.json")).unwrap();

        assert!(audio_path(&root).is_err());
    }

    #[test]
    fn small_audio_opens_one_bound_handle_without_chunk_manifest() {
        let root = unique_tmp("nrr-small-audio-chunking");
        fs::create_dir_all(&root).unwrap();
        let audio = root.join("retained-audio.wav");
        fs::write(&audio, wav_bytes(64)).unwrap();

        fs::write(
            root.join("audio-retention.json"),
            json!({"audioPath": audio}).to_string(),
        )
        .unwrap();
        let opened = open_current_audio(&root).unwrap();

        assert_eq!(opened.len, wav_bytes(64).len() as u64);
        assert!(!root.join("batch-audio-chunks.json").exists());
    }

    #[test]
    fn chunking_rejects_audio_above_total_bound_before_materializing_chunks() {
        let root = unique_tmp("nrr-audio-total-bound");
        fs::create_dir_all(&root).unwrap();
        let audio = root.join("retained-audio.wav");
        let file = fs::File::create(&audio).unwrap();
        file.set_len(MAX_RETAINED_AUDIO_BYTES + 1).unwrap();

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
        assert!(!root.join("batch-audio-chunks").exists());
    }

    #[test]
    fn opened_audio_handle_remains_bound_after_path_replacement() {
        let root = unique_tmp("nrr-audio-opened-handle");
        fs::create_dir_all(&root).unwrap();
        let audio = root.join("retained-audio.wav");
        fs::write(&audio, b"original").unwrap();
        let mut opened = open_regular_file(&audio).unwrap();
        fs::rename(&audio, root.join("original.wav")).unwrap();
        fs::write(&audio, b"replacement").unwrap();
        let mut bytes = Vec::new();
        opened.read_to_end(&mut bytes).unwrap();

        assert_eq!(bytes, b"original");
    }

    #[test]
    fn combine_single_chunk_transcript_returns_original_value() {
        let value = json!({"text":"single chunk"});

        let combined = combine_chunk_transcripts(std::slice::from_ref(&value), 1);

        assert_eq!(combined, value);
    }

    fn wav_bytes(payload_len: usize) -> Vec<u8> {
        let mut bytes = Vec::new();
        let data_len = payload_len as u32;
        bytes.extend_from_slice(b"RIFF");
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
