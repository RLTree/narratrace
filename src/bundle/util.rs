use super::REQUIRED_CLAIM_COUNT;
use crate::safe_path::{open_regular_file, read_regular_text_bounded};
use anyhow::{Context, Result, bail};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::{Component, Path, PathBuf};

const MAX_BUNDLE_ARTIFACT_BYTES: u64 = 8 * 1024 * 1024;

pub(super) fn require_schema(value: &Value, file: &str, expected: &str) -> Result<()> {
    if value.get("schema").and_then(Value::as_str) != Some(expected) {
        bail!("{file} must declare schema {expected}");
    }
    Ok(())
}

pub(super) fn require_array<'a>(
    value: &'a Value,
    pointer: &str,
    file: &str,
) -> Result<&'a Vec<Value>> {
    value
        .pointer(pointer)
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow::anyhow!("{file}#{pointer} must be an array"))
}

pub(super) fn bundle_artifact_path(repo_root: &Path, value: &str) -> Result<PathBuf> {
    let relative = Path::new(value);
    if relative.is_absolute() {
        bail!("bundle artifact path must be repo-relative, got absolute path: {value}");
    }
    let mut out = PathBuf::new();
    for component in relative.components() {
        match component {
            Component::Normal(part) => out.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                bail!("bundle artifact path must not escape repo root: {value}")
            }
        }
    }
    if out.as_os_str().is_empty() {
        bail!("bundle artifact path must not be empty");
    }
    if repo_root.join(".codex-plugin/plugin.json").is_file() {
        if let Ok(suffix) = out.strip_prefix(".codex/skills/narrated-record-replay") {
            return Ok(repo_root
                .join("skills")
                .join("narrated-record-replay")
                .join(suffix));
        }
    }
    Ok(repo_root.join(out))
}

pub(super) fn required_string<'a>(
    value: &'a Value,
    pointer: &str,
    context: &str,
) -> Result<&'a str> {
    value
        .pointer(pointer)
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("{context}#{pointer} must be a string"))
}

pub(super) fn assert_digest(pointer: &str, path: &Path, expected: &str) -> Result<()> {
    let actual = format!("sha256:{}", sha256_file_hex(path)?);
    if actual != expected {
        bail!(
            "{pointer} stale digest for {}: expected {expected}, got {actual}",
            path.display()
        );
    }
    Ok(())
}

fn sha256_file_hex(path: &Path) -> Result<String> {
    let mut file = open_regular_file(path)?;
    if file.metadata()?.len() > MAX_BUNDLE_ARTIFACT_BYTES {
        bail!("bundle artifact exceeds byte limit: {}", path.display());
    }
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

pub(super) fn read_json(path: &Path) -> Result<Value> {
    Ok(serde_json::from_str(&read_text(path)?)?)
}

pub(super) fn read_text(path: &Path) -> Result<String> {
    read_regular_text_bounded(path, MAX_BUNDLE_ARTIFACT_BYTES)
        .with_context(|| format!("failed to read {} as utf-8 (bounded)", path.display()))
}

pub(super) fn required_claim_ids() -> Vec<String> {
    (1..=REQUIRED_CLAIM_COUNT)
        .map(|index| format!("CLAIM-{index:03}"))
        .collect()
}

pub(super) fn required_claim_ids_hash() -> &'static str {
    "sha256:aca912de73b9642db7c553592fd158f46bde2562814839a2ee20449ea446f1e5"
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[cfg(unix)]
    #[test]
    fn read_text_rejects_symlinked_bundle_path_ancestor() {
        let root = unique_tmp("nrr-bundle-symlink-ancestor");
        fs::create_dir_all(&root).unwrap();
        let outside = unique_tmp("nrr-bundle-outside");
        fs::create_dir_all(&outside).unwrap();
        fs::write(outside.join("secret.json"), "{}").unwrap();
        std::os::unix::fs::symlink(&outside, root.join("linked")).unwrap();

        let result = read_text(&root.join("linked/secret.json"));

        assert!(result.is_err());
    }

    #[test]
    fn bundle_reader_and_hasher_reject_oversized_artifact() {
        let root = unique_tmp("nrr-bundle-oversize");
        fs::create_dir_all(&root).unwrap();
        let path = root.join("large.json");
        fs::File::create(&path)
            .unwrap()
            .set_len(MAX_BUNDLE_ARTIFACT_BYTES + 1)
            .unwrap();
        assert!(read_text(&path).is_err());
        assert!(sha256_file_hex(&path).is_err());
    }

    #[cfg(unix)]
    fn unique_tmp(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}-{nanos}"))
    }
}
