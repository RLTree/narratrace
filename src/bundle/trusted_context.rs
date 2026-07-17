use super::source_identity::SourceIdentity;
use super::trusted_goal::TrustedGoalObservation;
use anyhow::{Result, bail};
use std::path::Path;

#[derive(Clone, Debug)]
pub(super) struct TrustedBundleContext {
    pub(super) source: SourceIdentity,
    pub(super) goal: TrustedGoalObservation,
}

impl TrustedBundleContext {
    pub(super) fn from_trusted_services(skill_dir: &Path) -> Result<Self> {
        let environment_candidate = std::env::var("NARRATED_REPLAY_TRUSTED_GOAL_OBSERVATION").ok();
        let goal = authenticated_goal_observation(environment_candidate.as_deref())?;
        Ok(Self {
            source: SourceIdentity::measure(skill_dir)?,
            goal,
        })
    }

    #[cfg(test)]
    pub(super) fn for_test(source: SourceIdentity, goal: TrustedGoalObservation) -> Self {
        Self { source, goal }
    }
}

fn authenticated_goal_observation(
    environment_candidate: Option<&str>,
) -> Result<TrustedGoalObservation> {
    if environment_candidate.is_some() {
        bail!("caller-controlled environment goal observations are not accepted");
    }
    bail!("trusted goal-service attestation unavailable")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_perfectly_shaped_fresh_environment_observation() {
        let candidate = serde_json::json!({
            "schema": "narrated-record-replay.trusted-goal-observation.v1",
            "observation_id": "attacker-fresh",
            "goal_id": "attacker-goal",
            "objective": "attacker objective",
            "status": "active",
            "contract_path": "/target/GOAL_CONTRACT.md",
            "contract_sha256": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "observed_at": "2026-07-17T12:00:00Z"
        })
        .to_string();

        let error = authenticated_goal_observation(Some(&candidate))
            .unwrap_err()
            .to_string();
        assert!(error.contains("environment goal observations are not accepted"));
    }
}
