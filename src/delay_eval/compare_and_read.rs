struct DelayComparisonSummary {
    status: &'static str,
    recommendation: &'static str,
    reason: &'static str,
    deltas: Value,
}

fn compare_delay_evaluations(baseline: &Value, candidate: &Value) -> DelayComparisonSummary {
    let baseline_mismatches = u64_at(baseline, "/alignmentMetrics/unresolvedMismatches");
    let candidate_mismatches = u64_at(candidate, "/alignmentMetrics/unresolvedMismatches");
    let baseline_utterances = u64_at(baseline, "/alignmentMetrics/finalAlignedUtteranceCount");
    let candidate_utterances = u64_at(candidate, "/alignmentMetrics/finalAlignedUtteranceCount");
    let baseline_first_delta = i64_at(baseline, "/latencyMetrics/firstRealtimeDeltaLatencyMs");
    let candidate_first_delta = i64_at(candidate, "/latencyMetrics/firstRealtimeDeltaLatencyMs");
    let baseline_first_completed = i64_at(
        baseline,
        "/latencyMetrics/firstCompletedRealtimeSegmentLatencyMs",
    );
    let candidate_first_completed = i64_at(
        candidate,
        "/latencyMetrics/firstCompletedRealtimeSegmentLatencyMs",
    );
    let baseline_first_audio = i64_at(
        baseline,
        "/latencyMetrics/recordReplayStartToFirstAudioChunkMs",
    );
    let candidate_first_audio = i64_at(
        candidate,
        "/latencyMetrics/recordReplayStartToFirstAudioChunkMs",
    );
    let baseline_marker_recall = u64_at(baseline, "/alignmentMetrics/scriptedMarkerRecall/found");
    let candidate_marker_recall = u64_at(candidate, "/alignmentMetrics/scriptedMarkerRecall/found");
    let deltas = json!({
        "firstAudioChunkDeltaMs": optional_i64_delta(candidate_first_audio, baseline_first_audio),
        "firstRealtimeDeltaDeltaMs": optional_i64_delta(candidate_first_delta, baseline_first_delta),
        "firstCompletedRealtimeSegmentDeltaMs": optional_i64_delta(candidate_first_completed, baseline_first_completed),
        "finalAlignedUtteranceCountDelta": optional_u64_delta(candidate_utterances, baseline_utterances),
        "unresolvedMismatchDelta": optional_u64_delta(candidate_mismatches, baseline_mismatches),
        "diagnosticMarkerRecallDelta": optional_u64_delta(candidate_marker_recall, baseline_marker_recall)
    });
    let metrics_complete = [
        baseline_mismatches,
        candidate_mismatches,
        baseline_utterances,
        candidate_utterances,
        baseline_first_delta.map(|value| value as u64),
        candidate_first_delta.map(|value| value as u64),
        baseline_first_completed.map(|value| value as u64),
        candidate_first_completed.map(|value| value as u64),
    ]
    .iter()
    .all(Option::is_some);
    if !metrics_complete {
        return DelayComparisonSummary {
            status: "inconclusive",
            recommendation: "keep-current-default",
            reason: "required comparison metrics are missing",
            deltas,
        };
    }
    let quality_not_worse = candidate_mismatches <= baseline_mismatches
        && candidate_utterances >= baseline_utterances
        && candidate_status(candidate) == Some("aligned");
    let latency_better = candidate_first_delta < baseline_first_delta
        || candidate_first_completed < baseline_first_completed
        || candidate_first_audio < baseline_first_audio;
    if quality_not_worse && latency_better {
        DelayComparisonSummary {
            status: "candidate-lower-latency-needs-operator-review",
            recommendation: "operator-review-before-default-change",
            reason: "candidate latency improved without worse machine alignment metrics; transcript/action usefulness review is still required",
            deltas,
        }
    } else {
        DelayComparisonSummary {
            status: "keep-current-default",
            recommendation: "keep-current-default",
            reason: "candidate did not improve latency while preserving machine alignment metrics",
            deltas,
        }
    }
}

fn evaluation_summary(value: &Value, path: &Path) -> Value {
    json!({
        "path": path.display().to_string(),
        "realtimeDelay": value.get("realtimeDelay").and_then(Value::as_str),
        "latencyMetrics": value.get("latencyMetrics").cloned().unwrap_or(Value::Null),
        "alignmentMetrics": value.get("alignmentMetrics").cloned().unwrap_or(Value::Null),
        "recordReplayEventCount": value.pointer("/recordReplay/eventCount").and_then(Value::as_u64)
    })
}

