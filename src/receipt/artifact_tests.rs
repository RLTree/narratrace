use super::artifacts::{MAX_RECEIPT_ARTIFACT_BYTES, artifact, artifact_entries, optional_artifact};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn artifact_entries_report_regular_file_stats_and_optional_absence() {
    let root = unique_tmp("nrr-receipt-artifacts");
    fs::create_dir_all(&root).unwrap();
    let path = root.join("notes.md");
    fs::write(&path, "one\ntwo").unwrap();

    let entries = artifact_entries(
        &[
            artifact("notes", path),
            optional_artifact("recording-metadata", None),
        ],
        "generated-review-candidate",
    )
    .unwrap();

    assert_eq!(entries[0]["name"], "notes");
    assert_eq!(entries[0]["exists"], Value::Bool(true));
    assert_eq!(entries[0]["isRegularFile"], Value::Bool(true));
    assert_eq!(entries[0]["bytes"], Value::Number(7.into()));
    assert_eq!(entries[0]["lineCount"], Value::Number(2.into()));
    assert!(
        entries[0]["contentFingerprint"]
            .as_str()
            .unwrap()
            .starts_with("sha256:")
    );
    assert!(entries[1]["path"].is_null());
    assert_eq!(entries[1]["exists"], Value::Bool(false));
}

#[cfg(unix)]
#[test]
fn artifact_entries_do_not_hash_symlink_targets() {
    let root = unique_tmp("nrr-receipt-artifact-symlink");
    fs::create_dir_all(&root).unwrap();
    let target = root.join("target.txt");
    fs::write(&target, "secret").unwrap();
    let link = root.join("link.txt");
    std::os::unix::fs::symlink(&target, &link).unwrap();

    let entries = artifact_entries(&[artifact("link", link)], "raw-local-private").unwrap();

    assert_eq!(entries[0]["exists"], Value::Bool(false));
    assert_eq!(entries[0]["isRegularFile"], Value::Bool(false));
    assert!(entries[0]["contentFingerprint"].is_null());
}

#[test]
fn artifact_entries_count_empty_file_lines_as_zero() {
    let root = unique_tmp("nrr-receipt-artifact-empty");
    fs::create_dir_all(&root).unwrap();
    let path = root.join("empty.txt");
    fs::write(&path, "").unwrap();

    let entries = artifact_entries(&[artifact("empty", path)], "raw-local-private").unwrap();

    assert_eq!(entries[0]["bytes"], Value::Number(0.into()));
    assert_eq!(entries[0]["lineCount"], Value::Number(0.into()));
}

#[test]
fn artifact_entries_fail_closed_on_malformed_jsonl() {
    let root = unique_tmp("nrr-receipt-artifact-malformed-jsonl");
    fs::create_dir_all(&root).unwrap();
    let path = root.join("events.jsonl");
    fs::write(&path, "{\"ok\":true}\n{bad}\n").unwrap();

    let error = artifact_entries(&[artifact("events", path)], "raw-local-private")
        .unwrap_err()
        .to_string();

    assert!(error.contains("malformed receipt JSONL row 2"));
}

#[test]
fn artifact_entries_reject_aggregate_budget_before_hashing() {
    let root = unique_tmp("nrr-receipt-artifact-budget");
    fs::create_dir_all(&root).unwrap();
    let path = root.join("large.txt");
    fs::File::create(&path)
        .unwrap()
        .set_len(MAX_RECEIPT_ARTIFACT_BYTES + 1)
        .unwrap();
    assert!(artifact_entries(&[artifact("large", path)], "raw-local-private").is_err());
}

fn unique_tmp(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    PathBuf::from("/private/tmp").join(format!("{prefix}-{nanos}"))
}
