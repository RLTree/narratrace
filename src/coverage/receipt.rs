use super::provenance::{TrustedCoverageRun, execute};
use crate::config::Args;
use crate::private_fs::{write_private, write_private_new};
use anyhow::{Result, bail};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

pub fn write_coverage_receipt(args: &Args) -> Result<()> {
    let skill_dir = args
        .skill_dir
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("--skill-dir is required"))?;
    let canonical = skill_dir.join("validation_artifacts/coverage/coverage-receipt.json");
    let receipt_path = args.coverage_receipt.as_deref().unwrap_or(&canonical);
    if receipt_path != canonical {
        if !args.custom_runtime_path_consent {
            bail!(
                "custom --coverage-receipt requires --i-consent-to-custom-runtime-paths and must name a new file"
            );
        }
        if receipt_path.exists() {
            bail!(
                "custom --coverage-receipt refuses existing path; choose a new file: {}",
                receipt_path.display()
            );
        }
    }
    let run = execute(skill_dir)?;
    let receipt = serde_json::to_string_pretty(&build_receipt(skill_dir, &run)?)?;
    if receipt_path == canonical {
        write_private(receipt_path, receipt)?;
    } else {
        write_private_new(receipt_path, receipt)?;
    }
    println!("{}", receipt_path.display());
    Ok(())
}

fn build_receipt(skill_dir: &Path, run: &TrustedCoverageRun) -> Result<Value> {
    let data = run
        .report
        .pointer("/data/0")
        .ok_or_else(|| anyhow::anyhow!("llvm-cov JSON missing /data/0"))?;
    let percent = data
        .pointer("/totals/lines/percent")
        .and_then(Value::as_f64)
        .ok_or_else(|| anyhow::anyhow!("llvm-cov JSON missing line coverage percent"))?;
    let uncovered = uncovered_files(skill_dir, data)?;
    let complete = percent == 100.0 && uncovered.is_empty();
    let coverage = if complete {
        json!({"percent": percent, "floor_percent": 100.0, "policy": "100_percent_required"})
    } else {
        json!({
            "percent": percent,
            "floor_percent": 100.0,
            "policy": "blocked_or_withheld",
            "owner": "root",
            "reason": "Repo-owned Rust line coverage is below the required 100 percent floor.",
            "blocker_or_debt_id": "coverage-below-100-percent"
        })
    };
    Ok(json!({
        "schema": "harness-ultragoal.coverage-receipt.v1",
        "claim_id": "CLAIM-005",
        "command": run.command,
        "tool": "cargo-llvm-cov",
        "target_paths": ["src"],
        "measured_dimensions": ["line"],
        "coverage": coverage,
        "uncovered_records": uncovered,
        "exclusions": [],
        "generated_at": run.generated_at,
        "provenance": run.provenance,
        "claim_ceiling": if complete { "supports_complete_claim" } else { "withheld_or_blocked" }
    }))
}

fn uncovered_files(skill_dir: &Path, data: &Value) -> Result<Vec<Value>> {
    let mut rows = Vec::new();
    for file in data
        .get("files")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow::anyhow!("llvm-cov JSON missing files array"))?
    {
        let Some(percent) = file
            .pointer("/summary/lines/percent")
            .and_then(Value::as_f64)
        else {
            continue;
        };
        if percent == 100.0 {
            continue;
        }
        let path = file
            .get("filename")
            .and_then(Value::as_str)
            .map(PathBuf::from)
            .ok_or_else(|| anyhow::anyhow!("llvm-cov file row missing filename"))?;
        let relative = path
            .strip_prefix(skill_dir)
            .unwrap_or(path.as_path())
            .display()
            .to_string();
        rows.push(json!({
            "path": relative,
            "reason": format!("line coverage is {percent:.6}% in current cargo-llvm-cov report"),
            "owner": "root",
            "blocker_or_debt_id": "coverage-below-100-percent"
        }));
    }
    Ok(rows)
}

#[cfg(test)]
#[path = "receipt_extra_tests.rs"]
mod receipt_extra_tests;
#[cfg(test)]
#[path = "receipt_tests.rs"]
mod receipt_tests;
