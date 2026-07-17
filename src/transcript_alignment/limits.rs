const MAX_ALIGNMENT_TOKENS_PER_SIDE: usize = 10_000;
const MAX_ALIGNMENT_CELLS: usize = 4_000_000;
const MAX_ALIGNMENT_TEXT_BYTES_PER_SIDE: usize = 4 * 1024 * 1024;
const MAX_ALIGNMENT_TOKEN_BYTES: usize = 256;

fn enforce_text_limits(
    cleaned_text: &str,
    realtime_segments: &[timeline::TranscriptSegment],
) -> Result<()> {
    if cleaned_text.len() > MAX_ALIGNMENT_TEXT_BYTES_PER_SIDE {
        anyhow::bail!("cleaned transcript text byte limit exceeded");
    }
    let realtime_bytes = realtime_segments.iter().try_fold(0_usize, |total, segment| {
        total.checked_add(segment.text.len())
            .ok_or_else(|| anyhow::anyhow!("realtime transcript byte count overflow"))
    })?;
    if realtime_bytes > MAX_ALIGNMENT_TEXT_BYTES_PER_SIDE {
        anyhow::bail!("realtime transcript text byte limit exceeded");
    }
    for token in std::iter::once(cleaned_text)
        .chain(realtime_segments.iter().map(|segment| segment.text.as_str()))
        .flat_map(|text| text.split(|ch: char| !ch.is_ascii_alphanumeric()))
        .filter(|token| !token.is_empty())
    {
        if token.len() > MAX_ALIGNMENT_TOKEN_BYTES {
            anyhow::bail!("transcript alignment token byte limit exceeded");
        }
    }
    Ok(())
}

fn enforce_alignment_limits(cleaned_tokens: usize, realtime_tokens: usize) -> Result<()> {
    if cleaned_tokens > MAX_ALIGNMENT_TOKENS_PER_SIDE
        || realtime_tokens > MAX_ALIGNMENT_TOKENS_PER_SIDE
    {
        anyhow::bail!(
            "transcript alignment token limit exceeded: cleaned={cleaned_tokens}, realtime={realtime_tokens}, max_per_side={MAX_ALIGNMENT_TOKENS_PER_SIDE}"
        );
    }
    let cells = cleaned_tokens
        .checked_add(1)
        .and_then(|rows| {
            realtime_tokens
                .checked_add(1)
                .and_then(|columns| rows.checked_mul(columns))
        })
        .ok_or_else(|| anyhow::anyhow!("transcript alignment work-product overflow"))?;
    if cells > MAX_ALIGNMENT_CELLS {
        anyhow::bail!(
            "transcript alignment work limit exceeded: cells={cells}, max_cells={MAX_ALIGNMENT_CELLS}"
        );
    }
    Ok(())
}
