#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::parse_args_from;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn delay_eval_writes_timing_and_alignment_metrics_without_raw_text() {
        let root = unique_tmp("nrr-delay-eval");
        let session_dir = root.join("session");
        let rnr_dir = root.join("rnr");
        fs::create_dir_all(&session_dir).unwrap();
        fs::create_dir_all(&rnr_dir).unwrap();
        let metadata_path = rnr_dir.join("session.json");
        let events_path = rnr_dir.join("events.jsonl");
        fs::write(&metadata_path, r#"{"startedAt":"2026-06-22T22:34:02Z"}"#).unwrap();
        fs::write(
            &events_path,
            "{\"kind\":\"click\"}\n{\"kind\":\"scroll\"}\n",
        )
        .unwrap();
        fs::write(
            session_dir.join("status.json"),
            r#"{"state":"stopped","delay":"high","model":"gpt-realtime-whisper"}"#,
        )
        .unwrap();
        fs::write(
            session_dir.join("capture-clock.json"),
            r#"{"audioStartedAtUnixMs":1782167643000,"firstAudioChunkAtUnixMs":1782167644000}"#,
        )
        .unwrap();
        fs::write(
            session_dir.join("transcript.timeline.jsonl"),
            "{\"kind\":\"delta\",\"monotonicOffsetMs\":1200,\"text\":\"private words\"}\n{\"kind\":\"completed\",\"monotonicOffsetMs\":5300,\"text\":\"Alpha marker private words\"}\n",
        )
        .unwrap();
        fs::write(
            session_dir.join("final-transcript-alignment.json"),
            r#"{"status":"aligned","unresolvedMismatches":0,"segments":[{"text":"Alpha marker"}]}"#,
        )
        .unwrap();
        let args = parse_args_from([
            "nrr",
            "delay-eval",
            "--session-dir",
            session_dir.to_str().unwrap(),
            "--recording-metadata",
            metadata_path.to_str().unwrap(),
            "--recording-events",
            events_path.to_str().unwrap(),
            "--i-consent-to-custom-runtime-paths",
        ])
        .unwrap();

        write_delay_evaluation(&args).unwrap();

        let output = fs::read_to_string(session_dir.join("delay-evaluation.json")).unwrap();
        let value: Value = serde_json::from_str(&output).unwrap();
        assert_eq!(
            value.pointer("/latencyMetrics/recordReplayStartToFirstAudioChunkMs"),
            Some(&json!(2000))
        );
        assert_eq!(
            value.pointer("/latencyMetrics/firstRealtimeDeltaLatencyMs"),
            Some(&json!(1200))
        );
        assert_eq!(
            value.pointer("/latencyMetrics/firstCompletedRealtimeSegmentLatencyMs"),
            Some(&json!(5300))
        );
        assert_eq!(
            value.pointer("/alignmentMetrics/finalAlignedUtteranceCount"),
            Some(&json!(1))
        );
        assert!(!output.contains("private words"));
        assert_eq!(
            value.pointer("/privacy/rawTranscriptCopied"),
            Some(&json!(false))
        );
        assert_eq!(
            value.pointer("/privacy/rawAudioCopied"),
            Some(&json!(false))
        );
        assert_eq!(
            value.pointer("/privacy/localProvenanceMetadataIncluded"),
            Some(&json!(true))
        );
    }

    #[test]
    fn delay_compare_requires_operator_review_before_default_change() {
        let root = unique_tmp("nrr-delay-compare");
        fs::create_dir_all(&root).unwrap();
        let baseline_path = root.join("high.json");
        let candidate_path = root.join("low.json");
        let output_dir = root.join("out");
        fs::create_dir_all(&output_dir).unwrap();
        fs::write(
            &baseline_path,
            evaluation_json("high", 19_778, 7_932, 10_613, 17, 0, 17),
        )
        .unwrap();
        fs::write(
            &candidate_path,
            evaluation_json("low", 3_000, 2_000, 5_000, 17, 0, 17),
        )
        .unwrap();
        let args = parse_args_from([
            "nrr",
            "delay-compare",
            "--session-dir",
            output_dir.to_str().unwrap(),
            "--baseline-delay-evaluation",
            baseline_path.to_str().unwrap(),
            "--candidate-delay-evaluation",
            candidate_path.to_str().unwrap(),
            "--i-consent-to-custom-runtime-paths",
        ])
        .unwrap();

        write_delay_comparison(&args).unwrap();

        let output = fs::read_to_string(output_dir.join("delay-comparison.json")).unwrap();
        let value: Value = serde_json::from_str(&output).unwrap();
        assert_eq!(
            value.get("status").and_then(Value::as_str),
            Some("candidate-lower-latency-needs-operator-review")
        );
        assert_eq!(
            value.pointer("/decision/defaultDelayChangeAllowed"),
            Some(&json!(false))
        );
        assert_eq!(
            value.pointer("/deltas/firstRealtimeDeltaDeltaMs"),
            Some(&json!(-5932))
        );
        assert!(!output.contains("private words"));
        assert_eq!(
            value.pointer("/privacy/rawTranscriptCopied"),
            Some(&json!(false))
        );
    }

    #[test]
    fn delay_compare_keeps_default_when_mismatches_regress() {
        let root = unique_tmp("nrr-delay-compare-regression");
        fs::create_dir_all(&root).unwrap();
        let baseline_path = root.join("high.json");
        let candidate_path = root.join("low.json");
        let output_dir = root.join("out");
        fs::create_dir_all(&output_dir).unwrap();
        fs::write(
            &baseline_path,
            evaluation_json("high", 19_778, 7_932, 10_613, 17, 0, 17),
        )
        .unwrap();
        fs::write(
            &candidate_path,
            evaluation_json("low", 3_000, 2_000, 5_000, 17, 2, 15),
        )
        .unwrap();
        let args = parse_args_from([
            "nrr",
            "delay-compare",
            "--session-dir",
            output_dir.to_str().unwrap(),
            "--baseline-delay-evaluation",
            baseline_path.to_str().unwrap(),
            "--candidate-delay-evaluation",
            candidate_path.to_str().unwrap(),
            "--i-consent-to-custom-runtime-paths",
        ])
        .unwrap();

        write_delay_comparison(&args).unwrap();

        let value: Value = serde_json::from_str(
            &fs::read_to_string(output_dir.join("delay-comparison.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(
            value.get("status").and_then(Value::as_str),
            Some("keep-current-default")
        );
        assert_eq!(
            value.pointer("/deltas/unresolvedMismatchDelta"),
            Some(&json!(2))
        );
    }

    fn evaluation_json(
        delay: &str,
        first_audio: i64,
        first_delta: i64,
        first_completed: i64,
        utterances: u64,
        mismatches: u64,
        marker_recall: u64,
    ) -> String {
        serde_json::to_string_pretty(&json!({
            "schema": "narrated-record-replay.delay-evaluation.v1",
            "status": "measured",
            "realtimeDelay": delay,
            "latencyMetrics": {
                "recordReplayStartToFirstAudioChunkMs": first_audio,
                "firstRealtimeDeltaLatencyMs": first_delta,
                "firstCompletedRealtimeSegmentLatencyMs": first_completed
            },
            "alignmentMetrics": {
                "finalAlignmentStatus": if mismatches == 0 { "aligned" } else { "aligned-with-review-warnings" },
                "finalAlignedUtteranceCount": utterances,
                "unresolvedMismatches": mismatches,
                "scriptedMarkerRecall": {
                    "found": marker_recall,
                    "diagnosticOnly": true
                }
            },
            "recordReplay": {
                "eventCount": 34
            },
            "privacy": {
                "rawTranscriptCopied": false,
                "rawAudioCopied": false
            }
        }))
        .unwrap()
    }

    fn unique_tmp(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        PathBuf::from(format!("/private/tmp/{name}-{nanos}"))
    }
}
