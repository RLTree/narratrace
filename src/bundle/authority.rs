use super::util::read_text;
use anyhow::{Result, bail};
use std::path::Path;

pub(super) fn validate_rust_only_authority(skill_dir: &Path) -> Result<()> {
    let authority_paths = [
        "AGENT_STANDARDS.md",
        "ARCHITECTURE.md",
        "COMPLETION_MANIFEST.json",
        "REFERENCES.md",
        "SKILL.md",
        "VALIDATION.md",
        "VERIFICATION_BACKLOG.json",
        "scripts/check",
    ];
    for relative in authority_paths {
        let path = skill_dir.join(relative);
        let text = read_text(&path)?;
        for forbidden in ["python3", "pytest", "__pycache__", "validate_bundle.py"] {
            if text.contains(forbidden) {
                bail!("{relative} contains retired Python authority token: {forbidden}");
            }
        }
    }
    Ok(())
}
