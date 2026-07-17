#[cfg(test)]
struct LoadedCleanupFixture {
    value: Value,
    sha256: String,
}

#[cfg(test)]
fn load_test_fixture(
    session_dir: &Path,
    variable: &str,
    label: &str,
) -> Result<Option<LoadedCleanupFixture>> {
    let Ok(raw) = std::env::var(variable) else {
        return Ok(None);
    };
    let fixture_path = crate::safe_path::normalize_system_temp(Path::new(&raw));
    let session_dir = crate::safe_path::normalize_system_temp(session_dir);
    if !fixture_path.starts_with(&session_dir) {
        bail!("{label} must stay inside the current test session");
    }
    regular_file_metadata(&fixture_path).with_context(|| format!("{label} not readable: {raw}"))?;
    let bytes = read_regular_bytes_bounded(&fixture_path, MAX_JSON_ARTIFACT_BYTES)?;
    Ok(Some(LoadedCleanupFixture {
        value: serde_json::from_slice(&bytes)?,
        sha256: sha256_bytes(&bytes),
    }))
}
