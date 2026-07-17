use super::util::{require_array, required_string};
use anyhow::{Result, bail};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ClaimDisposition {
    ProvenStatic,
    NonProven,
}

pub(super) fn validate_claim(value: &Value, index: usize) -> Result<()> {
    for key in [
        "id",
        "title",
        "claim_scope",
        "claim_kind",
        "status",
        "claim_surface",
        "claim_ceiling_effect",
    ] {
        if value.get(key).and_then(Value::as_str).is_none() {
            bail!("COMPLETION_MANIFEST.json#/claims/{index} missing string {key}");
        }
    }
    let claim_id = required_string(value, "/id", "completion manifest claim")?;
    if required_string(value, "/claim_scope", "completion manifest claim")? != "required" {
        bail!("claim {claim_id} must remain in required scope");
    }
    match required_string(value, "/claim_kind", "completion manifest claim")? {
        "feature_completion" | "integration" | "process_compliance" | "security"
        | "static_contract" | "ui_surface" => {}
        other => bail!("unsupported claim kind: {other}"),
    }
    validate_surface(required_string(
        value,
        "/claim_surface",
        "completion manifest claim",
    )?)?;
    match required_string(value, "/claim_ceiling_effect", "completion manifest claim")? {
        "satisfied" | "degraded" | "blocked" | "withheld" => {}
        other => bail!("unsupported claim ceiling effect: {other}"),
    }
    let status = parse_status(required_string(
        value,
        "/status",
        "completion manifest claim",
    )?)?;
    let allowed = parse_allowed_surfaces(value, index)?;
    let evidence = require_array(value, "/evidence", "completion manifest claim")?;
    match status {
        ClaimDisposition::ProvenStatic if evidence.is_empty() => {
            bail!("proven claim {claim_id} must include verified evidence")
        }
        ClaimDisposition::NonProven => {
            let backlog = required_string(value, "/backlog_row_id", "non-proven claim")?;
            if backlog.trim().is_empty() {
                bail!("non-proven claim {claim_id} must bind a backlog row");
            }
        }
        ClaimDisposition::ProvenStatic => {}
    }
    for (evidence_index, item) in evidence.iter().enumerate() {
        for key in ["id", "claim_id", "kind", "surface", "path", "digest"] {
            if item.get(key).and_then(Value::as_str).is_none() {
                bail!(
                    "COMPLETION_MANIFEST.json#/claims/{index}/evidence/{evidence_index} missing string {key}"
                );
            }
        }
        if required_string(item, "/claim_id", "claim evidence")? != claim_id {
            bail!("claim evidence claim_id does not match containing claim {claim_id}");
        }
        let surface = required_string(item, "/surface", "claim evidence")?;
        if !allowed.contains(surface) {
            bail!("claim evidence surface {surface} is not allowed for {claim_id}");
        }
        validate_digest(required_string(item, "/digest", "claim evidence")?)?;
        if required_string(item, "/path", "claim evidence")?
            .trim()
            .is_empty()
        {
            bail!("claim evidence path must not be empty");
        }
        match required_string(item, "/kind", "claim evidence")? {
            "static_check" | "runtime_execution" => {}
            other => bail!("unsupported claim evidence kind: {other}"),
        }
    }
    Ok(())
}

pub(super) fn validate_backlog_bindings(manifest: &Value, backlog: &Value) -> Result<()> {
    let mut rows = BTreeMap::new();
    for row in require_array(backlog, "/rows", "VERIFICATION_BACKLOG.json")? {
        let id = required_string(row, "/id", "verification backlog row")?;
        let claim_id = required_string(row, "/claim_id", "verification backlog row")?;
        if rows.insert(id, claim_id).is_some() {
            bail!("duplicate verification backlog row id: {id}");
        }
    }
    for claim in require_array(manifest, "/claims", "COMPLETION_MANIFEST.json")? {
        if parse_status(required_string(
            claim,
            "/status",
            "completion manifest claim",
        )?)? == ClaimDisposition::NonProven
        {
            let claim_id = required_string(claim, "/id", "completion manifest claim")?;
            let backlog_id = required_string(claim, "/backlog_row_id", "non-proven claim")?;
            if rows.get(backlog_id).copied() != Some(claim_id) {
                bail!("backlog row {backlog_id} is not bound to claim {claim_id}");
            }
        }
    }
    Ok(())
}

pub(super) fn validate_manifest_claim_ceiling(
    manifest: &Value,
    amendments: &[Value],
) -> Result<()> {
    let proven = require_array(manifest, "/claims", "COMPLETION_MANIFEST.json")?
        .iter()
        .filter_map(|claim| {
            (claim.get("status").and_then(Value::as_str) == Some("proven_static"))
                .then(|| claim.get("id").and_then(Value::as_str).map(str::to_string))
                .flatten()
        })
        .collect::<BTreeSet<_>>();
    let latest = amendments
        .last()
        .ok_or_else(|| anyhow::anyhow!("AMENDMENTS.jsonl must contain at least one row"))?;
    let declared = require_array(latest, "/after_claim_ceiling", "latest amendment")?
        .iter()
        .map(|value| {
            value.as_str().map(str::to_string).ok_or_else(|| {
                anyhow::anyhow!("latest amendment claim ceiling must contain strings")
            })
        })
        .collect::<Result<BTreeSet<_>>>()?;
    if proven != declared {
        bail!("latest amendment claim ceiling does not match manifest proven_static claims");
    }
    Ok(())
}

fn parse_status(value: &str) -> Result<ClaimDisposition> {
    match value {
        "proven_static" => Ok(ClaimDisposition::ProvenStatic),
        "contract_only" | "simulated_only" | "lane_owed" | "root_owed" | "externally_blocked"
        | "withheld_claim" | "unsupported" => Ok(ClaimDisposition::NonProven),
        other => bail!("unsupported completion claim status: {other}"),
    }
}

fn parse_allowed_surfaces(value: &Value, index: usize) -> Result<BTreeSet<&str>> {
    let mut allowed = BTreeSet::new();
    for surface in require_array(
        value,
        "/allowed_evidence_surfaces",
        "completion manifest claim",
    )? {
        let surface = surface.as_str().ok_or_else(|| {
            anyhow::anyhow!(
                "COMPLETION_MANIFEST.json#/claims/{index}/allowed_evidence_surfaces must contain strings"
            )
        })?;
        validate_surface(surface)?;
        allowed.insert(surface);
    }
    Ok(allowed)
}

fn validate_surface(surface: &str) -> Result<()> {
    match surface {
        "static" | "runtime_cli" | "runtime_api" | "ui_computer" | "observability"
        | "ui_browser" | "product_cohesion" => Ok(()),
        other => bail!("unsupported evidence surface: {other}"),
    }
}

fn validate_digest(value: &str) -> Result<()> {
    let Some(hex) = value.strip_prefix("sha256:") else {
        bail!("claim evidence digest must use sha256");
    };
    if hex.len() != 64 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("claim evidence digest must contain 64 hexadecimal characters");
    }
    Ok(())
}
