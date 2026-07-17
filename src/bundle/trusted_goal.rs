#[cfg(test)]
use super::receipt::parse_strict_utc_seconds;
#[cfg(test)]
use super::util::assert_digest;
use super::util::required_string;
use anyhow::{Result, bail};
use serde_json::Value;
#[cfg(test)]
use sha2::{Digest, Sha256};
#[cfg(test)]
use std::path::Path;

#[cfg(test)]
const GOAL_SCHEMA: &str = "narrated-record-replay.trusted-goal-observation.v1";
#[cfg(test)]
const MAX_OBSERVATION_AGE_MS: i64 = 5 * 60 * 1_000;
#[cfg(test)]
const MAX_FUTURE_SKEW_MS: i64 = 60 * 1_000;

#[derive(Clone, Debug)]
pub(super) struct TrustedGoalObservation {
    value: Value,
    binding_digest: String,
}

impl TrustedGoalObservation {
    #[cfg(test)]
    pub(super) fn from_value_at(value: Value, skill_dir: &Path, now_ms: i64) -> Result<Self> {
        if value.get("schema").and_then(Value::as_str) != Some(GOAL_SCHEMA) {
            bail!("trusted goal observation must declare schema {GOAL_SCHEMA}");
        }
        for pointer in [
            "/observation_id",
            "/goal_id",
            "/objective",
            "/status",
            "/contract_path",
            "/contract_sha256",
            "/observed_at",
        ] {
            if required_string(&value, pointer, "trusted goal observation")?
                .trim()
                .is_empty()
            {
                bail!("trusted goal observation#{pointer} must not be empty");
            }
        }
        if required_string(&value, "/status", "trusted goal observation")? != "active" {
            bail!("trusted goal observation status must be active");
        }
        let expected_contract_path = skill_dir.join("GOAL_CONTRACT.md");
        if required_string(&value, "/contract_path", "trusted goal observation")?
            != expected_contract_path.to_string_lossy()
        {
            bail!(
                "trusted goal observation contract_path must name the validated GOAL_CONTRACT.md"
            );
        }
        let observed_at = required_string(&value, "/observed_at", "trusted goal observation")?;
        let observed_ms = parse_strict_utc_seconds(observed_at).ok_or_else(|| {
            anyhow::anyhow!("trusted goal observed_at must be strict UTC RFC3339 seconds")
        })?;
        let age_ms = now_ms
            .checked_sub(observed_ms)
            .ok_or_else(|| anyhow::anyhow!("trusted goal observation age overflow"))?;
        if age_ms > MAX_OBSERVATION_AGE_MS || age_ms < -MAX_FUTURE_SKEW_MS {
            bail!("trusted goal observation is stale or exceeds allowed future skew");
        }
        assert_digest(
            "trusted goal observation contract_sha256",
            &skill_dir.join("GOAL_CONTRACT.md"),
            required_string(&value, "/contract_sha256", "trusted goal observation")?,
        )?;
        let binding_digest = format!("sha256:{:x}", Sha256::digest(serde_json::to_vec(&value)?));
        Ok(Self {
            value,
            binding_digest,
        })
    }

    pub(super) fn validate_binding(&self, binding: &Value) -> Result<()> {
        for (binding_pointer, observation_pointer) in [
            ("/goal_id", "/goal_id"),
            ("/objective", "/objective"),
            ("/contract_path", "/contract_path"),
            ("/checked_at", "/observed_at"),
        ] {
            let actual = required_string(binding, binding_pointer, "goal binding")?;
            let expected =
                required_string(&self.value, observation_pointer, "trusted goal observation")?;
            if actual != expected {
                bail!(
                    "goal binding {binding_pointer} does not match trusted live goal observation"
                );
            }
        }
        if required_string(binding, "/status", "goal binding")? != "bound" {
            bail!("goal binding status must be bound");
        }
        Ok(())
    }

    pub(super) fn goal_id(&self) -> &str {
        self.value["goal_id"].as_str().expect("validated goal_id")
    }

    pub(super) fn observation_id(&self) -> &str {
        self.value["observation_id"]
            .as_str()
            .expect("validated observation_id")
    }

    pub(super) fn observed_at(&self) -> &str {
        self.value["observed_at"]
            .as_str()
            .expect("validated observed_at")
    }

    pub(super) fn binding_digest(&self) -> &str {
        &self.binding_digest
    }

    #[cfg(test)]
    pub(super) fn for_test() -> Self {
        let value = serde_json::json!({
            "observation_id": "test-observation",
            "goal_id": "test-goal",
            "objective": "test objective",
            "contract_path": "GOAL_CONTRACT.md",
            "observed_at": "2026-07-17T00:00:00Z"
        });
        let binding_digest = format!(
            "sha256:{:x}",
            Sha256::digest(serde_json::to_vec(&value).unwrap())
        );
        Self {
            value,
            binding_digest,
        }
    }
}
