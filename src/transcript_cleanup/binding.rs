#[derive(Debug, Clone)]
struct CleanupBinding {
    session_identity: String,
    requested_model: String,
    used_model: String,
    config_sha256: String,
    batch_artifact_sha256: String,
    batch_receipt_sha256: String,
    dictionary_sha256: String,
    source: String,
    response_sha256: String,
    consent_scope: String,
    validation_status: String,
}

#[derive(Debug, Clone)]
pub(crate) struct VerifiedCleanedTranscript {
    pub text: String,
    pub artifact_sha256: String,
    pub receipt_sha256: String,
    pub batch_artifact_sha256: String,
    pub session_identity: String,
    pub validation_policy_version: String,
}

impl CleanupBinding {
    fn new(
        _session_dir: &Path,
        args: &Args,
        batch: &batch_transcribe::VerifiedBatchTranscript,
        dictionary: &[String],
        used_model: &str,
        source: &str,
        response_sha256: &str,
        validation: &CleanupValidation,
    ) -> Result<Self> {
        let consent_scope = if source == "fixture" {
            CLEANUP_FIXTURE_SCOPE
        } else {
            CLEANUP_CONSENT_SCOPE
        };
        Ok(Self {
            session_identity: batch.session_identity.clone(),
            requested_model: cleanup_model_name(&args.cleanup_model).to_string(),
            used_model: used_model.to_string(),
            config_sha256: cleanup_config_sha256(args, dictionary),
            batch_artifact_sha256: batch.artifact_sha256.clone(),
            batch_receipt_sha256: batch.receipt_sha256.clone(),
            dictionary_sha256: dictionary_sha256(dictionary),
            source: source.to_string(),
            response_sha256: response_sha256.to_string(),
            consent_scope: consent_scope.to_string(),
            validation_status: validation.status.to_string(),
        })
    }
}

fn sha256_bytes(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn read_regular_bytes_bounded(path: &Path, max_bytes: u64) -> Result<Vec<u8>> {
    let file = open_regular_file(path)?;
    let len = file.metadata()?.len();
    if len > max_bytes {
        bail!("artifact exceeds {max_bytes} byte limit");
    }
    let mut bytes = Vec::with_capacity(len as usize);
    file.take(max_bytes + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > max_bytes {
        bail!("artifact exceeds {max_bytes} byte limit");
    }
    Ok(bytes)
}

fn parse_required_str<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("missing or invalid {key}"))
}

fn cleanup_config_sha256(args: &Args, dictionary: &[String]) -> String {
    cleanup_config_sha256_for_model(cleanup_model_name(&args.cleanup_model), dictionary)
}

fn cleanup_config_sha256_for_model(model: &str, dictionary: &[String]) -> String {
    sha256_bytes(
        format!(
            "cleanup-config-v2\0{}\0{CLEANUP_POLICY_VERSION}\0{CLEANUP_VALIDATOR_VERSION}\0{}",
            model,
            dictionary_sha256(dictionary)
        )
        .as_bytes(),
    )
}

pub(crate) fn verified_cleaned_for_alignment(
    session_dir: &Path,
) -> Result<VerifiedCleanedTranscript> {
    let batch = batch_transcribe::verified_batch_for_alignment(session_dir)?;
    let receipt_path = session_dir.join("cleanup-receipt.json");
    let receipt_bytes = read_regular_bytes_bounded(&receipt_path, MAX_JSON_ARTIFACT_BYTES)
        .context("cleaned transcript requires a current completed receipt")?;
    let receipt: Value = serde_json::from_slice(&receipt_bytes)?;
    let requested_model = parse_required_str(&receipt, "requestedModel")?;
    let used_model = parse_required_str(&receipt, "usedModel")?;
    if !cleanup_model_candidates(requested_model).contains(&used_model) {
        bail!("cleanup receipt used model is outside the current fallback policy");
    }
    let source = parse_required_str(&receipt, "source")?;
    let consent_scope = match source {
        "openai-responses" => CLEANUP_CONSENT_SCOPE,
        #[cfg(test)]
        "fixture" => CLEANUP_FIXTURE_SCOPE,
        _ => bail!("cleanup receipt source is not authorized"),
    };
    let dictionary = static_cleanup_dictionary();
    let expected = CleanupBinding {
        session_identity: batch.session_identity.clone(),
        requested_model: requested_model.to_string(),
        used_model: used_model.to_string(),
        config_sha256: cleanup_config_sha256_for_model(requested_model, &dictionary),
        batch_artifact_sha256: batch.artifact_sha256.clone(),
        batch_receipt_sha256: batch.receipt_sha256.clone(),
        dictionary_sha256: dictionary_sha256(&dictionary),
        source: source.to_string(),
        response_sha256: parse_required_str(&receipt, "responseSha256")?.to_string(),
        consent_scope: consent_scope.to_string(),
        validation_status: "verified-conservative-transform".to_string(),
    };
    verify_cleanup_artifacts(session_dir, &receipt_bytes, &receipt, &expected)?
        .ok_or_else(|| anyhow::anyhow!("cleaned transcript cache binding is stale or mismatched"))
}

