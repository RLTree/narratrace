fn spawn_audio_capture(
    args: &Args,
    runtime: RuntimeConfig<'_>,
    input: &crate::audio_input::ResolvedAudioInput,
) -> Result<tokio::process::Child> {
    let mut command = Command::new(runtime.ffmpeg_binary);
    command
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-f",
            "avfoundation",
            "-i",
            &input.ffmpeg_input,
        ])
        .args([
            "-ac",
            "1",
            "-ar",
            &SAMPLE_RATE.to_string(),
            "-af",
            &args.audio_filter,
            "-f",
            "s16le",
            "pipe:1",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    Ok(command.spawn()?)
}

fn reserve_capture_audio(
    args: &Args,
    session_dir: &std::path::Path,
    input: &crate::audio_input::ResolvedAudioInput,
    quota: &mut CaptureQuota,
    bytes: u64,
) -> Result<()> {
    if let Err(error) = quota.reserve_audio(bytes) {
        write_status(
            session_dir,
            "failed",
            &args.delay,
            args.max_seconds,
            Some(input),
            Some("capture audio quota exceeded"),
        )?;
        return Err(error);
    }
    Ok(())
}

fn handle_capture_event(
    args: &Args,
    session_dir: &std::path::Path,
    input: &crate::audio_input::ResolvedAudioInput,
    quota: &mut CaptureQuota,
    text: &str,
    monotonic_offset_ms: u64,
) -> Result<EventKind> {
    match handle_event(session_dir, text, monotonic_offset_ms, quota) {
        Ok(kind) => Ok(kind),
        Err(error) => {
            write_status(
                session_dir,
                "failed",
                &args.delay,
                args.max_seconds,
                Some(input),
                Some("realtime event rejected before persistence"),
            )?;
            Err(error)
        }
    }
}

fn fail_websocket_read(
    args: &Args,
    session_dir: &std::path::Path,
    input: &crate::audio_input::ResolvedAudioInput,
    error: tokio_tungstenite::tungstenite::Error,
) -> Result<()> {
    write_status(
        session_dir,
        "failed",
        &args.delay,
        args.max_seconds,
        Some(input),
        Some("realtime websocket read failed; payload omitted"),
    )?;
    Err(error.into())
}
