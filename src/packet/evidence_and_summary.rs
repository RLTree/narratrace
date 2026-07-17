fn write_evidence_boundary_report(
    session_dir: &Path,
    temporal: &timeline::TemporalArtifacts,
    packet_path: &Path,
    voice_path: &Path,
    recording_metadata: Option<&str>,
    recording_events: Option<&str>,
) -> Result<PathBuf> {
    let context = read_json(temporal.context_path.clone()).unwrap_or(Value::Null);
    let redaction_status = context
        .pointer("/redactionPolicy/status")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let claim_ceiling = context
        .pointer("/alignmentDiagnostics/claimCeiling")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let missing_audio_clock = context
        .pointer("/alignmentDiagnostics/missingAudioClock")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let metadata_artifact = external_artifact(recording_metadata);
    let events_artifact = external_artifact(recording_events);
    let final_alignment =
        read_json(session_dir.join("final-transcript-alignment.json")).unwrap_or(Value::Null);
    let batch_transcript = external_artifact(Some(
        &session_dir
            .join("batch-transcript.json")
            .display()
            .to_string(),
    ));
    let cleaned_transcript = external_artifact(Some(
        &session_dir
            .join("cleaned-transcript.json")
            .display()
            .to_string(),
    ));
    let retained_audio_path = retained_audio_artifact_path(session_dir);
    let retained_audio = external_artifact(Some(&retained_audio_path.display().to_string()));
    let report_path = session_dir.join("evidence-boundary-report.json");
    write_private(
        &report_path,
        serde_json::to_string_pretty(&serde_json::json!({
            "schema": "narrated-record-replay.evidence-boundary-report.v1",
            "claimIds": ["CLAIM-010", "CLAIM-011"],
            "claimCeiling": "fixture-or-local-packet-only; real non-toy packet inspection still owed",
            "alignmentClaimCeiling": claim_ceiling,
            "evidenceSurfaces": {
                "recordReplayMetadataProvided": recording_metadata.is_some(),
                "recordReplayEventsProvided": recording_events.is_some(),
                "audioClockPresent": !missing_audio_clock,
                "transcriptSegments": temporal.transcript_segment_count,
                "recordReplayEvents": temporal.rnr_event_count,
                "alignedSegments": temporal.alignment_count,
                "conflictWarnings": temporal.conflict_count,
                "redactionStatus": redaction_status,
                "transcriptSourcePreference": "aligned final cleaned transcript when present; realtime raw remains timing spine",
                "finalTranscriptAlignmentStatus": final_alignment.get("status").and_then(Value::as_str).unwrap_or("not-generated"),
                "finalTranscriptUnresolvedMismatches": final_alignment.get("unresolvedMismatches").and_then(Value::as_u64).unwrap_or(0),
                "recordReplayArtifacts": {
                    "metadata": metadata_artifact,
                    "events": events_artifact
                },
                "transcriptArtifacts": {
                    "realtimeRawTimeline": external_artifact(Some(&session_dir.join("transcript.timeline.jsonl").display().to_string())),
                    "batchRaw": batch_transcript,
                    "cleaned": cleaned_transcript,
                    "alignedFinal": external_artifact(Some(&session_dir.join("final-transcript-alignment.json").display().to_string())),
                    "retainedAudio": retained_audio
                }
            },
            "artifactPaths": {
                "packet": packet_path.display().to_string(),
                "temporalContext": temporal.context_path.display().to_string(),
                "timestampedNotes": temporal.notes_path.display().to_string(),
                "thoughtProcess": session_dir.join("thought-process.md").display().to_string(),
                "replayVoiceParameters": voice_path.display().to_string(),
                "batchTranscript": session_dir.join("batch-transcript.json").display().to_string(),
                "cleanedTranscript": session_dir.join("cleaned-transcript.json").display().to_string(),
                "finalTranscriptAlignment": session_dir.join("final-transcript-alignment.json").display().to_string(),
                "audioRetention": session_dir.join("audio-retention.json").display().to_string()
            },
            "requiredReview": [
                "Confirm transcript-derived claims are not treated as UI evidence.",
                "Inspect conflict warnings before converting narration into replay steps.",
                "Inspect generated artifacts for raw-private leakage before sharing.",
                "Inspect final transcript alignment confidence and mismatches before treating cleaned text as final.",
                "Confirm retained audio remains local-private and is not copied into generated packets.",
                "Confirm Record & Replay metadata and events come from the same live run before closing live capture claims."
            ],
            "unsupportedClaims": [
                "Live narrated capture is not proven by this report alone.",
                "Packet usefulness is not proven until a real non-toy workflow packet is inspected.",
                "Review UI quality is not proven by static artifact generation alone.",
                "Replay voice behavior is not proven by planned parameter artifacts."
            ]
        }))?,
    )?;
    Ok(report_path)
}

