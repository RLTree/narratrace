struct MonotonicPolicy {
    status: &'static str,
    segment_count: usize,
    claim_ceiling: &'static str,
}

fn monotonic_clock_policy(segments: &[TranscriptSegment]) -> MonotonicPolicy {
    let segment_count = segments
        .iter()
        .filter(|segment| segment.monotonic_offset_ms.is_some())
        .count();
    if segment_count == 0 {
        MonotonicPolicy {
            status: "not-captured",
            segment_count,
            claim_ceiling: "wall-clock alignment only; monotonic drift proof is still owed",
        }
    } else {
        MonotonicPolicy {
            status: "process-local-offsets-captured",
            segment_count,
            claim_ceiling: "process-local transcript offsets captured; Record & Replay cross-process monotonic drift proof is still owed",
        }
    }
}

fn timestamped_notes_markdown(
    segments: &[TranscriptSegment],
    events: &[RnrEvent],
    audio_started_at_unix_ms: Option<u64>,
) -> String {
    let mut out = String::from(
        "---\nlast_edited: 2026-06-15\n---\n\n# Timestamped Narration Notes\n\nEvidence boundary: transcript text is spoken context. Nearby UI events are timestamp-window alignments, not proof that the spoken text caused those actions.\n\n",
    );
    if segments.is_empty() {
        out.push_str("_No timestamped transcript segments captured._\n");
        return out;
    }
    for segment in segments {
        let transcript = render_untrusted_markdown("Transcript segment", segment.text.trim());
        out.push_str(&format!(
            "- [{}-{}] {}\n",
            format_offset(segment.start_ms),
            format_offset(segment.end_ms),
            transcript
        ));
        if let Some(audio_start) = audio_started_at_unix_ms {
            let midpoint = audio_start as i64 + ((segment.start_ms + segment.end_ms) / 2) as i64;
            for event in nearby_events(events, midpoint).into_iter().take(4) {
                let kind = render_untrusted_markdown("Event kind", &event.kind);
                let app = render_untrusted_markdown(
                    "Event app",
                    event.app.as_deref().unwrap_or("unknown app"),
                );
                let window = render_untrusted_markdown(
                    "Event window",
                    event.window.as_deref().unwrap_or("unknown window"),
                );
                out.push_str(&format!("  - Nearby UI: {kind}; {app}; {window}\n"));
            }
        }
    }
    out
}

fn nearby_events(events: &[RnrEvent], unix_ms: i64) -> Vec<&RnrEvent> {
    events
        .iter()
        .filter(|event| {
            event
                .unix_ms
                .map(|event_ms| (event_ms - unix_ms).abs() <= ALIGNMENT_WINDOW_MS)
                .unwrap_or(false)
        })
        .collect()
}

fn read_recording_metadata(value: Option<&str>) -> Result<Value> {
    let Some(path) = value.and_then(safe_existing_file) else {
        return Ok(Value::Null);
    };
    let text = ingestion::read_bounded(
        &path,
        "recording metadata artifact",
        ingestion::METADATA_MAX_BYTES,
    )?;
    Ok(serde_json::from_str(&text).unwrap_or(Value::Null))
}

fn read_json(path: &Path) -> Result<Value> {
    let text = ingestion::read_bounded(
        path,
        "capture clock artifact",
        ingestion::METADATA_MAX_BYTES,
    )?;
    Ok(serde_json::from_str(&text)?)
}

fn existing_path(value: &str) -> Option<&Path> {
    let path = Path::new(value);
    regular_file_metadata(path).ok()?;
    Some(path)
}

fn safe_existing_file(value: &str) -> Option<PathBuf> {
    let path = Path::new(value);
    regular_file_metadata(path).ok()?;
    Some(path.to_path_buf())
}
