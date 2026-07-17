use super::*;

#[test]
fn similarity_handles_claude_code_vs_cloud_code() {
    assert!(phrase_similarity("cloud code", "Claude Code") >= 0.5);
}

#[test]
fn hallucinated_inserted_word_lowers_but_does_not_break_alignment() {
    let similarity = phrase_similarity(
        "chicken tinga pizza flabbergasted",
        "Chicken Tinga pizza nuggets flavors",
    );

    assert!(similarity > 0.3);
    assert!(similarity < 0.9);
}

#[test]
fn chatgpt_atlas_split_wording_still_matches() {
    assert!(phrase_similarity("chat g p t atlas web browser", "ChatGPT Atlas web browser") > 0.4);
}

#[test]
fn marker_alignment_reduces_marker_window_mismatches() {
    let realtime = vec![
        timeline::TranscriptSegment {
            id: 1,
            start_ms: 0,
            end_ms: 1_000,
            monotonic_offset_ms: None,
            timing_source: "fixture".to_string(),
            text: "Alpha marker yellow canoe".to_string(),
        },
        timeline::TranscriptSegment {
            id: 2,
            start_ms: 1_000,
            end_ms: 2_000,
            monotonic_offset_ms: None,
            timing_source: "fixture".to_string(),
            text: "Bravo marker cactus receipts train stations".to_string(),
        },
    ];

    let aligned = align_cleaned_text(
            "Alpha marker. A yellow canoe is arguing with twelve invisible umbrellas. Bravo marker. My neighbor's cactus collects receipts from imaginary train stations.",
            &realtime,
        )
        .unwrap();

    assert_eq!(aligned.len(), 2);
    assert!(aligned[0].mismatch.is_none());
    assert!(aligned[1].mismatch.is_none());
    assert!(aligned[1].text.contains("Bravo marker"));
}

#[test]
fn marker_alignment_keeps_long_utterance_as_one_final_segment() {
    let realtime = vec![
        timeline::TranscriptSegment {
            id: 1,
            start_ms: 0,
            end_ms: 5_000,
            monotonic_offset_ms: None,
            timing_source: "fixture".to_string(),
            text: "Quebec marker The last velvet".to_string(),
        },
        timeline::TranscriptSegment {
            id: 2,
            start_ms: 5_000,
            end_ms: 10_000,
            monotonic_offset_ms: None,
            timing_source: "fixture".to_string(),
            text: "pineapple refused to discuss elevators arithmetic".to_string(),
        },
        timeline::TranscriptSegment {
            id: 3,
            start_ms: 10_000,
            end_ms: 15_000,
            monotonic_offset_ms: None,
            timing_source: "fixture".to_string(),
            text: "or unusually polite geese".to_string(),
        },
        timeline::TranscriptSegment {
            id: 4,
            start_ms: 15_000,
            end_ms: 20_000,
            monotonic_offset_ms: None,
            timing_source: "fixture".to_string(),
            text: "Zulu marker done".to_string(),
        },
    ];

    let aligned = align_cleaned_text(
            "Quebec marker. The last velvet pineapple refused to discuss elevators, arithmetic, or unusually polite geese. Zulu marker. Done.",
            &realtime,
        )
        .unwrap();

    assert_eq!(aligned.len(), 2);
    assert_eq!(aligned[0].source_realtime_ids, vec![1, 2, 3]);
    assert_eq!(aligned[0].start_ms, 0);
    assert_eq!(aligned[0].end_ms, 15_000);
    assert!(aligned[0].mismatch.is_none());
}

#[test]
fn marker_alignment_tolerates_realtime_split_marker_phrase() {
    let realtime = vec![
        timeline::TranscriptSegment {
            id: 1,
            start_ms: 0,
            end_ms: 5_000,
            monotonic_offset_ms: None,
            timing_source: "fixture".to_string(),
            text: "Charlie marker violin sneezed twice apologized once Delta".to_string(),
        },
        timeline::TranscriptSegment {
            id: 2,
            start_ms: 5_000,
            end_ms: 10_000,
            monotonic_offset_ms: None,
            timing_source: "fixture".to_string(),
            text: "marker left handed comet delivered three envelopes".to_string(),
        },
        timeline::TranscriptSegment {
            id: 3,
            start_ms: 10_000,
            end_ms: 15_000,
            monotonic_offset_ms: None,
            timing_source: "fixture".to_string(),
            text: "Echo marker fourteen buttons".to_string(),
        },
    ];

    let aligned = align_cleaned_text(
            "Charlie marker. The violin sneezed twice. Delta marker. A left-handed comet delivered three envelopes. Echo marker. Fourteen buttons.",
            &realtime,
        )
        .unwrap();

    assert_eq!(aligned.len(), 3);
    assert!(aligned[1].text.starts_with("Delta marker"));
    assert_eq!(aligned[1].source_realtime_ids, vec![1, 2]);
    assert!(aligned[1].mismatch.is_none());
}
