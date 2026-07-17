use super::*;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn complete_claim_requires_both_total_and_file_coverage() {
    let run = TrustedCoverageRun {
        report: json!({"data": [{
            "totals": {"lines": {"percent": 100.0}},
            "files": [{
                "filename": "/tmp/repo/src/main.rs",
                "summary": {"lines": {"percent": 99.0}}
            }]
        }]}),
        command: "trusted".into(),
        generated_at: "2026-01-01T00:00:00Z".into(),
        provenance: json!({"bound": true}),
    };
    let receipt = build_receipt(Path::new("/tmp/repo"), &run).unwrap();
    assert_eq!(receipt["coverage"]["policy"], "blocked_or_withheld");
    assert_eq!(receipt["provenance"]["bound"], true);
}

#[test]
fn custom_receipt_refuses_existing_file_before_running_coverage() {
    let root = std::env::temp_dir().join(format!(
        "nrr-coverage-existing-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&root).unwrap();
    let output = root.join("selected.json");
    fs::write(&output, "ORIGINAL").unwrap();
    let args = crate::config::parse_args_from([
        "nrr",
        "coverage-receipt",
        "--skill-dir",
        env!("CARGO_MANIFEST_DIR"),
        "--coverage-receipt",
        output.to_str().unwrap(),
        "--i-consent-to-custom-runtime-paths",
    ])
    .unwrap();
    let error = write_coverage_receipt(&args).unwrap_err().to_string();
    assert!(error.contains("refuses existing path"));
    assert_eq!(fs::read_to_string(output).unwrap(), "ORIGINAL");
}
