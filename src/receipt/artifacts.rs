use crate::safe_path::{open_regular_file, regular_file_metadata};
use anyhow::{Result, bail};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::{Path, PathBuf};

pub(super) const MAX_RECEIPT_ARTIFACT_BYTES: u64 = 64 * 1024 * 1024;
const MAX_RECEIPT_AGGREGATE_BYTES: u64 = 256 * 1024 * 1024;
const MAX_RECEIPT_JSONL_ROWS: u64 = 100_000;
const MAX_RECEIPT_JSONL_ROW_BYTES: usize = 1024 * 1024;

pub struct Artifact {
    name: &'static str,
    path: Option<PathBuf>,
}

pub fn artifact(name: &'static str, path: PathBuf) -> Artifact {
    Artifact {
        name,
        path: Some(path),
    }
}

pub fn optional_artifact(name: &'static str, path: Option<&str>) -> Artifact {
    Artifact {
        name,
        path: path.map(PathBuf::from),
    }
}

pub fn artifact_entries(artifacts: &[Artifact], policy: &str) -> Result<Vec<Value>> {
    let mut remaining = MAX_RECEIPT_AGGREGATE_BYTES;
    artifacts
        .iter()
        .map(|artifact| artifact_entry(artifact, policy, &mut remaining))
        .collect()
}

fn artifact_entry(artifact: &Artifact, policy: &str, remaining: &mut u64) -> Result<Value> {
    match &artifact.path {
        Some(path) => {
            let metadata = regular_file_metadata(path).ok();
            let stats = metadata
                .as_ref()
                .map(|_| file_stats(path, remaining))
                .transpose()?;
            Ok(json!({
                "name": artifact.name,
                "path": path.display().to_string(),
                "exists": metadata.is_some(),
                "isRegularFile": metadata.is_some(),
                "policy": policy,
                "bytes": stats.as_ref().map(|stats| stats.bytes),
                "lineCount": stats.as_ref().map(|stats| stats.line_count),
                "contentFingerprint": stats.map(|stats| stats.fingerprint)
            }))
        }
        None => Ok(json!({
            "name": artifact.name,
            "path": Value::Null,
            "exists": false,
            "policy": policy,
            "bytes": Value::Null,
            "lineCount": Value::Null,
            "contentFingerprint": Value::Null
        })),
    }
}

struct FileStats {
    bytes: u64,
    line_count: u64,
    fingerprint: String,
}

fn file_stats(path: &Path, remaining: &mut u64) -> Result<FileStats> {
    let mut file = open_regular_file(path)?;
    let len = file.metadata()?.len();
    if len > MAX_RECEIPT_ARTIFACT_BYTES || len > *remaining {
        bail!("receipt artifact byte budget exceeded: {}", path.display());
    }
    *remaining -= len;
    let validate_jsonl = path
        .extension()
        .is_some_and(|extension| extension == "jsonl");
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    let mut line = Vec::new();
    let mut line_count = 0_u64;
    let mut last = None;
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
        for byte in &buffer[..count] {
            last = Some(*byte);
            if *byte == b'\n' {
                line_count += 1;
                validate_jsonl_row(validate_jsonl, line_count, &line)?;
                line.clear();
            } else if validate_jsonl {
                if line.len() >= MAX_RECEIPT_JSONL_ROW_BYTES {
                    bail!("receipt JSONL row byte limit exceeded: {}", path.display());
                }
                line.push(*byte);
            }
        }
        if line_count > MAX_RECEIPT_JSONL_ROWS {
            bail!("receipt JSONL row limit exceeded: {}", path.display());
        }
    }
    if len > 0 && last != Some(b'\n') {
        line_count += 1;
        validate_jsonl_row(validate_jsonl, line_count, &line)?;
    }
    Ok(FileStats {
        bytes: len,
        line_count,
        fingerprint: format!("sha256:{:x}", hasher.finalize()),
    })
}

fn validate_jsonl_row(enabled: bool, row: u64, bytes: &[u8]) -> Result<()> {
    if enabled && !bytes.iter().all(u8::is_ascii_whitespace) {
        let text = std::str::from_utf8(bytes)?;
        serde_json::from_str::<Value>(text)
            .map_err(|error| anyhow::anyhow!("malformed receipt JSONL row {row}: {error}"))?;
    }
    Ok(())
}
