fn verify_cleanup_artifacts(
    session_dir: &Path,
    receipt_bytes: &[u8],
    receipt: &Value,
    expected: &CleanupBinding,
) -> Result<Option<VerifiedCleanedTranscript>> {
    let artifact_bytes = read_regular_bytes_bounded(
        &session_dir.join("cleaned-transcript.json"),
        MAX_JSON_ARTIFACT_BYTES,
    )?;
    let artifact: Value = serde_json::from_slice(&artifact_bytes)?;
    let artifact_sha256 = sha256_bytes(&artifact_bytes);
    let source = artifact.get("sourceBinding").unwrap_or(&Value::Null);
    let raw_response_sha256 = artifact
        .get("raw")
        .map(serde_json::to_vec)
        .transpose()?
        .map(|bytes| sha256_bytes(&bytes));
    if artifact.get("schema").and_then(Value::as_str) != Some(CLEANUP_ARTIFACT_SCHEMA)
        || !cleanup_binding_matches(receipt, expected)
        || expected.validation_status != "verified-conservative-transform"
        || artifact
            .get("validation")
            .and_then(|value| value.get("status"))
            .and_then(Value::as_str)
            != Some("verified-conservative-transform")
        || artifact
            .get("cleanupValidationPolicyVersion")
            .and_then(Value::as_str)
            != Some(CLEANUP_VALIDATOR_VERSION)
        || !cleanup_artifact_binding_matches(source, expected)
        || raw_response_sha256.as_deref() != Some(expected.response_sha256.as_str())
        || receipt.get("artifactSha256").and_then(Value::as_str) != Some(artifact_sha256.as_str())
    {
        return Ok(None);
    }
    let text = artifact
        .get("cleanedText")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    if text.is_empty() {
        return Ok(None);
    }
    Ok(Some(VerifiedCleanedTranscript {
        text,
        artifact_sha256,
        receipt_sha256: sha256_bytes(receipt_bytes),
        batch_artifact_sha256: expected.batch_artifact_sha256.clone(),
        session_identity: expected.session_identity.clone(),
        validation_policy_version: CLEANUP_VALIDATOR_VERSION.to_string(),
    }))
}

fn cleanup_artifact_binding_matches(source: &Value, expected: &CleanupBinding) -> bool {
    source.get("sessionIdentity").and_then(Value::as_str)
        == Some(expected.session_identity.as_str())
        && source.get("configSha256").and_then(Value::as_str)
            == Some(expected.config_sha256.as_str())
        && source.get("batchArtifactSha256").and_then(Value::as_str)
            == Some(expected.batch_artifact_sha256.as_str())
        && source.get("batchReceiptSha256").and_then(Value::as_str)
            == Some(expected.batch_receipt_sha256.as_str())
        && source.get("dictionarySha256").and_then(Value::as_str)
            == Some(expected.dictionary_sha256.as_str())
        && source.get("responseSha256").and_then(Value::as_str)
            == Some(expected.response_sha256.as_str())
        && source.get("consentScope").and_then(Value::as_str)
            == Some(expected.consent_scope.as_str())
}
