use super::transcript::transcript_segments_checked;
use crate::config::parse_args_from;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn generated_v2_alignment_receipt_is_accepted_by_timeline() {
    let root = unique_tmp("nrr-alignment-authority-integration");
    fs::create_dir_all(&root).unwrap();
    let args = parse_args_from([
        "nrr",
        "packet",
        "--session-dir",
        root.to_str().unwrap(),
        "--i-consent-to-custom-runtime-paths",
    ])
    .unwrap();
    crate::transcript_cleanup::write_bound_cleanup_fixture_for_test(
        &args,
        &root,
        "cloud code opened chat g p t atlas",
        "Claude Code opened ChatGPT Atlas.",
    )
    .unwrap();
    fs::write(
        root.join("transcript.timeline.jsonl"),
        r#"{"kind":"completed","monotonicOffsetMs":1000,"text":"cloud code opened chat g p t atlas"}"#,
    )
    .unwrap();

    crate::transcript_alignment::ensure_final_alignment(&root)
        .unwrap()
        .unwrap();
    let segments = transcript_segments_checked(&root).unwrap();

    assert_eq!(segments[0].text, "Claude Code opened ChatGPT Atlas.");
    assert_eq!(
        segments[0].timing_source,
        "aligned-cleaned-batch-text-with-realtime-window"
    );
    fs::remove_dir_all(root).unwrap();
}

fn unique_tmp(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    PathBuf::from("/private/tmp").join(format!("{prefix}-{nanos}"))
}
