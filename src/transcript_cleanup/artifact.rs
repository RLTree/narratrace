fn write_cleanup_artifact(
    session_dir: &Path,
    output_path: &Path,
    batch: &batch_transcribe::VerifiedBatchTranscript,
    dictionary: &[String],
    model_input: &CleanupModelInput,
    suggested_text: String,
    raw: Value,
    fallback: Option<String>,
    binding: &CleanupBinding,
    validation: &CleanupValidation,
) -> Result<()> {
    let cleaned_text = validation.is_verified().then_some(suggested_text.as_str());
    let artifact = serde_json::to_string_pretty(&json!({
        "schema": CLEANUP_ARTIFACT_SCHEMA,
        "source": binding.source,
        "requestedModel": binding.requested_model,
        "model": binding.used_model,
        "modelFallback": fallback,
        "store": false,
        "tools": [],
        "toolChoice": "none",
        "cleanupPolicyVersion": CLEANUP_POLICY_VERSION,
        "cleanupValidationPolicyVersion": CLEANUP_VALIDATOR_VERSION,
        "trustedInstructions": model_input.trusted_instructions,
        "untrustedInput": model_input.untrusted_data,
        "dictionary": dictionary,
        "rawBatchText": batch.text,
        "suggestedText": suggested_text,
        "cleanedText": cleaned_text,
        "raw": raw,
        "validation": {
            "status": validation.status,
            "reason": validation.reason
        },
        "sourceBinding": {
            "sessionIdentity": binding.session_identity,
            "configSha256": binding.config_sha256,
            "batchArtifactSha256": binding.batch_artifact_sha256,
            "batchReceiptSha256": binding.batch_receipt_sha256,
            "dictionarySha256": binding.dictionary_sha256,
            "responseSha256": binding.response_sha256,
            "consentScope": binding.consent_scope
        },
        "privacy": {
            "localPrivate": true,
            "copyIntoGeneratedPacketsByDefault": false,
            "conservativeCleanupOnly": true
        }
    }))?;
    let artifact_sha256 = sha256_bytes(artifact.as_bytes());
    write_private(output_path, artifact.as_bytes())?;
    write_private(
        session_dir.join("cleanup-receipt.json"),
        serde_json::to_string_pretty(&json!({
            "schema": CLEANUP_RECEIPT_SCHEMA,
            "status": "completed",
            "sessionIdentity": binding.session_identity,
            "requestedModel": binding.requested_model,
            "usedModel": binding.used_model,
            "configSha256": binding.config_sha256,
            "batchArtifactSha256": binding.batch_artifact_sha256,
            "batchReceiptSha256": binding.batch_receipt_sha256,
            "dictionarySha256": binding.dictionary_sha256,
            "source": binding.source,
            "responseSha256": binding.response_sha256,
            "consentScope": binding.consent_scope,
            "validationStatus": binding.validation_status,
            "cleanupValidationPolicyVersion": CLEANUP_VALIDATOR_VERSION,
            "artifactSha256": artifact_sha256,
            "artifact": output_path.display().to_string()
        }))?,
    )
}

fn write_cleanup_disabled(session_dir: &Path, reason: &str) -> Result<()> {
    write_private(
        session_dir.join("cleanup-receipt.json"),
        serde_json::to_string_pretty(&json!({
            "schema": CLEANUP_RECEIPT_SCHEMA,
            "status": "disabled",
            "cleanupValidationPolicyVersion": CLEANUP_VALIDATOR_VERSION,
            "reason": reason
        }))?,
    )
}

#[cfg(test)]
pub(crate) fn write_bound_cleanup_fixture_for_test(
    args: &Args,
    session_dir: &Path,
    raw_text: &str,
    cleaned_text: &str,
) -> Result<PathBuf> {
    batch_transcribe::write_bound_batch_fixture_for_test(
        session_dir,
        &args.batch_transcription_model,
        raw_text,
    )?;
    let batch = batch_transcribe::verified_batch_for_cleanup(args, session_dir)?;
    let dictionary = build_dictionary(args, session_dir);
    let input = cleanup_model_input(&dictionary, raw_text)?;
    let raw = json!({"output_text": cleaned_text});
    let validation = validate_cleanup_output(raw_text, cleaned_text, &dictionary);
    let binding = CleanupBinding::new(
        session_dir,
        args,
        &batch,
        &dictionary,
        cleanup_model_name(&args.cleanup_model),
        "fixture",
        &sha256_bytes(&serde_json::to_vec(&raw)?),
        &validation,
    )?;
    let output = session_dir.join("cleaned-transcript.json");
    write_cleanup_artifact(
        session_dir,
        &output,
        &batch,
        &dictionary,
        &input,
        cleaned_text.to_string(),
        raw,
        None,
        &binding,
        &validation,
    )?;
    Ok(output)
}
