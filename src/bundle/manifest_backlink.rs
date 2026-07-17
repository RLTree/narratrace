use super::util::{read_json, required_string};
use anyhow::{Result, bail};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::Path;

pub(super) fn validate_backlog_manifest_backlink(
    backlog_path: &Path,
    manifest_path: &Path,
) -> Result<()> {
    let backlog = read_json(backlog_path)?;
    let digest = required_string(&backlog, "/manifest_digest", "VERIFICATION_BACKLOG.json")?;
    if !digest.starts_with("sha256:") {
        bail!("VERIFICATION_BACKLOG.json#/manifest_digest must be a sha256 digest");
    }
    if digest == "sha256:4fcb427e16d293e8f69378686e698492c00e5020fa7079991cdce98c845f27a9" {
        bail!("VERIFICATION_BACKLOG.json#/manifest_digest points at obsolete validator state");
    }
    let actual = normalized_manifest_digest(manifest_path)?;
    if digest != actual {
        bail!(
            "VERIFICATION_BACKLOG.json#/manifest_digest stale normalized manifest digest: expected {digest}, got {actual}"
        );
    }
    Ok(())
}

fn normalized_manifest_digest(manifest_path: &Path) -> Result<String> {
    let mut manifest = read_json(manifest_path)?;
    normalized_manifest_value_digest(&mut manifest)
}

pub(super) fn normalized_manifest_value_digest(manifest: &mut Value) -> Result<String> {
    manifest
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("COMPLETION_MANIFEST.json must be a JSON object"))?
        .remove("verification_backlog_digest");
    let bytes = serde_json::to_vec(manifest)?;
    Ok(format!("sha256:{:x}", Sha256::digest(bytes)))
}
