use crate::safe_path::read_regular_file_bounded;
use anyhow::{Context, Result, bail};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

const MAX_SOURCE_FILES: usize = 2_048;
const MAX_SOURCE_FILE_BYTES: u64 = 8 * 1024 * 1024;
const MAX_SOURCE_TOTAL_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct SourceIdentity {
    digest: String,
}

impl SourceIdentity {
    pub(super) fn measure(skill_dir: &Path) -> Result<Self> {
        let mut paths = vec![PathBuf::from("Cargo.toml"), PathBuf::from("Cargo.lock")];
        collect_source_paths(skill_dir, Path::new("src"), &mut paths)?;
        paths.sort();
        if paths.len() > MAX_SOURCE_FILES {
            bail!("source snapshot exceeds {MAX_SOURCE_FILES} files");
        }
        let mut total = 0_u64;
        let mut hasher = Sha256::new();
        hasher.update(b"narrated-record-replay-source-v1\0");
        for relative in paths {
            let relative_text = relative
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("source path is not valid UTF-8"))?;
            let bytes =
                read_regular_file_bounded(&skill_dir.join(&relative), MAX_SOURCE_FILE_BYTES)?;
            total = total
                .checked_add(bytes.len() as u64)
                .ok_or_else(|| anyhow::anyhow!("source snapshot size overflow"))?;
            if total > MAX_SOURCE_TOTAL_BYTES {
                bail!("source snapshot exceeds {MAX_SOURCE_TOTAL_BYTES} bytes");
            }
            update_length_prefixed(&mut hasher, relative_text.as_bytes());
            update_length_prefixed(&mut hasher, &bytes);
        }
        Ok(Self {
            digest: format!("sha256:{:x}", hasher.finalize()),
        })
    }

    pub(super) fn digest(&self) -> &str {
        &self.digest
    }

    #[cfg(test)]
    pub(super) fn for_test(digest: &str) -> Self {
        Self {
            digest: digest.to_string(),
        }
    }

    pub(super) fn validate_manifest(&self, manifest: &Value) -> Result<()> {
        let declared =
            super::util::required_string(manifest, "/commit", "COMPLETION_MANIFEST.json")?;
        if declared != self.digest {
            bail!(
                "COMPLETION_MANIFEST.json#/commit must equal measured source identity: expected {}, got {declared}",
                self.digest
            );
        }
        Ok(())
    }

    pub(super) fn assert_unchanged(&self, skill_dir: &Path) -> Result<()> {
        let current = Self::measure(skill_dir)?;
        if current != *self {
            bail!("source identity changed during bundle validation");
        }
        Ok(())
    }
}

fn collect_source_paths(root: &Path, relative: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    let directory = root.join(relative);
    for entry in fs::read_dir(&directory)
        .with_context(|| format!("failed to enumerate {}", directory.display()))?
    {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let child = relative.join(entry.file_name());
        if file_type.is_symlink() {
            bail!(
                "source snapshot must not follow symlink: {}",
                child.display()
            );
        }
        if file_type.is_dir() {
            collect_source_paths(root, &child, out)?;
        } else if file_type.is_file() {
            out.push(child);
        } else {
            bail!(
                "source snapshot contains unsupported file type: {}",
                child.display()
            );
        }
    }
    Ok(())
}

fn update_length_prefixed(hasher: &mut Sha256, bytes: &[u8]) {
    hasher.update((bytes.len() as u64).to_be_bytes());
    hasher.update(bytes);
}
