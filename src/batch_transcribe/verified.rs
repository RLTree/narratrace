pub(crate) fn verified_batch_for_cleanup(
    args: &Args,
    session_dir: &Path,
) -> Result<VerifiedBatchTranscript> {
    verified_batch_from_receipt(session_dir, Some(&args.batch_transcription_model))
}

pub(crate) fn verified_batch_for_alignment(session_dir: &Path) -> Result<VerifiedBatchTranscript> {
    verified_batch_from_receipt(session_dir, None)
}

fn verified_batch_from_receipt(
    session_dir: &Path,
    expected_model: Option<&str>,
) -> Result<VerifiedBatchTranscript> {
    let receipt_path = session_dir.join("batch-transcription-receipt.json");
    let receipt_bytes = read_regular_bytes_bounded(&receipt_path, MAX_JSON_ARTIFACT_BYTES)
        .context("batch transcript requires a current completed receipt")?;
    let receipt: Value = serde_json::from_slice(&receipt_bytes)?;
    if receipt.get("schema").and_then(Value::as_str) != Some(BATCH_RECEIPT_SCHEMA)
        || receipt.get("status").and_then(Value::as_str) != Some("completed")
    {
        bail!("batch transcript receipt is not current and completed");
    }
    let model = parse_required_str(&receipt, "model")?;
    if expected_model.is_some_and(|expected| expected != model) {
        bail!("batch transcript model does not match current configuration");
    }
    let prompt = build_prompt(None, None);
    let source_kind = parse_required_str(&receipt, "sourceKind")?;
    let expected = match source_kind {
        "retained-audio" => {
            let audio = open_current_audio(session_dir)?;
            BatchBinding::for_audio(session_dir, model, &prompt, &audio.sha256, audio.len)?
        }
        #[cfg(test)]
        "test-fixture" => BatchBinding {
            session_identity: session_identity(session_dir)?,
            model: model.to_string(),
            config_sha256: batch_config_sha256(model, &prompt),
            source_kind: source_kind.to_string(),
            source_sha256: parse_required_str(&receipt, "sourceSha256")?.to_string(),
            source_bytes: receipt
                .get("sourceBytes")
                .and_then(Value::as_u64)
                .ok_or_else(|| anyhow::anyhow!("missing or invalid sourceBytes"))?,
            consent_scope: BATCH_FIXTURE_SCOPE.to_string(),
        },
        _ => bail!("batch transcript source kind is not authorized"),
    };
    verified_cached_batch(session_dir, &expected)?
        .ok_or_else(|| anyhow::anyhow!("batch transcript cache binding is stale or mismatched"))
}
