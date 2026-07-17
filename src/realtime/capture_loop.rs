async fn capture_inner(args: &Args, session_dir: &std::path::Path) -> Result<()> {
    let runtime = RuntimeConfig::production()?;
    capture_inner_with_runtime(args, session_dir, runtime).await
}

async fn capture_inner_with_runtime(
    args: &Args,
    session_dir: &std::path::Path,
    runtime: RuntimeConfig<'_>,
) -> Result<()> {
    let resolved_input = resolve_avfoundation_input(&args.input)?;
    if let Err(error) = ensure_not_iphone_input(&resolved_input) {
        let message = error.to_string();
        write_status(
            session_dir,
            "failed",
            &args.delay,
            args.max_seconds,
            Some(&resolved_input),
            Some(&message),
        )?;
        return Err(error);
    }
    let api_key = std::env::var("OPENAI_API_KEY")?;
    let mut request = runtime.realtime_url.into_client_request()?;
    request
        .headers_mut()
        .insert("Authorization", format!("Bearer {api_key}").parse()?);
    let websocket_config = WebSocketConfig::default()
        .max_message_size(Some(MAX_REALTIME_MESSAGE_BYTES))
        .max_frame_size(Some(MAX_REALTIME_MESSAGE_BYTES));
    let (socket, _) = connect_async_with_config(request, Some(websocket_config), false).await?;
    let (mut write, mut read) = socket.split();
    write
        .send(Message::Text(
            session_update(&args.delay).to_string().into(),
        ))
        .await?;
    timeline::write_capture_clock(&session_dir, &args.delay)?;
    write_status(
        &session_dir,
        "recording",
        &args.delay,
        args.max_seconds,
        Some(&resolved_input),
        None,
    )?;
    write_sync_sentinel(session_dir, "start", 0)?;

    let mut ffmpeg = spawn_audio_capture(args, runtime, &resolved_input)?;
    let mut stdout = ffmpeg
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("ffmpeg stdout unavailable"))?;
    let mut buffer = [0_u8; 8192];
    let mut retained_audio = AudioRetentionWriter::create(
        session_dir,
        args.audio_retention_path.as_deref(),
        &args.audio_retention_mode,
    )?;

    let started = Instant::now();
    let mut audio_chunks_sent = 0_u64;
    let mut audio_bytes_sent = 0_u64;
    let mut realtime_messages = 0_u64;
    let mut realtime_delta_events = 0_u64;
    let mut realtime_completed_segments = 0_u64;
    let mut realtime_error_events = 0_u64;
    let mut audio_bytes_since_commit = 0_u64;
    let mut audio_commits_sent = 0_u64;
    let mut last_audio_commit = Instant::now();
    let mut first_audio_chunk_recorded = false;
    let mut quota = CaptureQuota::default();
    loop {
        if session_dir.join(".stop").exists() {
            break;
        }
        if let Some(max_seconds) = args.max_seconds
            && started.elapsed() >= Duration::from_secs(max_seconds)
        {
            break;
        }
        tokio::select! {
            read_bytes = stdout.read(&mut buffer) => {
                let count = read_bytes?;
                if count == 0 {
                    break;
                }
                reserve_capture_audio(
                    args,
                    session_dir,
                    &resolved_input,
                    &mut quota,
                    count as u64,
                )?;
                let monotonic_offset_ms = started.elapsed().as_millis() as u64;
                if !first_audio_chunk_recorded {
                    timeline::record_first_audio_chunk_clock(&session_dir)?;
                    first_audio_chunk_recorded = true;
                }
                let audio = STANDARD.encode(&buffer[..count]);
                let msg = json!({"type":"input_audio_buffer.append","audio":audio});
                write.send(Message::Text(msg.to_string().into())).await?;
                if let Some(writer) = retained_audio.as_mut() {
                    writer.append(&buffer[..count], monotonic_offset_ms)?;
                }
                audio_chunks_sent += 1;
                audio_bytes_sent += count as u64;
                audio_bytes_since_commit += count as u64;
                if should_commit_audio_buffer(
                    audio_bytes_since_commit,
                    last_audio_commit.elapsed(),
                    PERIODIC_COMMIT_INTERVAL,
                    MINIMUM_COMMIT_BYTES,
                ) {
                    write
                        .send(Message::Text(
                            json!({"type":"input_audio_buffer.commit"})
                                .to_string()
                                .into(),
                        ))
                        .await?;
                    audio_commits_sent += 1;
                    audio_bytes_since_commit = 0;
                    last_audio_commit = Instant::now();
                }
            }
            message = read.next() => match message {
                Some(Ok(Message::Text(text))) => {
                    realtime_messages += 1;
                    let kind = handle_capture_event(
                        args,
                        session_dir,
                        &resolved_input,
                        &mut quota,
                        &text,
                        started.elapsed().as_millis() as u64,
                    )?;
                    match kind {
                        EventKind::Completed => realtime_completed_segments += 1,
                        EventKind::Delta => realtime_delta_events += 1,
                        EventKind::Error => realtime_error_events += 1,
                        EventKind::Other => {}
                    }
                }
                Some(Ok(_)) => {}
                Some(Err(error)) => {
                    return fail_websocket_read(args, session_dir, &resolved_input, error);
                }
                None => break,
            },
            _ = sleep(Duration::from_millis(400)) => {}
        }
    }
    let mut final_commit =
        send_final_audio_commit(&mut write, audio_bytes_since_commit, audio_commits_sent).await;
    let drain_started = Instant::now();
    let drain_deadline = drain_started + Duration::from_secs(5);
    let mut post_commit_messages = 0_u64;
    let mut post_commit_completed_segments = 0_u64;
    let mut post_commit_errors = final_commit.errors();
    loop {
        let remaining = drain_deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero()
            || post_commit_completed_segments > 0
            || !post_commit_errors.is_empty()
        {
            break;
        }
        match timeout(remaining, read.next()).await {
            Ok(Some(Ok(Message::Text(text)))) => {
                post_commit_messages += 1;
                let kind = handle_capture_event(
                    args,
                    session_dir,
                    &resolved_input,
                    &mut quota,
                    &text,
                    started.elapsed().as_millis() as u64,
                )?;
                match kind {
                    EventKind::Completed => {
                        post_commit_completed_segments += 1;
                        realtime_completed_segments += 1;
                    }
                    EventKind::Delta => realtime_delta_events += 1,
                    EventKind::Error => {
                        realtime_error_events += 1;
                        post_commit_errors.push("realtime-error-event".to_string());
                    }
                    EventKind::Other => {}
                }
            }
            Ok(Some(Ok(_))) => {
                post_commit_messages += 1;
            }
            Ok(Some(Err(error))) => {
                post_commit_errors.push(error.to_string());
                break;
            }
            Ok(None) | Err(_) => break,
        }
    }
    let final_commit_error =
        final_commit.finish(post_commit_completed_segments, &mut post_commit_errors);
    write_private(
        session_dir.join("post-commit-drain.json"),
        serde_json::to_string_pretty(&post_commit_drain_receipt(
            drain_started.elapsed().as_millis() as u64,
            post_commit_messages,
            post_commit_completed_segments,
            post_commit_errors,
            audio_chunks_sent,
            audio_bytes_sent,
            final_commit.audio_commits_sent,
            final_commit.audio_bytes_pending,
            realtime_messages,
            realtime_delta_events,
            realtime_completed_segments,
            realtime_error_events,
            &args.audio_filter,
            final_commit.status(),
        ))?,
    )?;
    if let Some(writer) = retained_audio.take() {
        let _ = writer.finalize(session_dir, &args.audio_retention_mode)?;
    }
    write_sync_sentinel(session_dir, "stop", started.elapsed().as_millis() as u64)?;
    let _ = ffmpeg.kill().await;
    update_thought_process(&session_dir)?;
    let terminal_state = if final_commit_error.is_some() {
        "failed"
    } else {
        "stopped"
    };
    write_status(
        &session_dir,
        terminal_state,
        &args.delay,
        args.max_seconds,
        Some(&resolved_input),
        final_commit_error.as_deref(),
    )?;
    if let Some(error) = final_commit_error {
        anyhow::bail!(error);
    }
    Ok(())
}
