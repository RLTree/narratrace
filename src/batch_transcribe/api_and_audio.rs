#[derive(Debug)]
struct OpenedAudio {
    file: File,
    len: u64,
    sha256: String,
    file_name: String,
}

pub(crate) struct ApprovedAudio {
    opened: OpenedAudio,
}

impl OpenedAudio {
    fn approve(self, args: &Args) -> Result<ApprovedAudio> {
        require_openai_postprocessing_consent(args)?;
        Ok(ApprovedAudio { opened: self })
    }
}

#[allow(dead_code)]
pub(crate) fn approve_current_audio_for_postprocessing(
    args: &Args,
    session_dir: &Path,
) -> Result<ApprovedAudio> {
    open_current_audio(session_dir)?.approve(args)
}

fn checked_audio_path(session_dir: &Path) -> Result<PathBuf> {
    let manifest_path = session_dir.join("audio-retention.json");
    let bytes = read_regular_bytes_bounded(&manifest_path, MAX_CONTEXT_BYTES)?;
    let manifest: Value = serde_json::from_slice(&bytes)?;
    let path = manifest
        .get("audioPath")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            anyhow::anyhow!("audio retention artifact is required for batch transcription")
        })?;
    let path = crate::safe_path::normalize_system_temp(Path::new(path));
    let expected = crate::safe_path::normalize_system_temp(&session_dir.join("retained-audio.wav"));
    if path != expected {
        bail!("audio retention artifact must be the current session retained-audio.wav");
    }
    Ok(path)
}

fn open_current_audio(session_dir: &Path) -> Result<OpenedAudio> {
    let path = checked_audio_path(session_dir)?;
    let mut file = open_regular_file(&path)?;
    let (sha256, len) = hash_file_and_rewind(&mut file, MAX_RETAINED_AUDIO_BYTES)?;
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("retained-audio.wav")
        .to_string();
    Ok(OpenedAudio {
        file,
        len,
        sha256,
        file_name,
    })
}

#[cfg(test)]
fn audio_path(session_dir: &Path) -> Result<PathBuf> {
    let path = checked_audio_path(session_dir)?;
    open_regular_file(&path)?;
    Ok(path)
}

fn transcribe_approved_audio(
    session_dir: &Path,
    client: &Client,
    api_key: &str,
    model: &str,
    prompt: &str,
    mut approved: ApprovedAudio,
) -> Result<Vec<Value>> {
    if approved.opened.len <= MAX_AUDIO_UPLOAD_BYTES as u64 {
        let value = call_transcription_api_reader(
            client,
            api_key,
            approved.opened.file,
            approved.opened.len,
            &approved.opened.file_name,
            model,
            prompt,
        )?;
        return Ok(vec![value]);
    }
    let source = &mut approved.opened.file;
    let mut header = [0_u8; WAV_HEADER_BYTES];
    source.read_exact(&mut header)?;
    if &header[0..4] != b"RIFF" || &header[8..12] != b"WAVE" {
        bail!("retained audio is too large and is not a supported PCM WAV artifact");
    }
    if !(approved.opened.len - WAV_HEADER_BYTES as u64).is_multiple_of(2) {
        bail!("retained audio PCM payload must contain complete 16-bit samples");
    }
    let max_payload = (MAX_AUDIO_UPLOAD_BYTES - WAV_HEADER_BYTES) / 2 * 2;
    let mut remaining = approved.opened.len - WAV_HEADER_BYTES as u64;
    let mut values = Vec::new();
    let mut chunk_digests = Vec::new();
    while remaining > 0 {
        let payload_len = remaining.min(max_payload as u64) as usize;
        let mut payload = vec![0_u8; payload_len];
        source.read_exact(&mut payload)?;
        let bytes = chunk_wav_bytes(&payload, 0, payload.len())?;
        chunk_digests.push(sha256_bytes(&bytes));
        let name = format!("chunk-{:04}.wav", values.len() + 1);
        values.push(call_transcription_api_reader(
            client,
            api_key,
            Cursor::new(bytes.clone()),
            bytes.len() as u64,
            &name,
            model,
            prompt,
        )?);
        remaining -= payload_len as u64;
    }
    write_chunk_manifest(session_dir, &approved.opened.sha256, &chunk_digests)?;
    Ok(values)
}

