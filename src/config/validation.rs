use super::{Args, DEFAULT_AUDIO_FILTER};
use crate::safe_path::validate_private_runtime_path;
use anyhow::Result;
use std::path::PathBuf;

pub(super) fn validate_args(args: &Args) -> Result<()> {
    validate_private_runtime_path("--root", &args.root, args.custom_runtime_path_consent)?;
    if let Some(session_dir) = &args.session_dir {
        validate_private_runtime_path(
            "--session-dir",
            session_dir,
            args.custom_runtime_path_consent,
        )?;
    }
    if let Some(path) = &args.audio_retention_path {
        validate_private_runtime_path(
            "--audio-retention-path",
            path,
            args.custom_runtime_path_consent,
        )?;
    }
    if let (Some(skill_dir), Some(receipt_path)) = (&args.skill_dir, &args.coverage_receipt) {
        let canonical = skill_dir.join("validation_artifacts/coverage/coverage-receipt.json");
        if receipt_path != &canonical && !args.custom_runtime_path_consent {
            anyhow::bail!(
                "custom --coverage-receipt requires --i-consent-to-custom-runtime-paths: {}",
                receipt_path.display()
            );
        }
    }
    if args.audio_filter != DEFAULT_AUDIO_FILTER && !args.custom_audio_filter_consent {
        anyhow::bail!(
            "custom --audio-filter requires --i-consent-to-custom-audio-filter; use the default filter for auto-approved runs"
        );
    }
    Ok(())
}

#[cfg(test)]
#[path = "validation_tests.rs"]
mod tests;

pub fn required_session_dir(args: &Args) -> Result<PathBuf> {
    args.session_dir
        .clone()
        .ok_or_else(|| anyhow::anyhow!("--session-dir is required"))
}
