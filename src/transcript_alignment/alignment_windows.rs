fn align_cleaned_utterances(
    cleaned_text: &str,
    realtime_segments: &[timeline::TranscriptSegment],
) -> Result<Option<Vec<FinalSegment>>> {
    let utterances = cleaned_utterance_spans(cleaned_text);
    if utterances.len() < 2 {
        return Ok(None);
    }
    let cleaned_tokens = normalized_tokens(cleaned_text);
    let realtime_tokens = realtime_tokens(realtime_segments);
    let token_alignment = monotonic_token_alignment(&cleaned_tokens, &realtime_tokens)?;

    let mut out = Vec::with_capacity(utterances.len());
    let mut previous_end_ms = 0_u64;
    for utterance in &utterances {
        let mut matched_realtime_indices = (utterance.token_start..utterance.token_end)
            .filter_map(|cleaned_index| token_alignment.get(cleaned_index).copied().flatten())
            .collect::<Vec<_>>();
        matched_realtime_indices.sort_unstable();
        matched_realtime_indices.dedup();

        let mut source_segment_indices = dedup_sorted(
            matched_realtime_indices
                .iter()
                .filter_map(|token_index| realtime_tokens.get(*token_index))
                .map(|token| token.segment_index)
                .collect(),
        );

        if let Some(label) = &utterance.marker_label {
            if let Some(marker_segment_index) = marker_segment_index(
                realtime_segments,
                label,
                source_segment_indices.first().copied().unwrap_or(0),
            ) {
                source_segment_indices.push(marker_segment_index);
                source_segment_indices = dedup_sorted(source_segment_indices);
            }
        }

        if let (Some(first), Some(last)) = (
            source_segment_indices.first().copied(),
            source_segment_indices.last().copied(),
        ) {
            source_segment_indices = (first..=last).collect();
        }

        let (start_ms, end_ms, source_realtime_ids, realtime_text) =
            if source_segment_indices.is_empty() {
                let (start_ms, end_ms) = proportional_window(
                    utterance.token_start,
                    utterance.token_end,
                    cleaned_tokens.len(),
                    realtime_segments,
                );
                (start_ms, end_ms, Vec::new(), String::new())
            } else {
                let source = source_segment_indices
                    .iter()
                    .filter_map(|index| realtime_segments.get(*index))
                    .collect::<Vec<_>>();
                (
                    source.first().map(|segment| segment.start_ms).unwrap_or(0),
                    source.last().map(|segment| segment.end_ms).unwrap_or(0),
                    source.iter().map(|segment| segment.id).collect::<Vec<_>>(),
                    source
                        .iter()
                        .map(|segment| segment.text.as_str())
                        .collect::<Vec<_>>()
                        .join(" "),
                )
            };

        let utterance_token_count = utterance
            .token_end
            .saturating_sub(utterance.token_start)
            .max(1);
        let token_confidence = matched_realtime_indices.len() as f64 / utterance_token_count as f64;
        let phrase_confidence = if realtime_text.is_empty() {
            0.0
        } else {
            phrase_similarity(&realtime_text, &utterance.text)
        };
        let confidence = token_confidence.max(phrase_confidence);
        let mismatch = utterance_alignment_mismatch(
            confidence,
            source_realtime_ids.is_empty(),
            utterance.marker_label.as_deref(),
            &realtime_text,
        );

        let start_ms = start_ms.max(previous_end_ms);
        let end_ms = end_ms.max(start_ms);
        previous_end_ms = end_ms;

        out.push(FinalSegment {
            id: out.len() + 1,
            start_ms,
            end_ms,
            text: utterance.text.clone(),
            confidence,
            source_realtime_ids,
            mismatch,
        });
    }
    Ok(Some(out))
}

fn utterance_alignment_mismatch(
    confidence: f64,
    has_no_source_window: bool,
    marker_label: Option<&str>,
    realtime_text: &str,
) -> Option<String> {
    if has_no_source_window {
        return Some("no-token-alignment-for-cleaned-utterance".to_string());
    }
    let marker_seen = marker_label
        .map(|label| text_has_marker_label(realtime_text, label))
        .unwrap_or(true);
    if confidence < 0.3 {
        Some("low-cleaned-utterance-token-similarity".to_string())
    } else if !marker_seen && confidence < 0.6 {
        Some("optional-marker-anchor-not-found-low-confidence-window".to_string())
    } else {
        None
    }
}

fn marker_segment_index(
    realtime_segments: &[timeline::TranscriptSegment],
    label: &str,
    start_index: usize,
) -> Option<usize> {
    realtime_segments
        .iter()
        .enumerate()
        .skip(start_index)
        .find(|(_, segment)| text_has_marker_label(&segment.text, label))
        .map(|(index, _)| index)
}

fn proportional_window(
    token_start: usize,
    token_end: usize,
    total_tokens: usize,
    realtime_segments: &[timeline::TranscriptSegment],
) -> (u64, u64) {
    let Some(first) = realtime_segments.first() else {
        return (0, 0);
    };
    let Some(last) = realtime_segments.last() else {
        return (0, 0);
    };
    let total_tokens = total_tokens.max(1) as u64;
    let duration = last.end_ms.saturating_sub(first.start_ms);
    let start = first
        .start_ms
        .saturating_add(duration.saturating_mul(token_start as u64) / total_tokens);
    let end = first
        .start_ms
        .saturating_add(duration.saturating_mul(token_end as u64) / total_tokens);
    (start, end.max(start))
}