fn candidate_status(value: &Value) -> Option<&str> {
    value
        .pointer("/alignmentMetrics/finalAlignmentStatus")
        .and_then(Value::as_str)
}

fn optional_i64_delta(candidate: Option<i64>, baseline: Option<i64>) -> Option<i64> {
    Some(candidate? - baseline?)
}

fn optional_u64_delta(candidate: Option<u64>, baseline: Option<u64>) -> Option<i64> {
    Some(candidate? as i64 - baseline? as i64)
}

fn u64_at(value: &Value, pointer: &str) -> Option<u64> {
    value.pointer(pointer).and_then(Value::as_u64)
}

fn i64_at(value: &Value, pointer: &str) -> Option<i64> {
    value.pointer(pointer).and_then(Value::as_i64).or_else(|| {
        value
            .pointer(pointer)
            .and_then(Value::as_u64)
            .map(|value| value as i64)
    })
}

#[derive(Debug, Clone)]
struct EventLatency {
    latency_ms: Option<u64>,
    source: &'static str,
}

fn first_event_latency(events: &[Value], kind: &str) -> EventLatency {
    let Some(event) = events
        .iter()
        .find(|event| event.get("kind").and_then(Value::as_str) == Some(kind))
    else {
        return EventLatency {
            latency_ms: None,
            source: "missing",
        };
    };
    if let Some(offset) = event.get("monotonicOffsetMs").and_then(Value::as_u64) {
        return EventLatency {
            latency_ms: Some(offset),
            source: "process-local-monotonic-offset",
        };
    }
    EventLatency {
        latency_ms: event.get("audioOffsetMs").and_then(Value::as_u64),
        source: "audio-wall-clock-offset",
    }
}

fn scripted_marker_recall(final_alignment: &Value) -> Value {
    let text = final_alignment
        .get("segments")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|segment| segment.get("text").and_then(Value::as_str))
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase();
    let found = SCRIPTED_MARKERS_ALPHA_QUEBEC
        .iter()
        .filter(|marker| text.contains(**marker))
        .count();
    json!({
        "expectedAlphaThroughQuebec": SCRIPTED_MARKERS_ALPHA_QUEBEC.len(),
        "found": found,
        "missing": SCRIPTED_MARKERS_ALPHA_QUEBEC.len().saturating_sub(found),
        "diagnosticOnly": true
    })
}

fn delta(value: Option<i64>, baseline: Option<i64>) -> Option<i64> {
    Some(value? - baseline?)
}

fn read_json(path: &Path) -> Result<Value> {
    let text = crate::safe_path::read_regular_text_bounded(path, MAX_DELAY_JSON_BYTES)?;
    Ok(serde_json::from_str(&text)?)
}

fn read_json_lines(path: &Path) -> Result<Vec<Value>> {
    if regular_file_metadata(path).is_err() {
        return Ok(Vec::new());
    }
    let text = crate::safe_path::read_regular_text_bounded(path, MAX_DELAY_JSONL_BYTES)?;
    let mut rows = Vec::new();
    for (index, line) in text.lines().enumerate() {
        if index >= MAX_DELAY_JSONL_ROWS {
            bail!("delay evaluation JSONL row limit exceeded");
        }
        if !line.trim().is_empty() {
            rows.push(serde_json::from_str::<Value>(line).with_context(|| {
                format!("malformed delay evaluation JSONL row {}", index + 1)
            })?);
        }
    }
    Ok(rows)
}

fn line_count(path: &Path) -> Result<u64> {
    use std::io::{BufRead, BufReader};
    let file = crate::safe_path::open_regular_file(path)?;
    if file.metadata()?.len() > MAX_DELAY_JSONL_BYTES {
        bail!("delay evaluation artifact byte limit exceeded");
    }
    let mut rows = 0_u64;
    for line in BufReader::new(file).lines() {
        line?;
        rows += 1;
        if rows > MAX_DELAY_JSONL_ROWS as u64 {
            bail!("delay evaluation JSONL row limit exceeded");
        }
    }
    Ok(rows)
}

fn parse_utc_timestamp_ms(value: &str) -> Result<i64> {
    crate::timeline::parse_utc_millis(value)
        .ok_or_else(|| anyhow::anyhow!("timestamp must be a valid UTC RFC3339 timestamp"))
}
