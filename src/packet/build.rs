use crate::config::{Args, MODEL, required_session_dir};
use crate::private_fs::write_private;
use crate::review;
use crate::safe_path::regular_file_metadata;
use crate::timeline;
use crate::voice;
use crate::{batch_transcribe, transcript_alignment, transcript_cleanup};
use anyhow::Result;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

pub fn make_packet(args: &Args) -> Result<()> {
    let session_dir = required_session_dir(args)?;
    enforce_openai_postprocessing_consent(args, &session_dir)?;
    update_thought_process(&session_dir)?;
    let manifest = read_json(session_dir.join("manifest.json")).unwrap_or(Value::Null);
    let status = read_json(session_dir.join("status.json")).unwrap_or(Value::Null);
    let goal = render_untrusted_markdown(
        "Goal",
        manifest
            .get("goal")
            .and_then(Value::as_str)
            .unwrap_or("not recorded"),
    );
    let model = render_untrusted_markdown(
        "Narration model",
        status.get("model").and_then(Value::as_str).unwrap_or(MODEL),
    );
    let metadata = render_untrusted_markdown(
        "Record & Replay metadata",
        args.recording_metadata.as_deref().unwrap_or("not-provided"),
    );
    let events = render_untrusted_markdown(
        "Record & Replay events",
        args.recording_events.as_deref().unwrap_or("not-provided"),
    );
    let batch_path = batch_transcribe::ensure_batch_transcript(
        args,
        &session_dir,
        args.recording_metadata.as_deref(),
        args.recording_events.as_deref(),
    )?;
    let cleanup_path = transcript_cleanup::ensure_cleaned_transcript(args, &session_dir)?;
    let final_alignment_path = transcript_alignment::ensure_final_alignment(&session_dir)?;
    let temporal = timeline::build_temporal_context(
        &session_dir,
        args.recording_metadata.as_deref(),
        args.recording_events.as_deref(),
    )?;
    let thought_path = session_dir.join("thought-process.md");
    let voice_artifact =
        voice::write_replay_voice_parameters(&session_dir, &temporal.context_path, args)?;
    let packet_path = session_dir.join("skill-refinement-packet.md");
    let evidence_report_path = write_evidence_boundary_report(
        &session_dir,
        &temporal,
        &packet_path,
        &voice_artifact.path,
        args.recording_metadata.as_deref(),
        args.recording_events.as_deref(),
    )?;
    let temporal_context = read_json(temporal.context_path.clone()).unwrap_or(Value::Null);
    let quality = narration_quality(&temporal_context);
    let session = render_markdown_path("Narration session", &session_dir);
    let transcript_events = render_markdown_path(
        "Transcript events",
        &session_dir.join("transcript.events.jsonl"),
    );
    let timestamped_transcript =
        render_markdown_path("Timestamped transcript", &temporal.notes_path);
    let temporal_context_path =
        render_markdown_path("Temporal context packet", &temporal.context_path);
    let thought = render_markdown_path("Thought process", &thought_path);
    let voice = render_markdown_path("Replay voice parameters", &voice_artifact.path);
    let evidence = render_markdown_path("Evidence boundary report", &evidence_report_path);
    let batch = render_optional_markdown_path("Batch transcript", batch_path.as_deref());
    let cleanup = render_optional_markdown_path("Cleaned transcript", cleanup_path.as_deref());
    let final_alignment =
        render_optional_markdown_path("Final aligned transcript", final_alignment_path.as_deref());
    write_private(
        &packet_path,
        format!(
            "---\nlast_edited: 2026-06-15\n---\n\n# Narrated Record & Replay Skill Refinement Packet\n\n{goal}\n\n## Capture Artifacts\n\n- {metadata}\n- {events}\n- {session}\n- {model}\n- {transcript_events}\n- {timestamped_transcript}\n- {temporal_context_path}\n- {thought}\n- {voice}\n- {evidence}\n- {batch}\n- {cleanup}\n- {final_alignment}\n\n## Evidence Boundary\n\nUse Record & Replay artifacts as observed UI/action evidence. Realtime transcript events are the timing spine. Batch plus cleanup artifacts are the preferred word authority when final alignment exists. Use the temporal context packet to pair reviewed spoken context with nearby UI events. If these conflict, prefer observed UI evidence for action order and use transcript artifacts to clarify intent. Treat replay voice parameters as planned replay instructions, not proof that replay executed with voice.\n\n## Refinement Instructions\n\n1. Preserve the demonstrated workflow's stable action sequence.\n2. Add variable inputs, hidden preferences, naming conventions, decision points, and success criteria only after reviewing local-private transcript artifacts.\n3. Use the temporal context packet to pair reviewed spoken context with nearby UI events before creating or editing skills.\n4. Mark timestamp-window alignments as inferred context unless the UI event directly proves the claim.\n5. Remove raw transcript detail that is not needed for replay.\n6. Add verification steps that prove the future replay produced the intended result.\n7. Do not store raw audio or raw transcript in durable memory by default.\n8. Inspect the evidence boundary report before converting transcript-derived claims into durable skill steps.\n9. Inspect final-transcript-alignment.json for low-confidence or unresolved mismatches before trusting cleaned words.\n\n## Temporal Alignment Summary\n\n- Transcript segments: {}\n- Record & Replay events: {}\n- Aligned segments: {}\n- Transcript action claims needing UI evidence: {}\n- Replay voice segment bindings: {}\n\n## Narration Quality Summary\n\n- Transcript words: {}\n- Transcript characters: {}\n- Narration density status: {}\n- Usefulness warning: {}\n\n## Transcript Review Boundary\n\nRaw transcript text and retained audio are intentionally not embedded in this packet. Use the local-private transcript artifacts listed above for operator review, then distill only necessary intent, variables, decision criteria, and verification requirements into durable skill edits.\n\n- Transcript segments available for review: {}\n- {thought}\n",
            temporal.transcript_segment_count,
            temporal.rnr_event_count,
            temporal.alignment_count,
            temporal.conflict_count,
            voice_artifact.segment_binding_count,
            quality.word_count,
            quality.char_count,
            quality.status,
            quality.warning,
            temporal.transcript_segment_count,
        ),
    )?;
    let review_artifact = review::write_review_artifact(
        &session_dir,
        &temporal.context_path,
        Some(&packet_path),
        Some(&voice_artifact.path),
        None,
        Some(&evidence_report_path),
        None,
        None,
    )?;
    println!(
        "{}",
        serde_json::json!({ "packetPath": packet_path, "reviewPath": review_artifact.html_path, "reviewContractPath": review_artifact.contract_path, "replayVoiceParametersPath": voice_artifact.path, "evidenceBoundaryReportPath": evidence_report_path, "sessionDir": session_dir })
    );
    Ok(())
}

