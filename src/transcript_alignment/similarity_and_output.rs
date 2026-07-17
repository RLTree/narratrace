pub fn phrase_similarity(left: &str, right: &str) -> f64 {
    let left = normalized_tokens(left);
    let right = normalized_tokens(right);
    if left.is_empty() && right.is_empty() {
        return 1.0;
    }
    if left.is_empty() || right.is_empty() {
        return 0.0;
    }
    let mut matches = 0_usize;
    for token in &left {
        if right
            .iter()
            .any(|candidate| token_similar(token, candidate))
        {
            matches += 1;
        }
    }
    matches as f64 / left.len() as f64
}

fn token_similar(left: &str, right: &str) -> bool {
    left == right
        || canonical_equivalent(left, right)
        || (left.len() > 5
            && right.len() > 5
            && left.len().abs_diff(right.len()) <= 2
            && bounded_edit_distance(left.as_bytes(), right.as_bytes(), 2) <= 2)
}

fn canonical_equivalent(left: &str, right: &str) -> bool {
    matches!(
        (left, right),
        ("cloud", "claude")
            | ("claude", "cloud")
            | ("chatgpt", "chat")
            | ("chat", "chatgpt")
            | ("atlas", "atlases")
            | ("atlases", "atlas")
            | ("tinga", "tinka")
            | ("kilo", "keyhole")
            | ("keyhole", "kilo")
            | ("mike", "mic")
            | ("mic", "mike")
            | ("papa", "pop")
            | ("pop", "papa")
            | ("vesper", "fesper")
            | ("fesper", "vesper")
    )
}

fn normalized_tokens(text: &str) -> Vec<String> {
    text.split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|part| !part.is_empty())
        .map(|part| part.to_ascii_lowercase())
        .collect()
}

fn split_words(text: &str) -> Vec<String> {
    text.split_whitespace().map(str::to_string).collect()
}

fn semantic_word_count(text: &str) -> usize {
    let tokens = normalized_tokens(text);
    let mut count = 0_usize;
    let mut index = 0_usize;
    while index < tokens.len() {
        if tokens[index] == "chat"
            && tokens.get(index + 1).is_some_and(|token| token == "g")
            && tokens.get(index + 2).is_some_and(|token| token == "p")
            && tokens.get(index + 3).is_some_and(|token| token == "t")
        {
            count += 1;
            index += 4;
        } else {
            count += 1;
            index += 1;
        }
    }
    count
}

fn bounded_edit_distance(left: &[u8], right: &[u8], limit: usize) -> usize {
    if left.len().abs_diff(right.len()) > limit {
        return limit + 1;
    }
    let mut costs: Vec<usize> = (0..=right.len()).collect();
    for (i, left_char) in left.iter().enumerate() {
        let mut last = i;
        costs[0] = i + 1;
        let mut row_min = costs[0];
        for (j, right_char) in right.iter().enumerate() {
            let old = costs[j + 1];
            costs[j + 1] = if left_char == right_char {
                last
            } else {
                1 + last.min(costs[j]).min(costs[j + 1])
            };
            row_min = row_min.min(costs[j + 1]);
            last = old;
        }
        if row_min > limit {
            return limit + 1;
        }
    }
    *costs.last().unwrap_or(&usize::MAX)
}

fn final_segment_json(segment: &FinalSegment) -> Value {
    json!({
        "id": segment.id,
        "startMs": segment.start_ms,
        "endMs": segment.end_ms,
        "text": redact_text(&segment.text),
        "confidence": segment.confidence,
        "sourceRealtimeSegmentIds": segment.source_realtime_ids,
        "mismatch": segment.mismatch,
        "source": "aligned-final-cleaned",
        "alignmentStrategy": if segment.source_realtime_ids.len() > 1 {
            "monotonic-token-window"
        } else {
            "monotonic-token-single-window"
        }
    })
}

fn write_final_timeline(session_dir: &Path, segments: &[FinalSegment]) -> Result<()> {
    let path = session_dir.join("final-transcript.timeline.jsonl");
    let mut out = String::new();
    for segment in segments {
        out.push_str(&format!(
            "{}\n",
            serde_json::to_string(&final_segment_json(segment))?
        ));
    }
    write_private(path, out)
}

fn write_disabled(session_dir: &Path, reason: &str) -> Result<()> {
    write_private(
        session_dir.join("final-transcript-alignment-receipt.json"),
        serde_json::to_string_pretty(&json!({
            "schema": FINAL_ALIGNMENT_RECEIPT_SCHEMA,
            "status": "disabled",
            "alignmentPolicyVersion": FINAL_ALIGNMENT_POLICY_VERSION,
            "reason": reason
        }))?,
    )
}
