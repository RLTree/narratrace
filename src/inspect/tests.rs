use super::*;
use crate::config::parse_args_from;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(unix)]
#[test]
fn inspect_json_reads_reject_symlinked_artifacts() {
    let root = unique_tmp("nrr-inspect-json-symlink");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("target.json"), r#"{"requiredReview":["raw"]}"#).unwrap();
    std::os::unix::fs::symlink(root.join("target.json"), root.join("evidence.json")).unwrap();

    assert!(read_json(&root.join("evidence.json")).is_err());
    assert!(!regular_file_exists(&root.join("evidence.json")));
}

#[test]
fn inspect_packet_writes_review_artifacts_from_synthetic_session() {
    let root = unique_tmp("nrr-inspect-packet");
    let session_dir = root.join("session");
    fs::create_dir_all(&session_dir).unwrap();
    fs::write(
        session_dir.join("temporal-context.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "alignments": [{"id": "a1"}],
            "alignmentDiagnostics": {
                "claimCeiling": "synthetic fixture only",
                "outOfWindowRecordReplayEvents": 0
            },
            "conflictDiagnostics": {"warnings": []},
            "redactionPolicy": {"status": "fixture-redacted"}
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        session_dir.join("evidence-boundary-report.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "requiredReview": [],
            "unsupportedClaims": [],
            "evidenceSurfaces": {
                "transcriptSegments": 1,
                "recordReplayEvents": 1,
                "alignedSegments": 1,
                "redactionStatus": "fixture-redacted"
            }
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        session_dir.join("skill-refinement-packet.md"),
        "# Fixture Packet\n\nSufficient narrated workflow detail for review.",
    )
    .unwrap();
    fs::write(
        session_dir.join("batch-transcription-receipt.json"),
        r#"{"status":"completed"}"#,
    )
    .unwrap();
    fs::write(
        session_dir.join("cleanup-receipt.json"),
        r#"{"status":"completed"}"#,
    )
    .unwrap();
    fs::write(
        session_dir.join("final-transcript-alignment-receipt.json"),
        r#"{"status":"completed"}"#,
    )
    .unwrap();
    let args = parse_args_from([
        "nrr",
        "inspect",
        "--session-dir",
        session_dir.to_str().unwrap(),
        "--i-consent-to-custom-runtime-paths",
    ])
    .unwrap();

    inspect_packet(&args).unwrap();

    let inspection = fs::read_to_string(session_dir.join("packet-inspection.json")).unwrap();
    assert!(inspection.contains("narrated-record-replay.packet-inspection.v1"));
    assert!(inspection.contains("generatedArtifactLeakScan"));
    assert!(session_dir.join("review-contract.json").is_file());
    assert!(session_dir.join("review-artifact.html").is_file());
}

#[test]
fn inspect_helpers_surface_missing_and_unusable_artifacts() {
    let root = unique_tmp("nrr-inspect-helper-blockers");
    fs::create_dir_all(&root).unwrap();
    let packet = root.join("packet.md");
    let temporal = root.join("temporal.json");
    let evidence_path = root.join("evidence.json");
    let review = root.join("review.json");
    fs::write(&packet, "packet").unwrap();
    fs::write(&temporal, "{}").unwrap();

    let evidence = serde_json::json!({
        "evidenceSurfaces": {
            "recordReplayArtifacts": {
                "metadata": {"provided": true, "usableForLiveProof": false},
                "events": {"provided": false}
            }
        }
    });
    let blocker_list = blockers(&temporal, &evidence_path, &review, &packet, &evidence, 1, 1);

    assert!(blocking_leak_count(&serde_json::json!({"findings":[{"blocksShare": true}]})) == 1);
    assert!(provided_external_artifact_unusable(
        &evidence,
        "/evidenceSurfaces/recordReplayArtifacts/metadata"
    ));
    assert_eq!(path_exists(&packet)["exists"], true);
    assert_eq!(existing(&packet), Some(packet.as_path()));
    assert!(blocker_list.contains(&"evidence-boundary-report.json is missing"));
    assert!(blocker_list.contains(&"conflict warnings require operator review"));
    assert!(
        blocker_list
            .iter()
            .any(|item| item.contains("metadata artifact"))
    );
}

#[test]
fn approved_record_replay_paths_require_live_proof_flags_and_regular_files() {
    let root = unique_tmp("nrr-inspect-approved-rnr");
    fs::create_dir_all(&root).unwrap();
    let approved = root.join("events.jsonl");
    let unapproved = root.join("metadata.json");
    fs::write(&approved, "{}\n").unwrap();
    fs::write(&unapproved, "{}").unwrap();
    let evidence = serde_json::json!({"evidenceSurfaces":{"recordReplayArtifacts":{
        "events":{"path":approved.display().to_string(),"provided":true,"safeRegularFile":true,"usableForLiveProof":true},
        "metadata":{"path":unapproved.display().to_string(),"provided":true,"safeRegularFile":true,"usableForLiveProof":false}
    }}});
    assert_eq!(approved_record_replay_paths(&evidence), vec![approved]);
}

#[cfg(unix)]
fn unique_tmp(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    PathBuf::from("/private/tmp").join(format!("{prefix}-{nanos}"))
}
