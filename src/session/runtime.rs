fn write_manifest(
    session_dir: &Path,
    goal: &str,
    coordination_mode: &str,
    coordinated: bool,
    microphone_consent: &str,
    audio_filter: &str,
) -> Result<()> {
    write_private(
        session_dir.join("manifest.json"),
        serde_json::to_string_pretty(&json!({
            "schema": "narrated-record-replay.v1",
            "goal": goal,
            "model": MODEL,
            "realtimeEndpointIntent": REALTIME_ENDPOINT_INTENT,
            "postStopQualityPipeline": {
                "realtimeTimingSpine": true,
                "defaultRealtimeDelay": "high",
                "batchTranscriptionModel": DEFAULT_BATCH_TRANSCRIPTION_MODEL,
                "cleanupModel": DEFAULT_CLEANUP_MODEL,
                "cleanupDictionaryEntryCap": 100,
                "audioFilter": audio_filter,
                "defaultAudioFilter": DEFAULT_AUDIO_FILTER,
                "rawAudioCopiedIntoGeneratedPacketsByDefault": false,
                "rawTranscriptsCopiedIntoGeneratedPacketsByDefault": false,
                "requiresPacketTimeOpenAIConsentFlag": "--i-consent-to-openai-postprocessing"
            },
            "microphoneConsent": microphone_consent,
            "startCoordination": {
                "mode": coordination_mode,
                "recordReplayAndMicrophoneSameOperation": coordinated,
                "manualSequentialStartAllowedForLiveProof": false,
                "claimCeiling": if coordinated {
                    "prepared for parent-orchestrated parallel Record & Replay plus microphone start"
                } else {
                    "manual or externally sequenced start; cannot close live Record & Replay capture proof"
                }
            },
            "narrationQualityTargets": narration_quality_targets(),
            "privacy": "local-private",
            "note": "Transcript is Tree's spoken thought process/context, not screen evidence."
        }))?,
    )?;
    Ok(())
}

pub fn status(args: &Args) -> Result<()> {
    let session_dir = required_session_dir(args)?;
    println!("{}", read_regular_text(&session_dir.join("status.json"))?);
    Ok(())
}

pub async fn stop(args: &Args) -> Result<()> {
    stop_with_poll(args, 40, Duration::from_millis(400)).await
}

async fn stop_with_poll(args: &Args, attempts: usize, delay: Duration) -> Result<()> {
    let session_dir = required_session_dir(args)?;
    write_private(session_dir.join(".stop"), stamp())?;
    mark_stop_requested(&session_dir)?;
    for _ in 0..attempts {
        let status_path = session_dir.join("status.json");
        if let Ok(text) = read_regular_text(&status_path)
            && terminal_status_state(&text)
        {
            println!("{text}");
            return Ok(());
        }
        sleep(delay).await;
    }
    write_private(
        session_dir.join("stop-timeout.json"),
        serde_json::to_string_pretty(&json!({
            "schema": "narrated-record-replay.stop-timeout.v1",
            "status": "timeout",
            "sessionDir": session_dir,
            "waitedMs": delay.as_millis() as u64 * attempts as u64,
            "stopFilePresent": session_dir.join(".stop").exists(),
            "lastStatus": read_regular_text(&session_dir.join("status.json"))
                .ok()
                .and_then(|text| serde_json::from_str::<Value>(&text).ok())
                .unwrap_or(Value::Null),
            "claimCeiling": "stop request observed but capture helper did not reach stopped or failed before timeout"
        }))?,
    )?;
    println!(
        "{}",
        serde_json::to_string_pretty(
            &json!({ "state": "stop_requested", "sessionDir": session_dir })
        )?
    );
    bail!(
        "stop timed out before capture reached stopped or failed: {}",
        session_dir.display()
    )
}

fn mark_stop_requested(session_dir: &Path) -> Result<()> {
    let status_path = session_dir.join("status.json");
    let mut status = read_regular_text(&status_path)
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
        .filter(Value::is_object)
        .unwrap_or_else(|| json!({}));
    if matches!(
        status.get("state").and_then(Value::as_str),
        Some("stopped" | "failed")
    ) {
        return Ok(());
    }
    status["state"] = json!("stop-requested");
    status["stopRequestedAt"] = json!(stamp());
    status["sessionDir"] = json!(session_dir);
    write_private(status_path, serde_json::to_string_pretty(&status)?)?;
    Ok(())
}

fn read_regular_text(path: &Path) -> Result<String> {
    read_regular_text_after_open(path, || {})
}

fn read_regular_text_after_open(path: &Path, after_open: impl FnOnce()) -> Result<String> {
    const MAX_STATUS_BYTES: u64 = 64 * 1024;
    let file = open_regular_file(path)?;
    after_open();
    let mut bytes = Vec::new();
    file.take(MAX_STATUS_BYTES + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_STATUS_BYTES {
        bail!(
            "status artifact exceeds {MAX_STATUS_BYTES} bytes: {}",
            path.display()
        );
    }
    String::from_utf8(bytes).context("status artifact is not valid UTF-8")
}

fn terminal_status_state(text: &str) -> bool {
    serde_json::from_str::<Value>(text)
        .ok()
        .and_then(|status| {
            status
                .get("state")
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
        .is_some_and(|state| matches!(state.as_str(), "stopped" | "failed"))
}

fn write_status(session_dir: &Path, state: &str, pid: Option<u32>) -> Result<()> {
    write_private(
        session_dir.join("status.json"),
        serde_json::to_string_pretty(&json!({
            "state": state,
            "sessionDir": session_dir,
            "pid": pid,
            "model": MODEL,
            "realtimeEndpointIntent": REALTIME_ENDPOINT_INTENT,
        }))?,
    )?;
    Ok(())
}

fn allocate_session_dir(root: &Path, goal: &str) -> Result<PathBuf> {
    allocate_session_dir_with(root, || {
        Ok(format!(
            "{}-{}-{}",
            stamp(),
            slugify(goal),
            random_session_nonce()?
        ))
    })
}

fn allocate_session_dir_with(
    root: &Path,
    mut next_name: impl FnMut() -> Result<String>,
) -> Result<PathBuf> {
    for _ in 0..32 {
        let candidate = root.join(next_name()?);
        let mut builder = fs::DirBuilder::new();
        #[cfg(unix)]
        builder.mode(0o700);
        match builder.create(&candidate) {
            Ok(()) => {
                create_private_dir_all(&candidate)?;
                return Ok(candidate);
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error.into()),
        }
    }
    bail!("could not allocate an exclusive narrated replay session directory")
}

fn random_session_nonce() -> Result<String> {
    let mut bytes = [0_u8; 16];
    fs::File::open("/dev/urandom")
        .context("open operating-system random source")?
        .read_exact(&mut bytes)
        .context("read operating-system random source")?;
    Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
}

#[cfg(test)]
#[path = "security_tests.rs"]
mod security_tests;

fn stamp() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{secs}")
}
