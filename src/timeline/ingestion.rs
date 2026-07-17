use crate::safe_path::regular_file_metadata;
use anyhow::{Result, bail};
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub(super) const EVENT_MAX_BYTES: u64 = 32 * 1024 * 1024;
pub(super) const EVENT_MAX_ROWS: usize = 20_000;
pub(super) const EVENT_TEXT_MAX_BYTES: usize = 4 * 1024;
pub(super) const MAX_ALIGNMENT_SEGMENTS: usize = 10_000;
pub(super) const MAX_ALIGNMENT_WORK_ITEMS: usize = 5_000_000;
pub(super) const METADATA_MAX_BYTES: u64 = 1024 * 1024;
pub(super) const TRANSCRIPT_MAX_BYTES: u64 = 8 * 1024 * 1024;
pub(super) const TRANSCRIPT_MAX_ROWS: usize = 20_000;
pub(super) const TRANSCRIPT_TEXT_MAX_BYTES: usize = 64 * 1024;

pub(super) fn read_bounded(path: &Path, label: &str, max_bytes: u64) -> Result<String> {
    let metadata = regular_file_metadata(path)?;
    if metadata.len() > max_bytes {
        bail!("{label} exceeds {max_bytes} byte limit");
    }

    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    File::open(path)?
        .take(max_bytes + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > max_bytes {
        bail!("{label} exceeds {max_bytes} byte limit");
    }
    String::from_utf8(bytes).map_err(Into::into)
}

pub(super) fn enforce_rows(text: &str, label: &str, max_rows: usize) -> Result<()> {
    if text.lines().count() > max_rows {
        bail!("{label} exceeds {max_rows} row limit");
    }
    Ok(())
}

pub(super) fn enforce_text(value: &str, label: &str, max_bytes: usize) -> Result<()> {
    if value.len() > max_bytes {
        bail!("{label} exceeds {max_bytes} byte limit");
    }
    Ok(())
}

pub(super) fn enforce_alignment_work(segments: usize, events: usize) -> Result<()> {
    let work = segments.checked_mul(events).unwrap_or(usize::MAX);
    if work > MAX_ALIGNMENT_WORK_ITEMS {
        bail!(
            "timeline alignment exceeds {} segment-event work item limit",
            MAX_ALIGNMENT_WORK_ITEMS
        );
    }
    Ok(())
}
