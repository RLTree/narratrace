use super::*;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn invalid_utf8_fails_closed_for_generated_and_raw_scans() {
    let root = unique_tmp("nrr-inspect-invalid-utf8");
    fs::create_dir_all(&root).unwrap();
    let path = root.join("review-artifact.html");
    fs::write(&path, [b'A', b'K', b'I', b'A', 0xff]).unwrap();
    let generated = artifact_spec("review-artifact", &path, "generated-review-candidate");
    let scan = leak_scan(&[generated], &[]);
    assert_eq!(scan["status"], "blocked");
    assert_eq!(scan["findings"][0]["disposition"], "artifact-read-or-decode-failed");

    let raw = artifact_spec("transcript-live", &path, "raw-local-private");
    let entry = raw_local_entry(&raw);
    assert_eq!(entry["containsSensitivePatterns"], true);
    assert_eq!(entry["sensitiveCategories"][0], "artifact-read-or-decode-failed");
}

#[test]
fn leak_scan_blocks_per_file_budget_exhaustion() {
    let root = unique_tmp("nrr-inspect-oversize");
    fs::create_dir_all(&root).unwrap();
    let path = root.join("review-artifact.html");
    fs::File::create(&path)
        .unwrap()
        .set_len(MAX_SCAN_BYTES_PER_FILE + 1)
        .unwrap();
    let artifact = artifact_spec("review-artifact", &path, "generated-review-candidate");
    let scan = leak_scan(&[artifact], &[]);
    assert_eq!(scan["status"], "blocked");
    assert_eq!(scan["findings"][0]["disposition"], "artifact-inspection-budget-exceeded");
}

#[test]
fn leak_scan_enforces_aggregate_budget() {
    let root = unique_tmp("nrr-inspect-aggregate");
    fs::create_dir_all(&root).unwrap();
    let mut paths = Vec::new();
    for index in 0..5 {
        let path = root.join(format!("review-{index}.txt"));
        fs::write(&path, vec![b' '; 7 * 1024 * 1024]).unwrap();
        paths.push(path);
    }
    let artifacts = paths
        .iter()
        .map(|path| artifact_spec("review-artifact", path, "generated-review-candidate"))
        .collect::<Vec<_>>();
    let scan = leak_scan(&artifacts, &[]);
    assert_eq!(scan["status"], "blocked");
    assert!(scan["findings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|finding| finding["disposition"] == "artifact-inspection-budget-exceeded"));
}

#[test]
fn only_exact_approved_external_private_path_is_nonblocking() {
    let session = unique_tmp("nrr-inspect-approved-session");
    let external = unique_tmp("nrr-inspect-approved-external");
    fs::create_dir_all(&session).unwrap();
    fs::create_dir_all(&external).unwrap();
    let approved = external.join("events.jsonl");
    let sibling = external.join("private-project.txt");
    fs::write(&approved, "{}\n").unwrap();
    fs::write(&sibling, "private").unwrap();
    let packet = session.join("review-artifact.html");
    fs::write(&packet, format!("event path {}", approved.display())).unwrap();
    let artifact = artifact_spec("review-artifact", &packet, "generated-review-candidate");
    assert_eq!(leak_scan(&[artifact], std::slice::from_ref(&approved))["status"],
        "expected-local-references-only");

    fs::write(&packet, format!("unrelated path {}", sibling.display())).unwrap();
    let artifact = artifact_spec("review-artifact", &packet, "generated-review-candidate");
    assert_eq!(leak_scan(&[artifact], std::slice::from_ref(&approved))["status"], "blocked");
}

fn unique_tmp(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    PathBuf::from(format!("/private/tmp/{prefix}-{nanos}"))
}
