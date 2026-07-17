fn write_sync_sentinel(
    session_dir: &std::path::Path,
    phase: &str,
    monotonic_offset_ms: u64,
) -> Result<()> {
    let line = format!(
        "{}\n",
        serde_json::to_string(&json!({
            "schema": "narrated-record-replay.narration-sync.v1",
            "phase": phase,
            "monotonicOffsetMs": monotonic_offset_ms,
            "source": "narration-capture-process",
            "privacy": {
                "rawTranscriptCopied": false,
                "rawAudioCopied": false,
                "metadataOnly": true
            }
        }))?
    );
    crate::private_fs::append_private(session_dir.join("narration.sync.jsonl"), &line)
}

#[cfg(test)]
mod sync_tests {
    use super::write_sync_sentinel;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn sync_sentinel_is_metadata_only_and_monotonic() {
        let root = unique_tmp("nrr-sync-sentinel");
        fs::create_dir_all(&root).unwrap();

        write_sync_sentinel(&root, "start", 0).unwrap();
        write_sync_sentinel(&root, "stop", 1234).unwrap();

        let lines = fs::read_to_string(root.join("narration.sync.jsonl")).unwrap();
        let records: Vec<serde_json::Value> = lines
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0]["phase"], "start");
        assert_eq!(records[1]["monotonicOffsetMs"], 1234);
        assert_eq!(records[1]["privacy"]["rawAudioCopied"], false);
        assert_eq!(records[1]["privacy"]["rawTranscriptCopied"], false);
    }

    fn unique_tmp(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        PathBuf::from("/private/tmp").join(format!("{prefix}-{nanos}"))
    }
}