fn call_transcription_api_reader<R: Read + Send + 'static>(
    client: &Client,
    api_key: &str,
    reader: R,
    len: u64,
    file_name: &str,
    model: &str,
    prompt: &str,
) -> Result<Value> {
    if len > MAX_AUDIO_UPLOAD_BYTES as u64 {
        bail!("audio upload chunk exceeds {MAX_AUDIO_UPLOAD_BYTES} bytes");
    }
    let file_part = multipart::Part::reader_with_length(reader, len)
        .file_name(file_name.to_string())
        .mime_str("audio/wav")?;
    let form = multipart::Form::new()
        .part("file", file_part)
        .text("model", model.to_string())
        .text("language", "en")
        .text("temperature", "0")
        .text("prompt", prompt.to_string())
        .text("include[]", "logprobs");
    let api_url = validated_api_url("NARRATED_REPLAY_BATCH_API_URL", API_URL)?;
    let response = client
        .post(api_url)
        .bearer_auth(api_key)
        .multipart(form)
        .send()?;
    let status = response.status();
    if !status.is_success() {
        bail!("batch transcription failed with {status}; response body omitted");
    }
    let body = read_response_text(response, "batch transcription response")?;
    Ok(serde_json::from_str(&body)?)
}

#[cfg(test)]
fn call_transcription_api(
    client: &Client,
    api_key: &str,
    audio_path: &Path,
    model: &str,
    prompt: &str,
) -> Result<Value> {
    let file = open_regular_file(audio_path)?;
    let len = file.metadata()?.len();
    call_transcription_api_reader(
        client,
        api_key,
        file,
        len,
        "retained-audio.wav",
        model,
        prompt,
    )
}

fn combine_chunk_transcripts(values: &[Value], chunk_count: usize) -> Value {
    if values.len() == 1 {
        return values[0].clone();
    }
    let text = values
        .iter()
        .filter_map(|value| value.get("text").and_then(Value::as_str))
        .collect::<Vec<_>>()
        .join("\n");
    json!({"text": text, "chunked": true, "chunkCount": chunk_count, "chunks": values})
}

fn chunk_wav_bytes(source: &[u8], start: usize, end: usize) -> Result<Vec<u8>> {
    if start > end || end > source.len() {
        bail!("invalid WAV chunk range");
    }
    let payload = &source[start..end];
    let payload_len = u32::try_from(payload.len()).context("WAV chunk payload too large")?;
    let mut out = Vec::with_capacity(WAV_HEADER_BYTES + payload.len());
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36_u32 + payload_len).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16_u32.to_le_bytes());
    out.extend_from_slice(&1_u16.to_le_bytes());
    out.extend_from_slice(&1_u16.to_le_bytes());
    out.extend_from_slice(&24_000_u32.to_le_bytes());
    out.extend_from_slice(&(24_000_u32 * 2).to_le_bytes());
    out.extend_from_slice(&2_u16.to_le_bytes());
    out.extend_from_slice(&16_u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&payload_len.to_le_bytes());
    out.extend_from_slice(payload);
    Ok(out)
}

fn write_chunk_manifest(session_dir: &Path, source_sha256: &str, chunks: &[String]) -> Result<()> {
    write_private(
        session_dir.join("batch-audio-chunks.json"),
        serde_json::to_string_pretty(&json!({
            "schema": "narrated-record-replay.batch-audio-chunks.v2",
            "sourceAudioSha256": source_sha256,
            "chunkCount": chunks.len(),
            "chunkSha256": chunks,
            "privacy": {"localPrivate": true, "rawAudioCopiedIntoChunks": false}
        }))?,
    )
}
