use crate::batch_transcribe;
use crate::config::{Args, DEFAULT_CLEANUP_MODEL};
use crate::private_fs::write_private;
use crate::safe_path::{open_regular_file, regular_file_metadata};
use anyhow::{Context, Result, bail};
use reqwest::blocking::Client;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
#[cfg(test)]
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Duration;

const API_URL: &str = "https://api.openai.com/v1/responses";
const DEFAULT_CLEANUP_FALLBACK_MODEL: &str = "gpt-5-mini";
const MAX_JSON_ARTIFACT_BYTES: u64 = 16 * 1024 * 1024;
const MAX_RESPONSE_BYTES: u64 = 16 * 1024 * 1024;

pub fn ensure_cleaned_transcript(args: &Args, session_dir: &Path) -> Result<Option<PathBuf>> {
    let output_path = session_dir.join("cleaned-transcript.json");
    if !args.cleanup_enabled {
        write_cleanup_disabled(session_dir, "disabled-by-config")?;
        return Ok(None);
    }
    if regular_file_metadata(&session_dir.join("batch-transcript.json")).is_err() {
        write_cleanup_disabled(session_dir, "missing-batch-transcript")?;
        return Ok(None);
    }
    let batch = batch_transcribe::verified_batch_for_cleanup(args, session_dir)?;
    let dictionary = build_dictionary(args, session_dir);
    if verified_cached_cleanup(args, session_dir, &batch, &dictionary)?.is_some() {
        return Ok(Some(output_path));
    }
    let model_input = cleanup_model_input(&dictionary, &batch.text)?;

    #[cfg(test)]
    if let Some(fixture) = load_test_fixture(
        session_dir,
        "NARRATED_REPLAY_CLEANUP_TRANSCRIPT_FIXTURE",
        "cleanup transcript fixture",
    )? {
        let suggested = cleanup_text(&fixture.value);
        let validation = validate_cleanup_output(&batch.text, &suggested, &dictionary);
        let binding = CleanupBinding::new(
            session_dir,
            args,
            &batch,
            &dictionary,
            cleanup_model_name(&args.cleanup_model),
            "fixture",
            &fixture.sha256,
            &validation,
        )?;
        write_cleanup_artifact(
            session_dir,
            &output_path,
            &batch,
            &dictionary,
            &model_input,
            suggested,
            fixture.value,
            None,
            &binding,
            &validation,
        )?;
        return Ok(Some(output_path));
    }

    require_cleanup_consent(args)?;
    let api_key =
        std::env::var("OPENAI_API_KEY").context("OPENAI_API_KEY is required for cleanup")?;
    let client = Client::builder()
        .timeout(Duration::from_secs(120))
        .build()?;
    let mut failures = Vec::new();
    for model in cleanup_model_candidates(&args.cleanup_model) {
        match call_cleanup_api(&client, &api_key, model, &model_input) {
            Ok(value) => {
                let suggested = cleanup_text(&value);
                let validation = validate_cleanup_output(&batch.text, &suggested, &dictionary);
                let response_sha256 = sha256_bytes(serde_json::to_string(&value)?.as_bytes());
                let binding = CleanupBinding::new(
                    session_dir,
                    args,
                    &batch,
                    &dictionary,
                    model,
                    "openai-responses",
                    &response_sha256,
                    &validation,
                )?;
                write_cleanup_artifact(
                    session_dir,
                    &output_path,
                    &batch,
                    &dictionary,
                    &model_input,
                    suggested,
                    value,
                    fallback_note(&args.cleanup_model, model),
                    &binding,
                    &validation,
                )?;
                return Ok(Some(output_path));
            }
            Err(error) => failures.push(format!("{model}: {error}")),
        }
    }
    bail!(
        "cleanup failed for all configured models: {}",
        failures.join(" | ")
    )
}

fn require_cleanup_consent(args: &Args) -> Result<()> {
    if !args.openai_postprocessing_consent {
        bail!(
            "--i-consent-to-openai-postprocessing is required before sending the current-session batch transcript to OpenAI"
        );
    }
    Ok(())
}
