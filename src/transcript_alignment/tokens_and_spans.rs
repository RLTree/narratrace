fn realtime_tokens(realtime_segments: &[timeline::TranscriptSegment]) -> Vec<RealtimeToken> {
    realtime_segments
        .iter()
        .enumerate()
        .flat_map(|(segment_index, segment)| {
            normalized_tokens(&segment.text)
                .into_iter()
                .map(move |text| RealtimeToken {
                    text,
                    segment_index,
                })
        })
        .collect()
}

fn monotonic_token_alignment(
    cleaned_tokens: &[String],
    realtime_tokens: &[RealtimeToken],
) -> Result<Vec<Option<usize>>> {
    let n = cleaned_tokens.len();
    let m = realtime_tokens.len();
    enforce_alignment_limits(n, m)?;
    let mut dp = vec![vec![0_u16; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            dp[i][j] = if token_similar(&cleaned_tokens[i], &realtime_tokens[j].text) {
                dp[i + 1][j + 1] + 1
            } else {
                dp[i + 1][j].max(dp[i][j + 1])
            };
        }
    }

    let mut out = vec![None; n];
    let mut i = 0_usize;
    let mut j = 0_usize;
    while i < n && j < m {
        if token_similar(&cleaned_tokens[i], &realtime_tokens[j].text)
            && dp[i][j] == dp[i + 1][j + 1] + 1
        {
            out[i] = Some(j);
            i += 1;
            j += 1;
        } else if dp[i + 1][j] >= dp[i][j + 1] {
            i += 1;
        } else {
            j += 1;
        }
    }
    Ok(out)
}

fn dedup_sorted(mut values: Vec<usize>) -> Vec<usize> {
    values.sort_unstable();
    values.dedup();
    values
}

fn cleaned_utterance_spans(text: &str) -> Vec<CleanedUtterance> {
    let marker_spans = marker_utterance_spans(text);
    if marker_spans.len() >= 2 {
        return marker_spans;
    }

    sentence_utterance_spans(text)
}

fn sentence_utterance_spans(text: &str) -> Vec<CleanedUtterance> {
    let mut spans = Vec::new();
    let mut start = 0_usize;
    let mut token_start = 0_usize;
    let mut previous_was_boundary = false;
    for (index, ch) in text.char_indices() {
        let is_boundary = matches!(ch, '.' | '?' | '!' | '\n');
        if is_boundary && !previous_was_boundary {
            let end = index + ch.len_utf8();
            token_start = push_utterance_span(text, start, end, token_start, &mut spans);
            start = end;
        }
        previous_was_boundary = is_boundary;
    }
    push_utterance_span(text, start, text.len(), token_start, &mut spans);

    if spans.len() <= 1 {
        spans = fixed_token_utterance_spans(text, 20);
    }
    spans
}

fn fixed_token_utterance_spans(text: &str, max_tokens: usize) -> Vec<CleanedUtterance> {
    let tokens = text
        .split_whitespace()
        .filter(|part| !part.trim().is_empty())
        .collect::<Vec<_>>();
    if tokens.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut cursor = 0_usize;
    while cursor < tokens.len() {
        let end = (cursor + max_tokens).min(tokens.len());
        let utterance = tokens[cursor..end].join(" ");
        out.push(CleanedUtterance {
            marker_label: None,
            text: utterance,
            token_start: cursor,
            token_end: end,
        });
        cursor = end;
    }
    out
}

fn push_utterance_span(
    source: &str,
    start: usize,
    end: usize,
    token_start: usize,
    spans: &mut Vec<CleanedUtterance>,
) -> usize {
    let utterance = source[start..end].trim();
    if utterance.is_empty() {
        return token_start;
    }
    let token_len = normalized_tokens(utterance).len();
    spans.push(CleanedUtterance {
        marker_label: marker_positions(utterance)
            .first()
            .map(|(label, _)| label.clone()),
        text: utterance.to_string(),
        token_start,
        token_end: token_start + token_len,
    });
    token_start + token_len
}

fn marker_utterance_spans(text: &str) -> Vec<CleanedUtterance> {
    let positions = marker_positions(text);
    positions
        .iter()
        .enumerate()
        .filter_map(|(index, (label, start))| {
            let end = positions
                .get(index + 1)
                .map(|(_, next_start)| *next_start)
                .unwrap_or(text.len());
            let utterance = text[*start..end].trim();
            if utterance.is_empty() {
                None
            } else {
                let token_start = normalized_tokens(&text[..*start]).len();
                let token_len = normalized_tokens(utterance).len();
                Some(CleanedUtterance {
                    marker_label: Some(label.clone()),
                    text: utterance.to_string(),
                    token_start,
                    token_end: token_start + token_len,
                })
            }
        })
        .collect()
}

fn marker_positions(text: &str) -> Vec<(String, usize)> {
    let lower = text.to_ascii_lowercase();
    let mut positions = Vec::new();
    for label in MARKER_LABELS {
        let needle = format!("{label} marker");
        let mut search_start = 0_usize;
        while let Some(offset) = lower[search_start..].find(&needle) {
            let start = search_start + offset;
            if marker_boundary(&lower, start, needle.len()) {
                positions.push((label.to_string(), start));
            }
            search_start = start + needle.len();
        }
    }
    positions.sort_by_key(|(_, start)| *start);
    positions
}

fn marker_boundary(text: &str, start: usize, len: usize) -> bool {
    let before = start
        .checked_sub(1)
        .and_then(|index| text.as_bytes().get(index))
        .copied();
    let after = text.as_bytes().get(start + len).copied();
    !before.is_some_and(|byte| byte.is_ascii_alphanumeric())
        && !after.is_some_and(|byte| byte.is_ascii_alphanumeric())
}

fn text_has_marker_label(text: &str, label: &str) -> bool {
    let tokens = normalized_tokens(text);
    marker_positions(text)
        .iter()
        .any(|(candidate, _)| candidate == label)
        || tokens.iter().any(|token| token == label)
        || marker_label_equivalent(label, &tokens)
}

fn marker_label_equivalent(label: &str, tokens: &[String]) -> bool {
    match label {
        "kilo" => tokens.iter().any(|token| token == "keyhole"),
        "mike" => tokens.iter().any(|token| token == "mic"),
        "papa" => tokens
            .windows(2)
            .any(|window| window[0] == "pop" && window[1] == "a"),
        _ => false,
    }
}

fn choose_take(
    cleaned_words: &[String],
    cursor: usize,
    expected_take: usize,
    remaining_segments: usize,
    realtime_text: &str,
) -> usize {
    let remaining_words = cleaned_words.len().saturating_sub(cursor);
    if remaining_words <= remaining_segments {
        return 1.min(remaining_words);
    }
    let max_take = remaining_words
        .saturating_sub(remaining_segments)
        .min(expected_take.saturating_mul(2).saturating_add(8))
        .max(1);
    let min_take = 1;
    let mut best_take = expected_take.min(max_take).max(min_take);
    let mut best_similarity = -1.0_f64;
    for take in min_take..=max_take {
        let text = cleaned_words[cursor..cursor + take].join(" ");
        let similarity = phrase_similarity(realtime_text, &text);
        if similarity > best_similarity
            || ((similarity - best_similarity).abs() < f64::EPSILON
                && take.abs_diff(expected_take) < best_take.abs_diff(expected_take))
        {
            best_similarity = similarity;
            best_take = take;
        }
    }
    best_take
}
