#[test]
fn proportional_window_handles_empty_and_nonempty_realtime() {
    assert_eq!(proportional_window(0, 1, 2, &[]), (0, 0));
    let realtime = vec![
        timeline::TranscriptSegment {
            id: 1,
            start_ms: 1_000,
            end_ms: 3_000,
            monotonic_offset_ms: None,
            timing_source: "fixture".to_string(),
            text: "first".to_string(),
        },
        timeline::TranscriptSegment {
            id: 2,
            start_ms: 3_000,
            end_ms: 9_000,
            monotonic_offset_ms: None,
            timing_source: "fixture".to_string(),
            text: "second".to_string(),
        },
    ];

    assert_eq!(proportional_window(1, 2, 4, &realtime), (3_000, 5_000));
}

#[test]
fn alignment_mismatch_reports_no_source_and_low_similarity() {
    assert_eq!(
        utterance_alignment_mismatch(0.9, true, None, ""),
        Some("no-token-alignment-for-cleaned-utterance".to_string())
    );
    assert_eq!(
        utterance_alignment_mismatch(0.2, false, None, "unrelated"),
        Some("low-cleaned-utterance-token-similarity".to_string())
    );
    assert_eq!(
        utterance_alignment_mismatch(0.5, false, Some("Bravo"), "alpha words only"),
        Some("optional-marker-anchor-not-found-low-confidence-window".to_string())
    );
    assert_eq!(
        utterance_alignment_mismatch(0.9, false, Some("Bravo"), "alpha words only"),
        None
    );
}

#[test]
fn alignment_returns_none_for_single_cleaned_utterance() {
    let realtime = vec![timeline::TranscriptSegment {
        id: 1,
        start_ms: 0,
        end_ms: 1_000,
        monotonic_offset_ms: None,
        timing_source: "fixture".to_string(),
        text: "only one sentence".to_string(),
    }];

    assert!(
        align_cleaned_utterances("Only one sentence.", &realtime)
            .unwrap()
            .is_none()
    );
}

#[test]
fn alignment_uses_proportional_window_for_unmatched_utterance() {
    let realtime = vec![
        timeline::TranscriptSegment {
            id: 1,
            start_ms: 0,
            end_ms: 1_000,
            monotonic_offset_ms: None,
            timing_source: "fixture".to_string(),
            text: "alpha words".to_string(),
        },
        timeline::TranscriptSegment {
            id: 2,
            start_ms: 1_000,
            end_ms: 4_000,
            monotonic_offset_ms: None,
            timing_source: "fixture".to_string(),
            text: "bravo words".to_string(),
        },
    ];

    let aligned = align_cleaned_utterances("Xylophone quartz. Invisible mustard.", &realtime)
        .unwrap()
        .unwrap();

    assert_eq!(aligned.len(), 2);
    assert!(aligned[0].source_realtime_ids.is_empty());
    assert_eq!(
        aligned[0].mismatch.as_deref(),
        Some("no-token-alignment-for-cleaned-utterance")
    );
    assert!(aligned[0].end_ms <= aligned[1].start_ms);
}