fn dictionary_sha256(dictionary: &[String]) -> String {
    sha256_bytes(dictionary.join("\0").as_bytes())
}

fn cleanup_binding_matches(receipt: &Value, expected: &CleanupBinding) -> bool {
    receipt.get("schema").and_then(Value::as_str) == Some(CLEANUP_RECEIPT_SCHEMA)
        && receipt.get("status").and_then(Value::as_str) == Some("completed")
        && receipt.get("sessionIdentity").and_then(Value::as_str)
            == Some(expected.session_identity.as_str())
        && receipt.get("requestedModel").and_then(Value::as_str)
            == Some(expected.requested_model.as_str())
        && receipt.get("usedModel").and_then(Value::as_str) == Some(expected.used_model.as_str())
        && receipt.get("configSha256").and_then(Value::as_str)
            == Some(expected.config_sha256.as_str())
        && receipt.get("batchArtifactSha256").and_then(Value::as_str)
            == Some(expected.batch_artifact_sha256.as_str())
        && receipt.get("batchReceiptSha256").and_then(Value::as_str)
            == Some(expected.batch_receipt_sha256.as_str())
        && receipt.get("dictionarySha256").and_then(Value::as_str)
            == Some(expected.dictionary_sha256.as_str())
        && receipt.get("source").and_then(Value::as_str) == Some(expected.source.as_str())
        && receipt.get("responseSha256").and_then(Value::as_str)
            == Some(expected.response_sha256.as_str())
        && receipt.get("consentScope").and_then(Value::as_str)
            == Some(expected.consent_scope.as_str())
        && receipt.get("validationStatus").and_then(Value::as_str)
            == Some(expected.validation_status.as_str())
        && receipt
            .get("cleanupValidationPolicyVersion")
            .and_then(Value::as_str)
            == Some(CLEANUP_VALIDATOR_VERSION)
}

fn verified_cached_cleanup(
    args: &Args,
    session_dir: &Path,
    batch: &batch_transcribe::VerifiedBatchTranscript,
    dictionary: &[String],
) -> Result<Option<VerifiedCleanedTranscript>> {
    let artifact_path = session_dir.join("cleaned-transcript.json");
    let receipt_path = session_dir.join("cleanup-receipt.json");
    if regular_file_metadata(&artifact_path).is_err()
        || regular_file_metadata(&receipt_path).is_err()
    {
        return Ok(None);
    }
    let receipt_bytes = read_regular_bytes_bounded(&receipt_path, MAX_JSON_ARTIFACT_BYTES)?;
    let receipt: Value = serde_json::from_slice(&receipt_bytes)?;
    if receipt.get("schema").and_then(Value::as_str) != Some(CLEANUP_RECEIPT_SCHEMA)
        || receipt.get("status").and_then(Value::as_str) != Some("completed")
    {
        return Ok(None);
    }
    let used_model = parse_required_str(&receipt, "usedModel")?;
    let source = parse_required_str(&receipt, "source")?;
    let validation = CleanupValidation {
        status: parse_required_str(&receipt, "validationStatus")?.to_string(),
        reason: String::new(),
    };
    let expected = CleanupBinding::new(
        session_dir,
        args,
        batch,
        dictionary,
        used_model,
        source,
        parse_required_str(&receipt, "responseSha256")?,
        &validation,
    )?;
    verify_cleanup_artifacts(session_dir, &receipt_bytes, &receipt, &expected)
}
