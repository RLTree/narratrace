use crate::safe_path::open_regular_file;
use anyhow::{Context, Result, bail};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct TrustedCoverageRun {
    pub report: Value,
    pub command: String,
    pub generated_at: String,
    pub provenance: Value,
}

pub fn execute(skill_dir: &Path) -> Result<TrustedCoverageRun> {
    let tool = find_tool("cargo-llvm-cov")?;
    let manifest = skill_dir.join("Cargo.toml");
    let args = [
        "llvm-cov".to_string(),
        "--manifest-path".to_string(),
        manifest.display().to_string(),
        "--json".to_string(),
        "--".to_string(),
        "--test-threads=1".to_string(),
    ];
    let output = Command::new(&tool)
        .args(&args)
        .current_dir(skill_dir)
        .env("NRR_TRUSTED_COVERAGE_CHILD", "1")
        .output()
        .with_context(|| format!("failed to run trusted coverage tool {}", tool.display()))?;
    if !output.status.success() {
        bail!(
            "trusted cargo-llvm-cov failed with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let report: Value = serde_json::from_slice(&output.stdout)
        .context("trusted cargo-llvm-cov stdout was not coverage JSON")?;
    let generated_at = trusted_timestamp()?;
    let command = format!("{} {}", tool.display(), args.join(" "));
    let report_sha256 = sha256(&output.stdout);
    let tool_sha256 = hash_file(&tool)?;
    let manifest_sha256 = hash_file(&manifest)?;
    let (source_tree_sha256, source_file_count) = hash_source_tree(&skill_dir.join("src"))?;
    let command_sha256 = sha256(command.as_bytes());
    let run_binding = bind(&[
        &command_sha256,
        &report_sha256,
        &tool_sha256,
        &manifest_sha256,
        &source_tree_sha256,
        &generated_at,
    ]);
    let provenance = json!({
        "schema": "narrated-record-replay.coverage-provenance.v1",
        "generator": "trusted-in-process-cargo-llvm-cov",
        "command_sha256": command_sha256,
        "report_sha256": report_sha256,
        "tool_path": tool.display().to_string(),
        "tool_sha256": tool_sha256,
        "manifest_sha256": manifest_sha256,
        "source_tree_sha256": source_tree_sha256,
        "source_file_count": source_file_count,
        "parent_process_id": std::process::id(),
        "run_binding_sha256": run_binding
    });
    Ok(TrustedCoverageRun {
        report,
        command,
        generated_at,
        provenance,
    })
}

pub fn validate(skill_dir: &Path, receipt: &Value) -> Result<()> {
    let provenance = receipt
        .pointer("/provenance")
        .ok_or_else(|| anyhow::anyhow!("coverage receipt lacks trusted provenance"))?;
    require(
        provenance,
        "/schema",
        "narrated-record-replay.coverage-provenance.v1",
    )?;
    require(
        provenance,
        "/generator",
        "trusted-in-process-cargo-llvm-cov",
    )?;
    let command = receipt["command"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("coverage receipt command missing"))?;
    let generated_at = receipt["generated_at"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("coverage receipt timestamp missing"))?;
    if !strict_utc_seconds(generated_at) {
        bail!("coverage receipt timestamp must be strict UTC RFC3339 seconds");
    }
    let tool = PathBuf::from(value_str(provenance, "/tool_path")?);
    if tool != find_tool("cargo-llvm-cov")? {
        bail!("coverage provenance tool path is not the allowlisted cargo-llvm-cov binary");
    }
    let (_, source_file_count) = hash_source_tree(&skill_dir.join("src"))?;
    let checks = [
        ("/command_sha256", sha256(command.as_bytes())),
        ("/tool_sha256", hash_file(&tool)?),
        (
            "/manifest_sha256",
            hash_file(&skill_dir.join("Cargo.toml"))?,
        ),
        (
            "/source_tree_sha256",
            hash_source_tree(&skill_dir.join("src"))?.0,
        ),
    ];
    for (pointer, expected) in checks {
        require(provenance, pointer, &expected)?;
    }
    if provenance["source_file_count"].as_u64() != Some(source_file_count as u64) {
        bail!("coverage provenance source file count no longer matches target tree");
    }
    let run_binding = bind(&[
        value_str(provenance, "/command_sha256")?,
        value_str(provenance, "/report_sha256")?,
        value_str(provenance, "/tool_sha256")?,
        value_str(provenance, "/manifest_sha256")?,
        value_str(provenance, "/source_tree_sha256")?,
        generated_at,
    ]);
    require(provenance, "/run_binding_sha256", &run_binding)
}

fn find_tool(name: &str) -> Result<PathBuf> {
    let home =
        option_env!("HOME").ok_or_else(|| anyhow::anyhow!("compile-time HOME is unavailable"))?;
    let tool = Path::new(home).join(".cargo/bin").join(name);
    if !tool.is_file() {
        bail!("allowlisted coverage tool is missing: {}", tool.display());
    }
    Ok(tool)
}

fn trusted_timestamp() -> Result<String> {
    let output = Command::new("/bin/date")
        .args(["-u", "+%Y-%m-%dT%H:%M:%SZ"])
        .output()?;
    if !output.status.success() {
        bail!("/bin/date failed while timestamping coverage");
    }
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

fn hash_source_tree(root: &Path) -> Result<(String, usize)> {
    let mut files = Vec::new();
    collect_files(root, root, &mut files)?;
    files.sort_by(|left, right| left.0.cmp(&right.0));
    let mut digest = Sha256::new();
    for (relative, path) in &files {
        digest.update(relative.as_bytes());
        digest.update([0]);
        digest.update(read_file(path)?);
        digest.update([0]);
    }
    Ok((format!("{:x}", digest.finalize()), files.len()))
}

fn collect_files(root: &Path, dir: &Path, files: &mut Vec<(String, PathBuf)>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path)?;
        if metadata.file_type().is_symlink() {
            bail!("coverage source tree contains symlink: {}", path.display());
        }
        if metadata.is_dir() {
            collect_files(root, &path, files)?;
        } else if metadata.is_file() {
            files.push((path.strip_prefix(root)?.display().to_string(), path));
        }
    }
    Ok(())
}

fn hash_file(path: &Path) -> Result<String> {
    Ok(sha256(&read_file(path)?))
}

fn read_file(path: &Path) -> Result<Vec<u8>> {
    let mut file = open_regular_file(path)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    Ok(bytes)
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn bind(values: &[&str]) -> String {
    sha256(values.join("\0").as_bytes())
}
fn value_str<'a>(value: &'a Value, pointer: &str) -> Result<&'a str> {
    value
        .pointer(pointer)
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("coverage provenance{pointer} must be a string"))
}
fn require(value: &Value, pointer: &str, expected: &str) -> Result<()> {
    if value_str(value, pointer)? != expected {
        bail!("coverage provenance{pointer} does not match trusted execution");
    }
    Ok(())
}

fn strict_utc_seconds(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 20
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[10] == b'T'
        && bytes[13] == b':'
        && bytes[16] == b':'
        && bytes[19] == b'Z'
        && bytes.iter().enumerate().all(|(index, byte)| {
            matches!(index, 4 | 7 | 10 | 13 | 16 | 19) || byte.is_ascii_digit()
        })
}

#[cfg(test)]
#[path = "provenance_test_support.rs"]
mod test_support;
#[cfg(test)]
pub use test_support::test_provenance;