fn enforce_openai_postprocessing_consent(args: &Args, session_dir: &Path) -> Result<()> {
    if args.openai_postprocessing_consent {
        return Ok(());
    }
    let batch_fixture_present = std::env::var("NARRATED_REPLAY_BATCH_TRANSCRIPT_FIXTURE").is_ok();
    let cleanup_fixture_present =
        std::env::var("NARRATED_REPLAY_CLEANUP_TRANSCRIPT_FIXTURE").is_ok();
    if openai_postprocessing_would_send_private_material(
        args,
        session_dir,
        batch_fixture_present,
        cleanup_fixture_present,
    ) {
        anyhow::bail!(
            "--i-consent-to-openai-postprocessing is required before packet sends retained audio or transcript text to OpenAI; use --disable-batch-transcription and --disable-cleanup for local-only packet generation"
        );
    }
    Ok(())
}

fn openai_postprocessing_would_send_private_material(
    args: &Args,
    session_dir: &Path,
    batch_fixture_present: bool,
    cleanup_fixture_present: bool,
) -> bool {
    let batch_path = session_dir.join("batch-transcript.json");
    let cleaned_path = session_dir.join("cleaned-transcript.json");
    let batch_exists = regular_file_metadata(&batch_path).is_ok();
    let cleaned_exists = regular_file_metadata(&cleaned_path).is_ok();
    let retained_audio_exists = retained_audio_private_material_exists(session_dir);
    let batch_would_send_audio = args.batch_transcription_enabled
        && !batch_exists
        && !batch_fixture_present
        && retained_audio_exists;
    let cleanup_has_batch_text = batch_exists || batch_fixture_present;
    let cleanup_would_send_text = args.cleanup_enabled
        && !cleaned_exists
        && !cleanup_fixture_present
        && cleanup_has_batch_text;
    batch_would_send_audio || cleanup_would_send_text
}

fn retained_audio_private_material_exists(session_dir: &Path) -> bool {
    if regular_file_metadata(&session_dir.join("retained-audio.wav")).is_ok() {
        return true;
    }
    let manifest_path = session_dir.join("audio-retention.json");
    let Ok(manifest) = read_json(manifest_path.clone()) else {
        return fs::symlink_metadata(manifest_path).is_ok();
    };
    let Some(audio_path) = manifest.get("audioPath").and_then(Value::as_str) else {
        return false;
    };
    regular_file_metadata(Path::new(audio_path)).is_ok()
}

fn retained_audio_artifact_path(session_dir: &Path) -> PathBuf {
    if let Ok(manifest) = read_json(session_dir.join("audio-retention.json"))
        && let Some(audio_path) = manifest.get("audioPath").and_then(Value::as_str)
    {
        return PathBuf::from(audio_path);
    }
    session_dir.join("retained-audio.wav")
}
