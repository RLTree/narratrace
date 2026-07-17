use crate::config::Args;
use crate::safe_path::open_regular_file;
use anyhow::{Context, Result, bail};
use serde_json::Value;
use std::io::Read;
use std::path::Path;

pub fn check_coverage_policy(args: &Args) -> Result<()> {
    let skill_dir = args
        .skill_dir
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("--skill-dir is required"))?;
    validate_coverage_policy_at(skill_dir, args.coverage_receipt.as_deref())?;
    println!("coverage-policy ok");
    Ok(())
}

#[cfg(test)]
fn validate_coverage_policy(skill_dir: &Path) -> Result<()> {
    validate_coverage_policy_at(skill_dir, None)
}

fn validate_coverage_policy_at(skill_dir: &Path, receipt_override: Option<&Path>) -> Result<()> {
    let template = read_json(&skill_dir.join("COVERAGE_RECEIPT.json"))?;
    require_str(
        &template,
        "/target_schema",
        "COVERAGE_RECEIPT.json",
        Some("harness-ultragoal.coverage-receipt.v1"),
    )?;
    require_str(
        &template,
        "/canonical_output",
        "COVERAGE_RECEIPT.json",
        Some("validation_artifacts/coverage/coverage-receipt.json"),
    )?;

    let canonical = skill_dir.join("validation_artifacts/coverage/coverage-receipt.json");
    let receipt_path = receipt_override.unwrap_or(&canonical);
    if receipt_override.is_none()
        && !receipt_path.exists()
        && !skill_dir.join("validation_artifacts").exists()
    {
        return Ok(());
    }
    let receipt = read_json(&receipt_path)?;
    require_str(
        &receipt,
        "/schema",
        "coverage-receipt.json",
        Some("harness-ultragoal.coverage-receipt.v1"),
    )?;
    for pointer in ["/claim_id", "/command", "/tool", "/generated_at"] {
        require_str(&receipt, pointer, "coverage-receipt.json", None)?;
    }
    require_nonempty_array(&receipt, "/target_paths")?;
    require_nonempty_array(&receipt, "/measured_dimensions")?;
    let policy = require_str(&receipt, "/coverage/policy", "coverage-receipt.json", None)?;
    let ceiling = require_str(&receipt, "/claim_ceiling", "coverage-receipt.json", None)?;
    reject_test_count_substitution(&receipt)?;
    super::provenance::validate(skill_dir, &receipt)?;
    validate_coverage_claim(policy, ceiling, &receipt)?;
    validate_exclusions(&receipt)?;
    Ok(())
}

fn validate_coverage_claim(policy: &str, ceiling: &str, receipt: &Value) -> Result<()> {
    let percent = receipt
        .pointer("/coverage/percent")
        .and_then(Value::as_f64)
        .ok_or_else(|| anyhow::anyhow!("coverage-receipt.json#/coverage/percent must be number"))?;
    receipt
        .pointer("/coverage/floor_percent")
        .and_then(Value::as_f64)
        .ok_or_else(|| {
            anyhow::anyhow!("coverage-receipt.json#/coverage/floor_percent must be number")
        })?;
    match policy {
        "100_percent_required" => {
            if percent != 100.0 || ceiling != "supports_complete_claim" {
                bail!("100 percent coverage claims require 100.0 percent and complete ceiling");
            }
            if !receipt
                .pointer("/uncovered_records")
                .and_then(Value::as_array)
                .is_some_and(Vec::is_empty)
            {
                bail!("100 percent coverage claims must have no uncovered records");
            }
        }
        "ratchet_floor" => require_blocker_fields(receipt, "ratchet_floor", ceiling)?,
        "blocked_or_withheld" => require_blocker_fields(receipt, "blocked_or_withheld", ceiling)?,
        _ => bail!("coverage-receipt.json#/coverage/policy is invalid: {policy}"),
    }
    Ok(())
}

fn require_blocker_fields(receipt: &Value, policy: &str, ceiling: &str) -> Result<()> {
    let expected = if policy == "ratchet_floor" {
        "ratchet_floor_only"
    } else {
        "withheld_or_blocked"
    };
    if ceiling != expected {
        bail!("{policy} coverage requires claim_ceiling {expected}");
    }
    for pointer in [
        "/coverage/owner",
        "/coverage/reason",
        "/coverage/blocker_or_debt_id",
    ] {
        require_str(receipt, pointer, "coverage-receipt.json", None)?;
    }
    Ok(())
}

fn validate_exclusions(receipt: &Value) -> Result<()> {
    for exclusion in receipt
        .pointer("/exclusions")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow::anyhow!("coverage-receipt.json#/exclusions must be an array"))?
    {
        for pointer in ["/path", "/kind", "/rationale"] {
            require_str(exclusion, pointer, "coverage exclusion", None)?;
        }
        if exclusion
            .pointer("/counts_as_covered")
            .and_then(Value::as_bool)
            != Some(false)
        {
            bail!("coverage exclusions must not count as covered");
        }
        if exclusion.pointer("/reviewed").and_then(Value::as_bool) != Some(true) {
            bail!("coverage exclusions must be reviewed");
        }
    }
    Ok(())
}

fn reject_test_count_substitution(receipt: &Value) -> Result<()> {
    let command = require_str(receipt, "/command", "coverage-receipt.json", None)?;
    let tool = require_str(receipt, "/tool", "coverage-receipt.json", None)?;
    let text = format!(
        "{} {}",
        command.to_ascii_lowercase(),
        tool.to_ascii_lowercase()
    );
    if text.contains("cargo test") && !text.contains("coverage") && !text.contains("llvm-cov") {
        bail!("plain cargo test is test evidence, not coverage proof");
    }
    Ok(())
}

fn require_nonempty_array<'a>(value: &'a Value, pointer: &str) -> Result<&'a Vec<Value>> {
    let array = value
        .pointer(pointer)
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow::anyhow!("coverage-receipt.json#{pointer} must be an array"))?;
    if array.is_empty() {
        bail!("coverage-receipt.json#{pointer} must not be empty");
    }
    Ok(array)
}

fn require_str<'a>(
    value: &'a Value,
    pointer: &str,
    label: &str,
    expected: Option<&str>,
) -> Result<&'a str> {
    let actual = value
        .pointer(pointer)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| anyhow::anyhow!("{label}#{pointer} must be a non-empty string"))?;
    if let Some(expected) = expected
        && actual != expected
    {
        bail!("{label}#{pointer} must be {expected}");
    }
    Ok(actual)
}

fn read_json(path: &Path) -> Result<Value> {
    let mut file = open_regular_file(path)?;
    let mut text = String::new();
    file.read_to_string(&mut text)
        .with_context(|| format!("failed to read {}", path.display()))?;
    Ok(serde_json::from_str(&text)?)
}

#[cfg(test)]
#[path = "policy_extra_tests.rs"]
mod policy_extra_tests;
#[cfg(test)]
#[path = "policy_tests.rs"]
mod policy_tests;
