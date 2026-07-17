#[cfg(test)]
mod io_tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn final_alignment_disabled_when_cleaned_transcript_missing() {
        let root = unique_tmp("nrr-align-missing-cleaned");
        fs::create_dir_all(&root).unwrap();

        let output = ensure_final_alignment(&root).unwrap();
        let receipt: Value = serde_json::from_str(
            &fs::read_to_string(root.join("final-transcript-alignment-receipt.json")).unwrap(),
        )
        .unwrap();

        assert!(output.is_none());
        assert_eq!(receipt["status"], "disabled");
        assert!(
            receipt["reason"]
                .as_str()
                .unwrap()
                .starts_with("unverified-cleaned-transcript")
        );
    }

    #[test]
    fn final_alignment_disabled_when_cleaned_text_empty() {
        let root = unique_tmp("nrr-align-empty-cleaned");
        fs::create_dir_all(&root).unwrap();
        fs::write(
            root.join("cleaned-transcript.json"),
            r#"{"cleanedText":"  "}"#,
        )
        .unwrap();

        let output = ensure_final_alignment(&root).unwrap();
        let receipt: Value = serde_json::from_str(
            &fs::read_to_string(root.join("final-transcript-alignment-receipt.json")).unwrap(),
        )
        .unwrap();

        assert!(output.is_none());
        assert!(
            receipt["reason"]
                .as_str()
                .unwrap()
                .starts_with("unverified-cleaned-transcript")
        );
    }

    #[test]
    fn final_alignment_writes_artifact_and_final_timeline() {
        let root = unique_tmp("nrr-align-write");
        fs::create_dir_all(&root).unwrap();
        let args = crate::config::parse_args_from([
            "nrr",
            "packet",
            "--session-dir",
            root.to_str().unwrap(),
            "--i-consent-to-custom-runtime-paths",
        ])
        .unwrap();
        crate::transcript_cleanup::write_bound_cleanup_fixture_for_test(
            &args,
            &root,
            "cloud code opened chat g p t atlas",
            "Claude Code opened ChatGPT Atlas.",
        )
        .unwrap();
        fs::write(
            root.join("transcript.timeline.jsonl"),
            r#"{"kind":"completed","monotonicOffsetMs":1000,"text":"cloud code opened chat g p t atlas"}"#,
        )
        .unwrap();

        let output = ensure_final_alignment(&root).unwrap().unwrap();
        let artifact: Value = serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
        let reread = final_segments(&root).unwrap();

        assert_eq!(
            artifact["wordAuthority"],
            "verified-cleaned-batch-transcript"
        );
        assert_eq!(
            artifact["privacy"]["rawSourcesCopiedIntoGeneratedPacketsByDefault"],
            false
        );
        assert_eq!(reread[0].text, "Claude Code opened ChatGPT Atlas.");
        assert!(root.join("final-transcript.timeline.jsonl").is_file());
    }

    #[test]
    fn final_segments_rejects_missing_malformed_and_non_segment_shapes() {
        let root = unique_tmp("nrr-align-final-segments-bad");
        fs::create_dir_all(&root).unwrap();

        assert!(final_segments(&root).is_none());
        fs::write(root.join("final-transcript-alignment.json"), "not json").unwrap();
        assert!(final_segments(&root).is_none());
        fs::write(
            root.join("final-transcript-alignment.json"),
            r#"{"segments":{}}"#,
        )
        .unwrap();
        assert!(final_segments(&root).is_none());
    }

    #[test]
    fn final_alignment_rejects_oversized_unbound_cleaned_artifact() {
        let root = unique_tmp("nrr-final-alignment-oversize");
        fs::create_dir_all(&root).unwrap();
        fs::File::create(root.join("cleaned-transcript.json"))
            .unwrap()
            .set_len(MAX_CLEANED_ARTIFACT_BYTES + 1)
            .unwrap();

        assert!(ensure_final_alignment(&root).unwrap().is_none());
        let receipt: Value = serde_json::from_str(
            &fs::read_to_string(root.join("final-transcript-alignment-receipt.json")).unwrap(),
        )
        .unwrap();
        assert!(
            receipt["reason"]
                .as_str()
                .unwrap()
                .starts_with("unverified-cleaned-transcript")
        );
    }

    #[test]
    fn final_segments_rejects_unbound_even_parseable_rows() {
        let root = unique_tmp("nrr-align-final-segments-good");
        fs::create_dir_all(&root).unwrap();
        fs::write(
            root.join("final-transcript-alignment.json"),
            r#"{
              "segments": [
                {"startMs": 10, "endMs": 20, "text": "keep me"},
                {"startMs": 30, "text": "drop me"},
                {
                  "startMs": 40,
                  "endMs": 55,
                  "text": "warn me",
                  "confidence": 0.42,
                  "sourceRealtimeSegmentIds": [1, "bad", 3],
                  "mismatch": "low-token-similarity"
                }
              ]
            }"#,
        )
        .unwrap();

        assert!(final_segments(&root).is_none());
    }

    #[test]
    fn proportional_alignment_reports_empty_realtime_and_trailing_cleaned_words() {
        assert!(
            align_cleaned_text("orphan cleaned words", &[])
                .unwrap()
                .is_empty()
        );

        let realtime = vec![timeline::TranscriptSegment {
            id: 1,
            start_ms: 100,
            end_ms: 200,
            monotonic_offset_ms: Some(100),
            timing_source: "fixture".to_string(),
            text: "alpha beta".to_string(),
        }];
        let aligned =
            align_cleaned_text("unrelated phrase with many extra cleaned tokens", &realtime)
                .unwrap();

        assert_eq!(aligned.len(), 1);
        assert_eq!(aligned[0].start_ms, 100);
        assert_eq!(aligned[0].source_realtime_ids, [1]);
        assert!(aligned[0].mismatch.is_some());
    }

    #[test]
    fn fallback_alignment_can_create_trailing_review_segment() {
        let realtime = vec![
            timeline::TranscriptSegment {
                id: 0,
                start_ms: 0,
                end_ms: 100,
                monotonic_offset_ms: None,
                timing_source: "fixture".to_string(),
                text: "tiny".to_string(),
            },
            timeline::TranscriptSegment {
                id: 0,
                start_ms: 100,
                end_ms: 200,
                monotonic_offset_ms: None,
                timing_source: "fixture".to_string(),
                text: "tiny".to_string(),
            },
        ];

        let aligned =
            align_cleaned_text("one two three four five six seven eight", &realtime).unwrap();

        assert!(aligned.iter().any(|segment| segment.mismatch.as_deref()
            == Some("cleaned-trailing-words-without-realtime-window")));
    }

    fn unique_tmp(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        PathBuf::from("/private/tmp").join(format!("{prefix}-{nanos}"))
    }
}
