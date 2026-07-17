#[test]
fn marker_alignment_tolerates_realtime_marker_label_misrecognition() {
    let realtime = vec![
        timeline::TranscriptSegment {
            id: 1,
            start_ms: 0,
            end_ms: 5_000,
            monotonic_offset_ms: None,
            timing_source: "fixture".to_string(),
            text: "Keyhole marker Fesper noodle prism".to_string(),
        },
        timeline::TranscriptSegment {
            id: 2,
            start_ms: 5_000,
            end_ms: 10_000,
            monotonic_offset_ms: None,
            timing_source: "fixture".to_string(),
            text: "Lima marker meticulous walrus".to_string(),
        },
        timeline::TranscriptSegment {
            id: 3,
            start_ms: 10_000,
            end_ms: 15_000,
            monotonic_offset_ms: None,
            timing_source: "fixture".to_string(),
            text: "mic marker first came the spoon".to_string(),
        },
        timeline::TranscriptSegment {
            id: 4,
            start_ms: 15_000,
            end_ms: 20_000,
            monotonic_offset_ms: None,
            timing_source: "fixture".to_string(),
            text: "pop a marker nine astronomers".to_string(),
        },
    ];

    let aligned = align_cleaned_text(
            "Kilo marker. Vesper noodle prism. Lima marker. A meticulous walrus. Mike marker. First came the spoon. Papa marker. Nine astronomers.",
            &realtime,
        )
        .unwrap();

    assert_eq!(aligned.len(), 4);
    assert!(aligned[0].text.starts_with("Kilo marker"));
    assert!(aligned[2].text.starts_with("Mike marker"));
    assert!(aligned[3].text.starts_with("Papa marker"));
}

#[test]
fn token_alignment_does_not_require_script_markers() {
    let realtime = vec![
        timeline::TranscriptSegment {
            id: 1,
            start_ms: 0,
            end_ms: 5_000,
            monotonic_offset_ms: None,
            timing_source: "fixture".to_string(),
            text: "I am opening the report and checking the title".to_string(),
        },
        timeline::TranscriptSegment {
            id: 2,
            start_ms: 5_000,
            end_ms: 10_000,
            monotonic_offset_ms: None,
            timing_source: "fixture".to_string(),
            text: "now I scroll to the glossary because the acronym is confusing".to_string(),
        },
        timeline::TranscriptSegment {
            id: 3,
            start_ms: 10_000,
            end_ms: 15_000,
            monotonic_offset_ms: None,
            timing_source: "fixture".to_string(),
            text: "I am selecting this paragraph because the source claim needs checking"
                .to_string(),
        },
    ];

    let aligned = align_cleaned_text(
            "I am opening the report and checking the title. Now I scroll to the glossary because the acronym is confusing. I am selecting this paragraph because the source claim needs checking.",
            &realtime,
        )
        .unwrap();

    assert_eq!(aligned.len(), 3);
    assert_eq!(aligned[0].source_realtime_ids, vec![1]);
    assert_eq!(aligned[1].source_realtime_ids, vec![2]);
    assert_eq!(aligned[2].source_realtime_ids, vec![3]);
    assert!(aligned.iter().all(|segment| segment.mismatch.is_none()));
}

#[test]
fn missing_marker_anchor_creates_local_uncertainty_not_global_fallback() {
    let realtime = vec![
        timeline::TranscriptSegment {
            id: 1,
            start_ms: 0,
            end_ms: 5_000,
            monotonic_offset_ms: None,
            timing_source: "fixture".to_string(),
            text: "Alpha marker copper pelican traded seven apricots".to_string(),
        },
        timeline::TranscriptSegment {
            id: 2,
            start_ms: 5_000,
            end_ms: 10_000,
            monotonic_offset_ms: None,
            timing_source: "fixture".to_string(),
            text: "imaginary landlord measured the hallway with a frozen baguette".to_string(),
        },
        timeline::TranscriptSegment {
            id: 3,
            start_ms: 10_000,
            end_ms: 15_000,
            monotonic_offset_ms: None,
            timing_source: "fixture".to_string(),
            text: "Charlie marker violin sneezed twice and boarded the ferry".to_string(),
        },
    ];

    let aligned = align_cleaned_text(
            "Alpha marker. A copper pelican traded seven apricots. Bravo marker. My imaginary landlord measured the hallway with a frozen baguette. Charlie marker. The violin sneezed twice and boarded the ferry.",
            &realtime,
        )
        .unwrap();

    assert_eq!(aligned.len(), 3);
    assert_eq!(aligned[0].source_realtime_ids, vec![1]);
    assert_eq!(aligned[1].source_realtime_ids, vec![2]);
    assert_eq!(aligned[2].source_realtime_ids, vec![3]);
    assert!(aligned[1].mismatch.is_none());
    assert!(aligned[0].mismatch.is_none());
    assert!(aligned[2].mismatch.is_none());
}
