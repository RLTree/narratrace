#[test]
fn rejects_quadratic_alignment_work_before_matrix_allocation() {
    let cleaned_text = format!("{}. {}.", "alpha ".repeat(1_000), "bravo ".repeat(1_000));
    let realtime = vec![timeline::TranscriptSegment {
        id: 1,
        start_ms: 0,
        end_ms: 1_000,
        monotonic_offset_ms: None,
        timing_source: "fixture".to_string(),
        text: format!("{} {}", "alpha ".repeat(1_000), "bravo ".repeat(1_000)),
    }];

    let error = align_cleaned_text(&cleaned_text, &realtime).unwrap_err();

    assert!(error.to_string().contains("alignment work limit exceeded"));
}

#[test]
fn rejects_single_oversized_token_dimension() {
    let error = enforce_alignment_limits(MAX_ALIGNMENT_TOKENS_PER_SIDE + 1, 1).unwrap_err();

    assert!(error.to_string().contains("alignment token limit exceeded"));
}

#[test]
fn preserves_valid_monotonic_alignment() {
    let cleaned = vec!["alpha".to_string(), "bravo".to_string()];
    let realtime = vec![
        RealtimeToken {
            text: "alpha".to_string(),
            segment_index: 0,
        },
        RealtimeToken {
            text: "bravo".to_string(),
            segment_index: 1,
        },
    ];

    assert_eq!(
        monotonic_token_alignment(&cleaned, &realtime).unwrap(),
        vec![Some(0), Some(1)]
    );
}

#[test]
fn rejects_giant_tokens_before_similarity_work() {
    let cleaned = format!("{}.", "a".repeat(MAX_ALIGNMENT_TOKEN_BYTES + 1));
    let realtime = vec![timeline::TranscriptSegment {
        id: 1,
        start_ms: 0,
        end_ms: 1,
        monotonic_offset_ms: None,
        timing_source: "fixture".into(),
        text: "short token".into(),
    }];
    assert!(align_cleaned_text(&cleaned, &realtime)
        .unwrap_err()
        .to_string()
        .contains("token byte limit"));
}

#[test]
fn bounded_edit_distance_exits_outside_threshold() {
    assert_eq!(bounded_edit_distance(b"abcdefgh", b"abcdxfgh", 2), 1);
    assert_eq!(bounded_edit_distance(b"abcdefgh", b"xyzuvwab", 2), 3);
    assert_eq!(bounded_edit_distance(b"a", b"abcdef", 2), 3);
}
