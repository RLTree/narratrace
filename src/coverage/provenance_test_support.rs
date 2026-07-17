use super::*;

pub fn test_provenance(skill_dir: &Path, command: &str, generated_at: &str) -> Value {
    fs::create_dir_all(skill_dir.join("src")).unwrap();
    if !skill_dir.join("Cargo.toml").exists() {
        fs::write(
            skill_dir.join("Cargo.toml"),
            "[package]\nname='fixture'\nversion='0.0.0'\n",
        )
        .unwrap();
    }
    if !skill_dir.join("src/lib.rs").exists() {
        fs::write(skill_dir.join("src/lib.rs"), "pub fn fixture() {}\n").unwrap();
    }
    let tool = find_tool("cargo-llvm-cov").unwrap();
    let command_sha256 = sha256(command.as_bytes());
    let report_sha256 = sha256(b"test-report");
    let tool_sha256 = hash_file(&tool).unwrap();
    let manifest_sha256 = hash_file(&skill_dir.join("Cargo.toml")).unwrap();
    let (source_tree_sha256, count) = hash_source_tree(&skill_dir.join("src")).unwrap();
    let binding = bind(&[
        &command_sha256,
        &report_sha256,
        &tool_sha256,
        &manifest_sha256,
        &source_tree_sha256,
        generated_at,
    ]);
    json!({
        "schema": "narrated-record-replay.coverage-provenance.v1",
        "generator": "trusted-in-process-cargo-llvm-cov",
        "command_sha256": command_sha256,
        "report_sha256": report_sha256,
        "tool_path": tool.display().to_string(),
        "tool_sha256": tool_sha256,
        "manifest_sha256": manifest_sha256,
        "source_tree_sha256": source_tree_sha256,
        "source_file_count": count,
        "parent_process_id": std::process::id(),
        "run_binding_sha256": binding
    })
}