fn external_artifact(path: Option<&str>) -> Value {
    let metadata = path.and_then(|path| regular_file_metadata(Path::new(path)).ok());
    let exists = metadata.is_some();
    let is_file = metadata.as_ref().is_some_and(|metadata| metadata.is_file());
    let bytes = metadata.as_ref().map(std::fs::Metadata::len);
    let non_empty = bytes.is_some_and(|bytes| bytes > 0);
    serde_json::json!({
        "path": path,
        "provided": path.is_some(),
        "exists": exists,
        "isFile": is_file,
        "bytes": bytes,
        "nonEmpty": non_empty,
        "safeRegularFile": is_file,
        "usableForLiveProof": path.is_some() && is_file && non_empty
    })
}

struct NarrationQuality {
    word_count: usize,
    char_count: usize,
    status: &'static str,
    warning: &'static str,
}

fn narration_quality(context: &Value) -> NarrationQuality {
    let segments = context
        .get("transcriptSegments")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let event_count = context
        .get("recordReplayEvents")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0);
    let text = segments
        .iter()
        .filter_map(timeline::consume_transcript_segment_text)
        .collect::<Vec<_>>()
        .join(" ");
    let word_count = text.split_whitespace().count();
    let char_count = text.chars().count();
    let sparse_for_ui = event_count >= 10 && (segments.len() < 2 || word_count < 30);
    let status = if sparse_for_ui || word_count < 10 {
        "too-sparse-for-non-toy-replay"
    } else if word_count < 50 {
        "needs-operator-distillation"
    } else {
        "sufficient-for-operator-review"
    };
    let warning = match status {
        "too-sparse-for-non-toy-replay" => {
            "Narration is too sparse for confident non-toy replay refinement."
        }
        "needs-operator-distillation" => {
            "Narration needs operator distillation before durable skill edits."
        }
        _ => "No narration-density warning from automated checks.",
    };
    NarrationQuality {
        word_count,
        char_count,
        status,
        warning,
    }
}

pub fn update_thought_process(session_dir: &Path) -> Result<()> {
    let segments = timeline::transcript_segments(session_dir);
    let transcript_timeline_path = session_dir.join("transcript.timeline.jsonl");
    let transcript_events_path = session_dir.join("transcript.events.jsonl");
    let transcript_final_path = session_dir.join("transcript.final.txt");
    let transcript_live_path = session_dir.join("transcript.live.txt");
    let transcript_timeline =
        render_markdown_path("Transcript timeline", &transcript_timeline_path);
    let transcript_events = render_markdown_path("Transcript events", &transcript_events_path);
    let transcript_final = render_markdown_path("Final transcript", &transcript_final_path);
    let transcript_live = render_markdown_path("Live transcript", &transcript_live_path);
    write_private(
        session_dir.join("thought-process.md"),
        format!(
            "---\nlast_edited: 2026-06-15\n---\n\n# Narrated Thought Process Boundary\n\nSource: live microphone transcription with {MODEL}.\n\nEvidence boundary: transcript text is Tree's spoken context and remains local-private by default. Use it to refine workflow intent, variable inputs, decision criteria, and verification only after operator review. It is not proof that an action happened on screen.\n\n## Local-Private Transcript Artifacts\n\n- Transcript segments: {}\n- {transcript_timeline}\n- {transcript_events}\n- {transcript_final}\n- {transcript_live}\n\n## Durable Handling Rule\n\nDo not copy raw transcript lines into skill files, review packets, memory, or plugin-distributed artifacts. Distill only the minimum reviewed intent needed for replay behavior.\n",
            segments.len(),
        ),
    )?;
    Ok(())
}

fn read_json(path: PathBuf) -> Result<Value> {
    regular_file_metadata(&path)?;
    Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
}
