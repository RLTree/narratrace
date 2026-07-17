use crate::redaction;
use crate::safe_path::{
    normalize_system_temp, open_regular_file, read_regular_text_bounded, regular_file_metadata,
    validate_cli_path,
};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

const MAX_SCAN_BYTES_PER_FILE: u64 = 8 * 1024 * 1024;
const MAX_SCAN_BYTES_AGGREGATE: u64 = 32 * 1024 * 1024;
const MAX_STATS_BYTES_PER_FILE: u64 = 512 * 1024 * 1024;
const MAX_STATS_BYTES_AGGREGATE: u64 = 1024 * 1024 * 1024;

pub(super) struct ArtifactSpec<'a> {
    name: &'static str,
    path: &'a Path,
    policy: &'static str,
}

pub(super) fn artifact_spec<'a>(
    name: &'static str,
    path: &'a Path,
    policy: &'static str,
) -> ArtifactSpec<'a> {
    ArtifactSpec { name, path, policy }
}

pub(super) fn artifact_entries(artifacts: &[ArtifactSpec<'_>]) -> Vec<Value> {
    artifacts.iter().map(artifact_entry).collect()
}

pub(super) fn raw_local_entries(artifacts: &[ArtifactSpec<'_>]) -> Vec<Value> {
    let mut stats_budget = MAX_STATS_BYTES_AGGREGATE;
    let mut scan_budget = MAX_SCAN_BYTES_AGGREGATE;
    artifacts
        .iter()
        .map(|artifact| raw_local_entry_with_budget(artifact, &mut stats_budget, &mut scan_budget))
        .collect()
}

pub(super) fn leak_scan(artifacts: &[ArtifactSpec<'_>], approved_paths: &[PathBuf]) -> Value {
    let mut remaining = MAX_SCAN_BYTES_AGGREGATE;
    let findings = artifacts
        .iter()
        .filter_map(|artifact| leak_finding(artifact, approved_paths, &mut remaining))
        .collect::<Vec<_>>();
    let blocked = findings.iter().any(|finding| {
        finding
            .get("blocksShare")
            .and_then(Value::as_bool)
            .unwrap_or(true)
    });
    json!({
        "status": if blocked {
            "blocked"
        } else if findings.is_empty() {
            "no-obvious-sensitive-patterns-detected"
        } else {
            "expected-local-references-only"
        },
        "scanner": "generated-artifact-pattern-scan-v1",
        "artifactCount": artifacts.len(),
        "findings": findings,
        "claimCeiling": "pattern scan only; operator privacy review is still required before sharing"
    })
}

fn artifact_entry(artifact: &ArtifactSpec<'_>) -> Value {
    json!({
        "name": artifact.name,
        "path": artifact.path.display().to_string(),
        "exists": artifact.path.exists(),
        "policy": artifact.policy
    })
}

#[cfg(test)]
fn raw_local_entry(artifact: &ArtifactSpec<'_>) -> Value {
    let mut stats_budget = MAX_STATS_BYTES_AGGREGATE;
    let mut scan_budget = MAX_SCAN_BYTES_AGGREGATE;
    raw_local_entry_with_budget(artifact, &mut stats_budget, &mut scan_budget)
}

fn raw_local_entry_with_budget(
    artifact: &ArtifactSpec<'_>,
    stats_budget: &mut u64,
    scan_budget: &mut u64,
) -> Value {
    let inspected = inspect_raw_artifact(artifact, stats_budget, scan_budget);
    let (stats, categories, safe_regular_file) = match inspected {
        Ok((stats, categories)) => (Some(stats), categories, true),
        Err(_) if !artifact.path.exists() => (None, Vec::new(), false),
        Err(_) => (None, vec!["unsafe-artifact-path"], false),
    };
    let contains_sensitive_patterns = !categories.is_empty();
    json!({
        "name": artifact.name,
        "path": artifact.path.display().to_string(),
        "exists": artifact.path.exists(),
        "safeRegularFile": safe_regular_file,
        "policy": artifact.policy,
        "bytes": stats.as_ref().map(|stats| stats.bytes),
        "lineCount": stats.as_ref().map(|stats| stats.line_count),
        "contentFingerprint": stats.as_ref().map(|stats| stats.fingerprint.as_str()),
        "sensitiveCategories": categories,
        "containsSensitivePatterns": contains_sensitive_patterns
    })
}

fn leak_finding(
    artifact: &ArtifactSpec<'_>,
    approved_paths: &[PathBuf],
    remaining: &mut u64,
) -> Option<Value> {
    let text = match read_regular_text_bounded(artifact.path, MAX_SCAN_BYTES_PER_FILE) {
        Ok(text) => text,
        Err(error) => {
            if regular_file_metadata(artifact.path).is_err() {
                return Some(blocked_path_finding(artifact, &error.to_string()));
            }
            let disposition = if error.to_string().contains("byte limit") {
                "artifact-inspection-budget-exceeded"
            } else {
                "artifact-read-or-decode-failed"
            };
            return Some(blocked_read_finding(artifact, disposition));
        }
    };
    if text.len() as u64 > *remaining {
        return Some(blocked_read_finding(artifact, "artifact-inspection-budget-exceeded"));
    }
    *remaining -= text.len() as u64;
    let mut categories = redaction::sensitive_categories(&text);
    if text.split_whitespace().any(|token| private_path_core(token).is_some())
        && !categories.contains(&"private-path")
    {
        categories.push("private-path");
    }
    if categories.is_empty() {
        return None;
    }
    let expected_local_paths =
        expected_local_artifact_paths_only(&text, artifact.path, approved_paths);
    let blocks_share = categories.iter().any(|category| match *category {
        "opaque-token" => false,
        "private-path" => !expected_local_paths,
        _ => true,
    });
    Some(json!({
        "artifact": artifact.name,
        "path": artifact.path.display().to_string(),
        "categories": categories,
        "disposition": if blocks_share {
            "unresolved-sensitive-content"
        } else if expected_local_paths {
            "expected-local-artifact-reference"
        } else {
            "expected-local-opaque-token-reference"
        },
        "blocksShare": blocks_share
    }))
}

fn blocked_path_finding(artifact: &ArtifactSpec<'_>, error: &str) -> Value {
    json!({
        "artifact": artifact.name,
        "path": artifact.path.display().to_string(),
        "categories": ["unsafe-artifact-path"],
        "disposition": format!("artifact-read-blocked: {error}"),
        "blocksShare": true
    })
}

fn blocked_read_finding(artifact: &ArtifactSpec<'_>, disposition: &str) -> Value {
    json!({
        "artifact": artifact.name,
        "path": artifact.path.display().to_string(),
        "categories": ["unsafe-artifact-content"],
        "disposition": disposition,
        "blocksShare": true
    })
}

fn expected_local_artifact_paths_only(
    text: &str,
    artifact_path: &Path,
    approved_paths: &[PathBuf],
) -> bool {
    let Some(session_dir) = artifact_path.parent() else {
        return false;
    };
    let mut saw_private_path = false;
    for token in text.split_whitespace() {
        let Some(core) = private_path_core(token) else {
            continue;
        };
        saw_private_path = true;
        let path = normalize_system_temp(Path::new(core));
        let session_dir = normalize_system_temp(session_dir);
        if validate_cli_path("generated artifact reference", &path).is_ok()
            && (path.starts_with(&session_dir)
                || approved_paths
                    .iter()
                    .map(|approved| normalize_system_temp(approved))
                    .any(|approved| path == approved))
        {
            continue;
        }
        return false;
    }
    saw_private_path
}

fn private_path_core(token: &str) -> Option<&str> {
    let core = token.trim_matches(|ch: char| {
        ch.is_ascii_punctuation() && !matches!(ch, '/' | '~' | '.' | '_' | '-')
    });
    if core.starts_with("/Users/")
        || core.starts_with("/home/")
        || core.starts_with("~/")
        || core.starts_with("/private/var/")
        || core.starts_with("/var/folders/")
        || core.starts_with("/private/tmp/")
    {
        Some(core)
    } else {
        None
    }
}
