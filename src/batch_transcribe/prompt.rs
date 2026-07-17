use crate::config::Args;
use crate::private_fs::write_private;
use crate::safe_path::{open_regular_file, regular_file_metadata};
use anyhow::{Context, Result, bail};
use reqwest::blocking::{Client, multipart};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
#[cfg(test)]
use std::fs;
use std::fs::File;
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::Duration;

const API_URL: &str = "https://api.openai.com/v1/audio/transcriptions";
const MAX_AUDIO_UPLOAD_BYTES: usize = 20 * 1024 * 1024;
const MAX_RETAINED_AUDIO_BYTES: u64 = 512 * 1024 * 1024;
const MAX_CONTEXT_BYTES: u64 = 4 * 1024 * 1024;
const MAX_JSON_ARTIFACT_BYTES: u64 = 16 * 1024 * 1024;
const MAX_RESPONSE_BYTES: u64 = 16 * 1024 * 1024;
const WAV_HEADER_BYTES: usize = 44;

pub fn ensure_batch_transcript(
    args: &Args,
    session_dir: &Path,
    metadata_path: Option<&str>,
    events_path: Option<&str>,
) -> Result<Option<PathBuf>> {
    let output_path = session_dir.join("batch-transcript.json");
    if !args.batch_transcription_enabled {
        write_disabled(session_dir, "disabled-by-config")?;
        return Ok(None);
    }
    let prompt = build_prompt(metadata_path, events_path);
    #[cfg(test)]
    if let Some(fixture) = load_test_fixture(
        session_dir,
        "NARRATED_REPLAY_BATCH_TRANSCRIPT_FIXTURE",
        "batch transcript fixture",
    )? {
        let binding = BatchBinding::for_fixture(
            session_dir,
            &args.batch_transcription_model,
            &prompt,
            &fixture.sha256,
            fixture.bytes,
        )?;
        if verified_cached_batch(session_dir, &binding)?.is_some() {
            return Ok(Some(output_path));
        }
        write_batch_artifact(
            session_dir,
            &output_path,
            &args.batch_transcription_model,
            &prompt,
            "fixture",
            fixture.value,
            &binding,
        )?;
        return Ok(Some(output_path));
    }

    let opened = match open_current_audio(session_dir) {
        Ok(audio) => audio,
        Err(error) => {
            write_disabled(session_dir, &format!("missing-audio-retention: {error}"))?;
            return Ok(None);
        }
    };
    let binding = BatchBinding::for_audio(
        session_dir,
        &args.batch_transcription_model,
        &prompt,
        &opened.sha256,
        opened.len,
    )?;
    if verified_cached_batch(session_dir, &binding)?.is_some() {
        return Ok(Some(output_path));
    }
    let approved = opened.approve(args)?;
    let api_key = std::env::var("OPENAI_API_KEY")
        .context("OPENAI_API_KEY is required for batch transcription")?;
    let client = Client::builder()
        .timeout(Duration::from_secs(180))
        .build()?;
    let values = transcribe_approved_audio(
        session_dir,
        &client,
        &api_key,
        &args.batch_transcription_model,
        &prompt,
        approved,
    )?;
    let value = combine_chunk_transcripts(&values, values.len());
    write_batch_artifact(
        session_dir,
        &output_path,
        &args.batch_transcription_model,
        &prompt,
        "openai-audio-transcriptions",
        value,
        &binding,
    )?;
    Ok(Some(output_path))
}

pub fn batch_text(value: &Value) -> String {
    value
        .pointer("/transcription/text")
        .and_then(Value::as_str)
        .or_else(|| value.pointer("/raw/text").and_then(Value::as_str))
        .unwrap_or("")
        .trim()
        .to_string()
}
