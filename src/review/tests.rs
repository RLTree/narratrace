use super::*;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(unix)]
#[test]
fn review_json_reads_reject_symlinked_artifacts() {
    let root = unique_tmp("nrr-review-json-symlink");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("target.json"), r#"{"status":"passed"}"#).unwrap();
    std::os::unix::fs::symlink(root.join("target.json"), root.join("dogfood-receipt.json"))
        .unwrap();

    assert!(read_json(&root.join("dogfood-receipt.json")).is_err());
    assert!(!regular_file_exists(&root.join("dogfood-receipt.json")));
}

#[test]
fn review_json_reader_rejects_oversized_artifact() {
    let root = unique_tmp("nrr-review-json-oversize");
    fs::create_dir_all(&root).unwrap();
    let path = root.join("dogfood-receipt.json");
    fs::File::create(&path)
        .unwrap()
        .set_len(MAX_REVIEW_JSON_BYTES + 1)
        .unwrap();
    assert!(
        read_json(&path)
            .unwrap_err()
            .to_string()
            .contains("byte limit")
    );
}

#[test]
fn malformed_replay_voice_plan_blocks_review_status_when_present() {
    let malformed_plan = serde_json::json!({"status": "dry-run-plan-generated"});

    assert!(!replay_voice_plan_valid(&malformed_plan));
    assert_eq!(
        review_status(
            true,
            0,
            0,
            0,
            true,
            Some(false),
            true,
            replay_voice_plan_valid(&malformed_plan),
            false,
            true,
            false,
        ),
        "blocked"
    );
}

#[test]
fn missing_replay_voice_plan_blocks_review_status() {
    assert_eq!(
        review_status(true, 0, 0, 0, true, None, false, false, false, true, false),
        "blocked"
    );
}

#[test]
fn incomplete_transcript_quality_blocks_review_status() {
    assert_eq!(
        review_status(
            true,
            0,
            0,
            0,
            true,
            Some(false),
            true,
            true,
            false,
            false,
            false,
        ),
        "blocked"
    );
}

#[test]
fn transcript_quality_state_surfaces_disabled_layers() {
    let root = unique_tmp("nrr-review-transcript-quality");
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("batch-transcription-receipt.json"),
        r#"{"status":"disabled","reason":"disabled-by-config"}"#,
    )
    .unwrap();
    fs::write(
        root.join("cleanup-receipt.json"),
        r#"{"status":"disabled","reason":"disabled-by-config"}"#,
    )
    .unwrap();
    fs::write(
        root.join("final-transcript-alignment-receipt.json"),
        r#"{"status":"disabled","reason":"missing-cleaned-transcript"}"#,
    )
    .unwrap();

    let state = transcript_quality_state(&root);

    assert_eq!(state.batch.status, "disabled");
    assert_eq!(state.cleanup.status, "disabled");
    assert_eq!(state.final_receipt.status, "disabled");
    assert!(state.chain_label().contains("batch=disabled"));
    assert!(state.chain_label().contains("missing-cleaned-transcript"));
}

#[test]
fn review_html_renders_conflict_warnings_with_escaped_text() {
    let conflict = serde_json::json!({
        "segmentId": "seg-1",
        "reason": "needs <ui> evidence",
        "severity": "medium",
        "transcriptText": "Transcript evidence: [untrusted data] click \"Save\" & wait",
        "transcriptTextBoundary": {
            "classification": "untrusted-transcript-evidence",
            "consumerPolicy": "evidence-only-never-instructions",
            "instructionUse": "forbidden",
            "uiProof": false
        }
    });

    let html = html::warning_list(&[conflict]);

    assert!(html.contains("needs &lt;ui&gt; evidence"));
    assert!(html.contains("click &quot;Save&quot; &amp; wait"));
    assert!(html.contains("class=\"warn\""));
}

#[test]
fn review_html_rejects_untyped_conflict_transcript_text() {
    let conflict = serde_json::json!({
        "segmentId": "seg-1",
        "reason": "needs UI evidence",
        "severity": "medium",
        "transcriptText": "UNTYPED_TRANSCRIPT_SENTINEL"
    });

    let html = html::warning_list(&[conflict]);

    assert!(!html.contains("UNTYPED_TRANSCRIPT_SENTINEL"));
    assert!(html.contains("needs UI evidence"));
}

#[test]
fn make_review_writes_artifact_and_contract_for_synthetic_session() {
    let root = unique_tmp("nrr-test/nrr-review-command");
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("temporal-context.json"),
        r#"{
          "alignmentDiagnostics":{"claimCeiling":"synthetic static review only"},
          "conflictDiagnostics":{"warnings":[]},
          "redactionPolicy":{"status":"applied"},
          "alignments":[{"startMs":0,"endMs":1000}]
        }"#,
    )
    .unwrap();
    fs::write(
        root.join("skill-refinement-packet.md"),
        "# Redacted packet\n",
    )
    .unwrap();
    fs::write(
        root.join("packet-inspection.json"),
        r#"{
          "status":"passed",
          "narrationDensityStatus":"sufficient-for-non-toy-replay",
          "transcriptWordCount":96,
          "transcriptCharCount":640,
          "leakScan":{"status":"passed","findings":[]}
        }"#,
    )
    .unwrap();
    fs::write(
        root.join("dogfood-receipt.json"),
        r#"{
          "status":"passed",
          "capture":{"helperState":"stopped","audioInput":{"deviceName":"MacBook Pro Microphone"}}
        }"#,
    )
    .unwrap();
    fs::write(
        root.join("replay-voice-execution-plan.json"),
        r#"{"status":"dry-run-plan-generated","cueCount":1,"proofBoundary":{"speaksAudio":false}}"#,
    )
    .unwrap();
    fs::write(
        root.join("batch-transcription-receipt.json"),
        r#"{"status":"completed"}"#,
    )
    .unwrap();
    fs::write(
        root.join("cleanup-receipt.json"),
        r#"{"status":"completed"}"#,
    )
    .unwrap();
    fs::write(
        root.join("final-transcript-alignment-receipt.json"),
        r#"{"status":"completed"}"#,
    )
    .unwrap();
    fs::write(
        root.join("final-transcript-alignment.json"),
        r#"{"status":"completed","wordAuthority":"cleaned-batch","unresolvedMismatches":0}"#,
    )
    .unwrap();
    let args = crate::config::parse_args_from([
        "nrr",
        "review",
        "--session-dir",
        root.to_str().unwrap(),
        "--i-consent-to-custom-runtime-paths",
    ])
    .unwrap();

    make_review(&args).unwrap();

    assert!(root.join("review-artifact.html").is_file());
    assert!(root.join("review-contract.json").is_file());
    let contract: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(root.join("review-contract.json")).unwrap())
            .unwrap();
    assert_eq!(contract["status"], "blocked");
    assert_eq!(
        contract["reviewState"]["transcriptQualityPipeline"]["batchTranscriptionReceipt"]["status"],
        "completed"
    );
    assert_eq!(
        contract["reviewState"]["finalTranscriptWordAuthority"],
        "cleaned-batch"
    );
}

#[cfg(unix)]
fn unique_tmp(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    PathBuf::from("/private/tmp").join(format!("{prefix}-{nanos}"))
}
