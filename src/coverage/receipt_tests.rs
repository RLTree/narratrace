use super::*;

#[test]
fn lower_than_full_coverage_writes_blocked_receipt() {
    let root = Path::new("/tmp/repo");
    let receipt = build_receipt(root, &run(root, 55.0, 50.0)).unwrap();
    assert_eq!(receipt["coverage"]["policy"], "blocked_or_withheld");
    assert_eq!(receipt["claim_ceiling"], "withheld_or_blocked");
    assert_eq!(receipt["uncovered_records"][0]["path"], "src/main.rs");
}

#[test]
fn complete_coverage_receipt_supports_complete_claim() {
    let root = Path::new("/tmp/repo");
    let receipt = build_receipt(root, &run(root, 100.0, 100.0)).unwrap();
    assert_eq!(receipt["coverage"]["policy"], "100_percent_required");
    assert_eq!(receipt["claim_ceiling"], "supports_complete_claim");
    assert!(receipt["uncovered_records"].as_array().unwrap().is_empty());
    assert_eq!(receipt["command"], "trusted cargo llvm-cov");
    assert_eq!(receipt["generated_at"], "2026-01-01T00:00:00Z");
}

#[test]
fn write_coverage_receipt_requires_skill_dir_before_execution() {
    let args = crate::config::parse_args_from(["nrr", "coverage-receipt"]).unwrap();
    assert!(
        write_coverage_receipt(&args)
            .unwrap_err()
            .to_string()
            .contains("--skill-dir is required")
    );
}

#[test]
fn malformed_coverage_rows_report_shape_errors() {
    let mut trusted = run(Path::new("/tmp/repo"), 10.0, 10.0);
    trusted.report = json!({"data": [{"totals": {"lines": {}}, "files": []}]});
    assert!(build_receipt(Path::new("/tmp/repo"), &trusted).is_err());
    trusted.report = json!({"data": [{"totals": {"lines": {"percent": 10.0}}}]});
    assert!(build_receipt(Path::new("/tmp/repo"), &trusted).is_err());
}

#[test]
fn uncovered_files_skip_full_rows_and_keep_external_paths_absolute() {
    let data = json!({"files": [
        {"filename": "/tmp/repo/src/full.rs", "summary": {"lines": {"percent": 100.0}}},
        {"filename": "/tmp/repo/src/no-percent.rs", "summary": {"lines": {}}},
        {"filename": "/external/partial.rs", "summary": {"lines": {"percent": 80.0}}}
    ]});
    let rows = uncovered_files(Path::new("/tmp/repo"), &data).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["path"], "/external/partial.rs");
}

fn run(root: &Path, total: f64, file: f64) -> TrustedCoverageRun {
    TrustedCoverageRun {
        report: json!({"data": [{
            "totals": {"lines": {"percent": total}},
            "files": [{
                "filename": root.join("src/main.rs"),
                "summary": {"lines": {"percent": file}}
            }]
        }]}),
        command: "trusted cargo llvm-cov".into(),
        generated_at: "2026-01-01T00:00:00Z".into(),
        provenance: json!({"generator": "test-trusted-run"}),
    }
}
