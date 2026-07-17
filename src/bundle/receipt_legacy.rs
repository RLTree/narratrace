use super::receipt::{validate_check_rows, validate_receipt_timestamp_shape};
use super::util::{read_json, require_schema, required_string};
use super::{LOCAL_VALIDATOR_NAME, LOCAL_VALIDATOR_SCHEMA, LOCAL_VALIDATOR_VERSION};
use anyhow::{Result, bail};
use std::path::Path;

pub(super) fn validate_legacy_receipt(
    path: &Path,
    expected_skill_dir: &Path,
    expected_repo_root: &Path,
    expected_commit: &str,
) -> Result<()> {
    let receipt = read_json(path)?;
    require_schema(
        &receipt,
        &path.display().to_string(),
        LOCAL_VALIDATOR_SCHEMA,
    )?;
    let skill_dir = expected_skill_dir.to_string_lossy();
    let repo_root = expected_repo_root.to_string_lossy();
    for (pointer, expected) in [
        ("/validator", LOCAL_VALIDATOR_NAME),
        ("/version", LOCAL_VALIDATOR_VERSION),
        ("/status", "passed"),
        ("/target/skill_dir", skill_dir.as_ref()),
        ("/target/repo_root", repo_root.as_ref()),
        ("/root", skill_dir.as_ref()),
        ("/commit", expected_commit),
    ] {
        let actual = required_string(&receipt, pointer, "legacy local validator receipt")?;
        if actual != expected {
            bail!("legacy local validator receipt expected {pointer}={expected}, got {actual}");
        }
    }
    if required_string(&receipt, "/run_id", "legacy local validator receipt")?
        .trim()
        .is_empty()
    {
        bail!("legacy local validator receipt run_id must not be empty");
    }
    validate_receipt_timestamp_shape(&receipt)?;
    validate_check_rows(&receipt)
}
