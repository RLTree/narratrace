struct FileStats {
    bytes: u64,
    line_count: u64,
    fingerprint: String,
}

fn inspect_raw_artifact(
    artifact: &ArtifactSpec<'_>,
    stats_remaining: &mut u64,
    scan_remaining: &mut u64,
) -> anyhow::Result<(FileStats, Vec<&'static str>)> {
    let mut file = open_regular_file(artifact.path)?;
    let len = file.metadata()?.len();
    if len > MAX_STATS_BYTES_PER_FILE || len > *stats_remaining {
        anyhow::bail!("artifact stats byte budget exceeded");
    }
    *stats_remaining -= len;
    let scan_text = artifact.name != "retained-audio"
        && len <= MAX_SCAN_BYTES_PER_FILE
        && len <= *scan_remaining;
    if scan_text {
        *scan_remaining -= len;
    }
    let mut hasher = <sha2::Sha256 as sha2::Digest>::new();
    let mut buffer = [0_u8; 64 * 1024];
    let mut text = scan_text.then(|| Vec::with_capacity(len as usize));
    let mut newlines = 0_u64;
    let mut last = None;
    loop {
        let count = std::io::Read::read(&mut file, &mut buffer)?;
        if count == 0 {
            break;
        }
        sha2::Digest::update(&mut hasher, &buffer[..count]);
        if let Some(text) = text.as_mut() {
            text.extend_from_slice(&buffer[..count]);
        }
        newlines += buffer[..count].iter().filter(|byte| **byte == b'\n').count() as u64;
        last = buffer.get(count - 1).copied();
    }
    let stats = FileStats {
        bytes: len,
        line_count: if len == 0 {
            0
        } else {
            newlines + u64::from(last != Some(b'\n'))
        },
        fingerprint: format!("sha256:{:x}", sha2::Digest::finalize(hasher)),
    };
    let categories = if artifact.name == "retained-audio" {
        vec!["raw-audio"]
    } else if let Some(text) = text {
        match String::from_utf8(text) {
            Ok(text) => redaction::sensitive_categories(&text),
            Err(_) => vec!["artifact-read-or-decode-failed"],
        }
    } else {
        vec!["artifact-inspection-budget-exceeded"]
    };
    Ok((stats, categories))
}
