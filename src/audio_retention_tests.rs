use super::*;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn writes_private_wav_header_with_data_size() {
    let root = unique_tmp("nrr-audio-retention");
    fs::create_dir_all(&root).unwrap();
    let mut writer = AudioRetentionWriter::create(&root, None, "private-wav")
        .unwrap()
        .unwrap();
    writer.append(&[1, 2, 3, 4], 12).unwrap();
    let path = writer.finalize(&root, "private-wav").unwrap();
    let bytes = fs::read(path).unwrap();
    assert_eq!(&bytes[0..4], b"RIFF");
    assert_eq!(&bytes[8..12], b"WAVE");
    assert_eq!(u32::from_le_bytes(bytes[40..44].try_into().unwrap()), 4);
    let manifest: serde_json::Value = read_json(&root.join("audio-retention.json"));
    assert_eq!(
        manifest.get("samples").and_then(serde_json::Value::as_u64),
        Some(2)
    );
    let chunks = fs::read_to_string(root.join("audio-chunks.jsonl")).unwrap();
    assert!(chunks.contains("\"sampleStart\":0"));
    assert!(chunks.contains("\"sampleEnd\":2"));
    assert!(chunks.contains("\"monotonicOffsetMs\":12"));
}

#[test]
fn disabled_mode_writes_metadata_only_manifest() {
    let root = unique_tmp("nrr-audio-retention-disabled");
    fs::create_dir_all(&root).unwrap();
    let writer = AudioRetentionWriter::create(&root, None, "disabled").unwrap();
    let manifest = read_json(&root.join("audio-retention.json"));
    assert!(writer.is_none());
    assert_eq!(manifest["mode"], "disabled");
    assert_eq!(manifest["format"], "not-retained");
    assert_eq!(manifest["privacy"]["containsRawMicrophoneAudio"], false);
}

#[test]
fn explicit_path_manifest_records_custom_private_audio_path() {
    let root = unique_tmp("nrr-audio-retention-explicit");
    let audio_path = root.join("private/same-stream.wav");
    fs::create_dir_all(&root).unwrap();
    let writer = AudioRetentionWriter::create(&root, Some(&audio_path), "private-wav")
        .unwrap()
        .unwrap();
    let path = writer.finalize(&root, "private-wav").unwrap();
    let manifest = read_json(&root.join("audio-retention.json"));
    assert_eq!(path, audio_path);
    assert_eq!(manifest["audioPath"], audio_path.display().to_string());
    assert_eq!(
        manifest["privacy"]["copyIntoGeneratedPacketsByDefault"],
        false
    );
}

#[test]
fn explicit_path_refuses_to_destroy_an_existing_file() {
    let root = unique_tmp("nrr-audio-retention-existing");
    let audio_path = root.join("selected.wav");
    fs::create_dir_all(&root).unwrap();
    fs::write(&audio_path, "ORIGINAL-USER-CONTENT").unwrap();
    let error = AudioRetentionWriter::create(&root, Some(&audio_path), "private-wav")
        .err()
        .expect("existing file must be refused");
    assert!(!error.to_string().is_empty());
    assert_eq!(
        fs::read_to_string(audio_path).unwrap(),
        "ORIGINAL-USER-CONTENT"
    );
}

fn read_json(path: &Path) -> serde_json::Value {
    serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

fn unique_tmp(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    PathBuf::from("/private/tmp").join(format!("{prefix}-{nanos}"))
}
