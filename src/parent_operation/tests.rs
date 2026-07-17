#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn parses_utc_timestamp_ms() {
        assert_eq!(
            parse_utc_timestamp_ms("2026-06-20T01:32:03Z").unwrap(),
            1_781_919_123_000
        );
    }

    #[test]
    fn periodic_commit_completed_segments_count_as_drain_completion() {
        let drain = json!({
            "completedSegments": 0,
            "captureStats": {
                "realtimeCompletedSegmentsObserved": 15
            }
        });

        assert_eq!(completed_transcript_segments_from_drain(&drain), Some(15));
    }

    #[test]
    fn parent_time_helpers_reject_malformed_inputs() {
        for value in [
            "2026-06-20 01:32:03Z",
            "2026-06-20T01:32Z",
            "2026-06-20T01:32:03:99Z",
            "2026-06-20Txx:32:03Z",
            "2026-13-01T01:32:03Z",
            "2026-02-31T25:61:61Z",
            "9223372036854775807-01-01T00:00:00Z",
        ] {
            assert!(parse_utc_timestamp_ms(value).is_err(), "{value}");
        }
        assert_eq!(completed_transcript_segments_from_drain(&json!({})), None);
    }

    #[test]
    fn parent_file_helpers_require_regular_bounded_json() {
        let root = unique_tmp("nrr-parent-file-helpers");
        fs::create_dir_all(&root).unwrap();
        let json_path = root.join("value.json");
        fs::write(&json_path, r#"{"ok":true}"#).unwrap();

        assert_eq!(read_json(&json_path).unwrap()["ok"], true);
        assert!(read_json(&root.join("missing.json")).is_err());
        fs::File::create(&json_path)
            .unwrap()
            .set_len(PARENT_JSON_MAX_BYTES + 1)
            .unwrap();
        assert!(
            read_json(&json_path)
                .unwrap_err()
                .to_string()
                .contains("byte limit")
        );
    }

    #[test]
    fn first_audio_chunk_clock_is_start_proof_when_available() {
        let fixture = parent_fixture("nrr-parent-first-audio", true, true, 1_000);

        let evaluation = evaluate_parent_operation(
            &fixture.session_dir,
            fixture.metadata_path.to_str().unwrap(),
            fixture.events_path.to_str().unwrap(),
        )
        .unwrap();

        assert_eq!(evaluation.start_delta_ms, 1_000);
        assert_eq!(
            evaluation.audio_started_at_source,
            "first-nonempty-ffmpeg-stdout-read"
        );
        assert_eq!(evaluation.status_text, TIMESTAMP_PROXIMITY_VERIFIED);
    }

    #[test]
    fn parent_operation_blocks_when_audio_start_delta_is_too_large() {
        let fixture = parent_fixture("nrr-parent-blocked-delta", true, true, 19_000);

        let evaluation = evaluate_parent_operation(
            &fixture.session_dir,
            fixture.metadata_path.to_str().unwrap(),
            fixture.events_path.to_str().unwrap(),
        )
        .unwrap();

        assert_eq!(evaluation.status_text, "blocked");
        assert_eq!(evaluation.start_delta_ms, 19_000);
        assert!(!evaluation.within_allowed_start_delta);
    }

    #[test]
    fn write_parent_operation_receipt_records_metadata_only_proof() {
        let fixture = parent_fixture("nrr-parent-write-receipt", true, true, 1_000);
        let args = crate::config::parse_args_from([
            "nrr",
            "receipt",
            "--session-dir",
            fixture.session_dir.to_str().unwrap(),
            "--recording-metadata",
            fixture.metadata_path.to_str().unwrap(),
            "--recording-events",
            fixture.events_path.to_str().unwrap(),
            "--i-consent-to-custom-runtime-paths",
        ])
        .unwrap();

        write_parent_operation_receipt(&args).unwrap();
        let receipt: Value = serde_json::from_str(
            &fs::read_to_string(fixture.session_dir.join("parent-operation-receipt.json")).unwrap(),
        )
        .unwrap();

        assert_eq!(receipt["status"], TIMESTAMP_PROXIMITY_VERIFIED);
        assert_eq!(receipt["rawPayloadCopied"], false);
        assert_eq!(
            receipt["sameStartChecks"]["recordReplayEventsPresent"],
            true
        );
        assert!(
            receipt["recordReplay"]["metadataDigest"]
                .as_str()
                .unwrap()
                .starts_with("sha256:")
        );
        assert!(
            receipt["recordReplay"]["eventsDigest"]
                .as_str()
                .unwrap()
                .starts_with("sha256:")
        );
    }

    struct ParentFixture {
        session_dir: PathBuf,
        metadata_path: PathBuf,
        events_path: PathBuf,
    }

    fn parent_fixture(prefix: &str, stopped: bool, drain: bool, delta_ms: i64) -> ParentFixture {
        let root = unique_tmp(prefix);
        let session_dir = root.join("session");
        let rnr_dir = root.join("rnr");
        fs::create_dir_all(&session_dir).unwrap();
        fs::create_dir_all(&rnr_dir).unwrap();
        let metadata_path = rnr_dir.join("session.json");
        let events_path = rnr_dir.join("events.jsonl");
        fs::write(
            &metadata_path,
            r#"{"id":"rnr-1","startedAt":"2026-06-19T01:38:25Z"}"#,
        )
        .unwrap();
        fs::write(
            &events_path,
            r#"{"id":1,"kind":"click","timestamp":"2026-06-19T01:38:26Z"}"#,
        )
        .unwrap();
        fs::write(
            session_dir.join("status.json"),
            format!(
                r#"{{"state":"{}","audioInput":{{"deviceName":"MacBook Pro Microphone"}}}}"#,
                if stopped { "stopped" } else { "failed" }
            ),
        )
        .unwrap();
        fs::write(
            session_dir.join("capture-clock.json"),
            format!(
                r#"{{"audioStartedAtUnixMs":1781833105000,"firstAudioChunkAtUnixMs":{}}}"#,
                1_781_833_105_000_i64 + delta_ms
            ),
        )
        .unwrap();
        fs::write(
            session_dir.join("post-commit-drain.json"),
            format!(
                r#"{{"completedSegments":{},"errors":[]}}"#,
                if drain { 1 } else { 0 }
            ),
        )
        .unwrap();
        ParentFixture {
            session_dir,
            metadata_path,
            events_path,
        }
    }

    fn unique_tmp(prefix: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        PathBuf::from("/private/tmp").join(format!("{prefix}-{nanos}"))
    }
}
