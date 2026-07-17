fn write_batch_artifact(
    session_dir: &Path,
    output_path: &Path,
    model: &str,
    prompt: &str,
    source: &str,
    raw: Value,
    binding: &BatchBinding,
) -> Result<()> {
    let transcription_text = raw
        .get("text")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let artifact = serde_json::to_string_pretty(&json!({
        "schema": BATCH_ARTIFACT_SCHEMA,
        "source": source,
        "model": model,
        "language": "en",
        "temperature": 0,
        "includeLogprobsRequested": true,
        "promptPolicyVersion": BATCH_PROMPT_POLICY_VERSION,
        "prompt": prompt,
        "raw": raw,
        "transcription": {"text": transcription_text},
        "sourceBinding": {
            "sessionIdentity": binding.session_identity,
            "configSha256": binding.config_sha256,
            "sourceKind": binding.source_kind,
            "sourceSha256": binding.source_sha256,
            "sourceBytes": binding.source_bytes,
            "consentScope": binding.consent_scope
        },
        "privacy": {
            "localPrivate": true,
            "rawAudioCopied": false,
            "copyIntoGeneratedPacketsByDefault": false
        }
    }))?;
    let artifact_sha256 = sha256_bytes(artifact.as_bytes());
    write_private(output_path, artifact.as_bytes())?;
    write_private(
        session_dir.join("batch-transcription-receipt.json"),
        serde_json::to_string_pretty(&json!({
            "schema": BATCH_RECEIPT_SCHEMA,
            "status": "completed",
            "source": source,
            "model": model,
            "sessionIdentity": binding.session_identity,
            "configSha256": binding.config_sha256,
            "sourceKind": binding.source_kind,
            "sourceSha256": binding.source_sha256,
            "sourceBytes": binding.source_bytes,
            "consentScope": binding.consent_scope,
            "artifactSha256": artifact_sha256,
            "artifact": output_path.display().to_string()
        }))?,
    )
}

fn write_disabled(session_dir: &Path, reason: &str) -> Result<()> {
    write_private(
        session_dir.join("batch-transcription-receipt.json"),
        serde_json::to_string_pretty(&json!({
            "schema": BATCH_RECEIPT_SCHEMA,
            "status": "disabled",
            "reason": reason
        }))?,
    )
}

#[cfg(test)]
pub(crate) fn write_bound_batch_fixture_for_test(
    session_dir: &Path,
    model: &str,
    text: &str,
) -> Result<PathBuf> {
    let output_path = session_dir.join("batch-transcript.json");
    let prompt = build_prompt(None, None);
    let raw = json!({"text": text});
    let fixture_bytes = serde_json::to_vec(&raw)?;
    let binding = BatchBinding::for_fixture(
        session_dir,
        model,
        &prompt,
        &sha256_bytes(&fixture_bytes),
        fixture_bytes.len() as u64,
    )?;
    write_batch_artifact(
        session_dir,
        &output_path,
        model,
        &prompt,
        "fixture",
        raw,
        &binding,
    )?;
    Ok(output_path)
}
