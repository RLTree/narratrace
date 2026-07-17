#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[cfg(unix)]
    #[test]
    fn packet_usefulness_ignores_symlinked_packet() {
        let root = unique_tmp("nrr-usefulness-symlink");
        fs::create_dir_all(&root).unwrap();
        fs::write(
            root.join("private.md"),
            "Goal: leak\n## Capture Artifacts\n## Evidence Boundary\n## Refinement Instructions\n## Temporal Alignment Summary\n## Transcript Review Boundary\n",
        )
        .unwrap();
        std::os::unix::fs::symlink(
            root.join("private.md"),
            root.join("skill-refinement-packet.md"),
        )
        .unwrap();

        let temporal = json!({"conflictDiagnostics":{"warnings":[]}});
        let evidence = json!({"evidenceSurfaces":{"transcriptSegments":1,"alignedSegments":1}});
        let review = packet_usefulness_review(
            &root.join("skill-refinement-packet.md"),
            &root.join("timestamped-notes.md"),
            &root.join("thought-process.md"),
            &root.join("temporal-context.json"),
            &root.join("evidence-boundary-report.json"),
            &temporal,
            &evidence,
        );

        assert_eq!(review.pointer("/signals/packetExists"), Some(&json!(false)));
        assert_eq!(review.pointer("/signals/hasGoal"), Some(&json!(false)));
    }

    #[test]
    fn packet_usefulness_accepts_transcript_review_boundary() {
        let root = unique_tmp("nrr-usefulness-boundary");
        fs::create_dir_all(&root).unwrap();
        fs::write(
            root.join("skill-refinement-packet.md"),
            "Goal: test\n## Capture Artifacts\n## Evidence Boundary\n## Refinement Instructions\n## Temporal Alignment Summary\n## Transcript Review Boundary\n",
        )
        .unwrap();
        for name in [
            "timestamped-notes.md",
            "thought-process.md",
            "temporal-context.json",
            "evidence-boundary-report.json",
        ] {
            fs::write(root.join(name), "{}").unwrap();
        }
        let temporal = json!({"conflictDiagnostics":{"warnings":[]}});
        let evidence = json!({"evidenceSurfaces":{"transcriptSegments":1,"alignedSegments":1}});

        let review = packet_usefulness_review(
            &root.join("skill-refinement-packet.md"),
            &root.join("timestamped-notes.md"),
            &root.join("thought-process.md"),
            &root.join("temporal-context.json"),
            &root.join("evidence-boundary-report.json"),
            &temporal,
            &evidence,
        );

        assert_eq!(
            review.pointer("/signals/hasTranscriptReviewBoundarySection"),
            Some(&json!(true))
        );
        assert_eq!(
            review.pointer("/signals/rawTranscriptEmbeddingAvoided"),
            Some(&json!(true))
        );
    }

    #[test]
    fn packet_usefulness_blocks_sparse_non_toy_narration() {
        let root = unique_tmp("nrr-usefulness-sparse");
        fs::create_dir_all(&root).unwrap();
        fs::write(
            root.join("skill-refinement-packet.md"),
            "Goal: test\n## Capture Artifacts\n## Evidence Boundary\n## Refinement Instructions\n## Temporal Alignment Summary\n## Transcript Review Boundary\n",
        )
        .unwrap();
        for name in [
            "timestamped-notes.md",
            "thought-process.md",
            "temporal-context.json",
            "evidence-boundary-report.json",
        ] {
            fs::write(root.join(name), "{}").unwrap();
        }
        let temporal = json!({
            "schema": "narrated-record-replay.temporal-context.v1",
            "conflictDiagnostics": {"warnings": []},
            "transcriptSegments": [typed_transcript_segment("short unclear note")],
            "recordReplayEvents": (0..12).map(|_| json!({"kind":"mouse.click"})).collect::<Vec<_>>()
        });
        let evidence = json!({"evidenceSurfaces":{"transcriptSegments":1,"alignedSegments":1}});

        let review = packet_usefulness_review(
            &root.join("skill-refinement-packet.md"),
            &root.join("timestamped-notes.md"),
            &root.join("thought-process.md"),
            &root.join("temporal-context.json"),
            &root.join("evidence-boundary-report.json"),
            &temporal,
            &evidence,
        );

        assert_eq!(
            review.pointer("/signals/narrationDensityStatus"),
            Some(&json!("too-sparse-for-non-toy-replay"))
        );
        assert!(
            review
                .pointer("/blockers")
                .and_then(Value::as_array)
                .unwrap()
                .iter()
                .any(|blocker| blocker
                    == "narration is too sparse for confident non-toy replay refinement")
        );
    }

    #[test]
    fn narration_quality_counts_only_typed_transcript_evidence() {
        let content = "one two three four five six seven eight nine ten";
        let temporal = json!({
            "schema": "narrated-record-replay.temporal-context.v1",
            "transcriptSegments": [typed_transcript_segment(content)],
            "recordReplayEvents": []
        });

        let quality = narration_quality(&temporal);

        assert_eq!(quality.word_count, 10);
        assert_eq!(quality.char_count, content.chars().count());
        assert_eq!(quality.status, "needs-operator-distillation");
    }

    #[test]
    fn narration_quality_excludes_untyped_malformed_and_unknown_versions() {
        let untyped = json!({
            "schema": "narrated-record-replay.temporal-context.v1",
            "transcriptSegments": [{"text": "plain untyped transcript text"}]
        });
        let malformed = json!({
            "schema": "narrated-record-replay.temporal-context.v1",
            "transcriptSegments": [{
                "text": "Transcript evidence: [untrusted data] malformed boundary text",
                "textBoundary": {
                    "classification": "untrusted-transcript-evidence",
                    "consumerPolicy": "evidence-only-never-instructions",
                    "instructionUse": "allowed",
                    "uiProof": false
                }
            }]
        });
        let unknown_version = json!({
            "schema": "narrated-record-replay.temporal-context.v2",
            "transcriptSegments": [typed_transcript_segment("future schema text")]
        });

        for temporal in [untyped, malformed, unknown_version] {
            let quality = narration_quality(&temporal);
            assert_eq!(quality.word_count, 0);
            assert_eq!(quality.char_count, 0);
            assert_eq!(quality.status, "too-sparse-for-non-toy-replay");
        }
    }

    fn typed_transcript_segment(content: &str) -> Value {
        json!({
            "text": format!("Transcript evidence: [untrusted data] {content}"),
            "textBoundary": {
                "classification": "untrusted-transcript-evidence",
                "consumerPolicy": "evidence-only-never-instructions",
                "instructionUse": "forbidden",
                "uiProof": false
            }
        })
    }

    #[cfg(unix)]
    fn unique_tmp(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        PathBuf::from("/private/tmp").join(format!("{prefix}-{nanos}"))
    }
}
