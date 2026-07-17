const BATCH_ARTIFACT_SCHEMA: &str = "narrated-record-replay.batch-transcript.v2";
const BATCH_RECEIPT_SCHEMA: &str = "narrated-record-replay.batch-transcription-receipt.v2";
const BATCH_AUDIO_CONSENT_SCOPE: &str = "current-session-retained-audio:openai-transcription";
#[cfg(test)]
const BATCH_FIXTURE_SCOPE: &str = "current-test-session-fixture:no-network";

#[derive(Debug, Clone)]
struct BatchBinding {
    session_identity: String,
    model: String,
    config_sha256: String,
    source_kind: String,
    source_sha256: String,
    source_bytes: u64,
    consent_scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct VerifiedBatchTranscript {
    pub text: String,
    pub artifact_sha256: String,
    pub receipt_sha256: String,
    pub session_identity: String,
}

impl BatchBinding {
    fn for_audio(
        session_dir: &Path,
        model: &str,
        prompt: &str,
        sha256: &str,
        bytes: u64,
    ) -> Result<Self> {
        Ok(Self {
            session_identity: session_identity(session_dir)?,
            model: model.to_string(),
            config_sha256: batch_config_sha256(model, prompt),
            source_kind: "retained-audio".to_string(),
            source_sha256: sha256.to_string(),
            source_bytes: bytes,
            consent_scope: BATCH_AUDIO_CONSENT_SCOPE.to_string(),
        })
    }

    #[cfg(test)]
    fn for_fixture(
        session_dir: &Path,
        model: &str,
        prompt: &str,
        sha256: &str,
        bytes: u64,
    ) -> Result<Self> {
        Ok(Self {
            session_identity: session_identity(session_dir)?,
            model: model.to_string(),
            config_sha256: batch_config_sha256(model, prompt),
            source_kind: "test-fixture".to_string(),
            source_sha256: sha256.to_string(),
            source_bytes: bytes,
            consent_scope: BATCH_FIXTURE_SCOPE.to_string(),
        })
    }
}

fn batch_config_sha256(model: &str, prompt: &str) -> String {
    sha256_bytes(
        format!(
            "batch-config-v2\0{model}\0{BATCH_PROMPT_POLICY_VERSION}\0{prompt}\0en\00\0logprobs"
        )
        .as_bytes(),
    )
}

fn session_identity(session_dir: &Path) -> Result<String> {
    let normalized = crate::safe_path::normalize_system_temp(session_dir);
    let manifest = normalized.join("manifest.json");
    let manifest_sha256 = if std::fs::symlink_metadata(&manifest).is_ok() {
        let bytes = read_regular_bytes_bounded(&manifest, MAX_CONTEXT_BYTES)?;
        sha256_bytes(&bytes)
    } else {
        "manifest-absent".to_string()
    };
    Ok(sha256_bytes(
        format!(
            "session-binding-v2\0{}\0{manifest_sha256}",
            normalized.display()
        )
        .as_bytes(),
    ))
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

fn sha256_bytes(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn hash_file_and_rewind(file: &mut File, max_bytes: u64) -> Result<(String, u64)> {
    file.seek(SeekFrom::Start(0))?;
    let len = file.metadata()?.len();
    if len > max_bytes {
        bail!("file exceeds {max_bytes} byte limit");
    }
    let mut hasher = Sha256::new();
    let copied = std::io::copy(&mut file.take(max_bytes + 1), &mut hasher)?;
    if copied > max_bytes {
        bail!("file exceeds {max_bytes} byte limit");
    }
    file.seek(SeekFrom::Start(0))?;
    let digest = hasher.finalize();
    Ok((
        digest.iter().map(|byte| format!("{byte:02x}")).collect(),
        copied,
    ))
}

fn parse_required_str<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("missing or invalid {key}"))
}

fn binding_matches(receipt: &Value, expected: &BatchBinding) -> bool {
    receipt.get("schema").and_then(Value::as_str) == Some(BATCH_RECEIPT_SCHEMA)
        && receipt.get("status").and_then(Value::as_str) == Some("completed")
        && receipt.get("sessionIdentity").and_then(Value::as_str)
            == Some(expected.session_identity.as_str())
        && receipt.get("model").and_then(Value::as_str) == Some(expected.model.as_str())
        && receipt.get("configSha256").and_then(Value::as_str)
            == Some(expected.config_sha256.as_str())
        && receipt.get("sourceKind").and_then(Value::as_str) == Some(expected.source_kind.as_str())
        && receipt.get("sourceSha256").and_then(Value::as_str)
            == Some(expected.source_sha256.as_str())
        && receipt.get("sourceBytes").and_then(Value::as_u64) == Some(expected.source_bytes)
        && receipt.get("consentScope").and_then(Value::as_str)
            == Some(expected.consent_scope.as_str())
}

fn verified_cached_batch(
    session_dir: &Path,
    expected: &BatchBinding,
) -> Result<Option<VerifiedBatchTranscript>> {
    let artifact_path = session_dir.join("batch-transcript.json");
    let receipt_path = session_dir.join("batch-transcription-receipt.json");
    if regular_file_metadata(&artifact_path).is_err()
        || regular_file_metadata(&receipt_path).is_err()
    {
        return Ok(None);
    }
    let artifact_bytes = read_regular_bytes_bounded(&artifact_path, MAX_JSON_ARTIFACT_BYTES)?;
    let receipt_bytes = read_regular_bytes_bounded(&receipt_path, MAX_JSON_ARTIFACT_BYTES)?;
    let artifact: Value = serde_json::from_slice(&artifact_bytes)?;
    let receipt: Value = serde_json::from_slice(&receipt_bytes)?;
    let artifact_sha256 = sha256_bytes(&artifact_bytes);
    let source = artifact.get("sourceBinding").unwrap_or(&Value::Null);
    if artifact.get("schema").and_then(Value::as_str) != Some(BATCH_ARTIFACT_SCHEMA)
        || !binding_matches(&receipt, expected)
        || source.get("sessionIdentity").and_then(Value::as_str)
            != Some(expected.session_identity.as_str())
        || source.get("configSha256").and_then(Value::as_str)
            != Some(expected.config_sha256.as_str())
        || source.get("sourceKind").and_then(Value::as_str) != Some(expected.source_kind.as_str())
        || source.get("sourceSha256").and_then(Value::as_str)
            != Some(expected.source_sha256.as_str())
        || source.get("sourceBytes").and_then(Value::as_u64) != Some(expected.source_bytes)
        || source.get("consentScope").and_then(Value::as_str)
            != Some(expected.consent_scope.as_str())
        || receipt.get("artifactSha256").and_then(Value::as_str) != Some(artifact_sha256.as_str())
    {
        return Ok(None);
    }
    let text = batch_text(&artifact);
    if text.is_empty() {
        return Ok(None);
    }
    Ok(Some(VerifiedBatchTranscript {
        text,
        artifact_sha256,
        receipt_sha256: sha256_bytes(&receipt_bytes),
        session_identity: expected.session_identity.clone(),
    }))
}
