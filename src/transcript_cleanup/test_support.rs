#[cfg(test)]
fn cleanup_test_read_json(path: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}

#[cfg(test)]
fn cleanup_test_unique_tmp(prefix: &str) -> PathBuf {
    PathBuf::from("/private/tmp").join(format!(
        "{prefix}-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}
