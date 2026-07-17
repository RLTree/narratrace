#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[cfg(unix)]
    #[test]
    fn leak_scan_blocks_symlinked_generated_artifact_without_reading_target() {
        let root = unique_tmp("nrr-inspect-symlink");
        fs::create_dir_all(&root).unwrap();
        let target = root.join("outside.txt");
        fs::write(&target, "private path /Users/tree/secret.txt").unwrap();
        let link = root.join("review-artifact.html");
        std::os::unix::fs::symlink(&target, &link).unwrap();
        let artifact = artifact_spec("review-artifact", &link, "generated-review-candidate");

        let scan = leak_scan(&[artifact], &[]);

        assert_eq!(scan["status"], "blocked");
        assert_eq!(
            scan["findings"][0]["categories"][0],
            Value::String("unsafe-artifact-path".to_string())
        );
        assert_eq!(scan["findings"][0]["blocksShare"], Value::Bool(true));
    }

    #[cfg(unix)]
    #[test]
    fn raw_local_entry_marks_symlink_unsafe_without_hashing_target() {
        let root = unique_tmp("nrr-inspect-raw-symlink");
        fs::create_dir_all(&root).unwrap();
        let target = root.join("outside.txt");
        fs::write(&target, "secret").unwrap();
        let link = root.join("transcript.timeline.jsonl");
        std::os::unix::fs::symlink(&target, &link).unwrap();
        let artifact = artifact_spec("transcript-timeline", &link, "raw-local-private");

        let entry = raw_local_entry(&artifact);

        assert_eq!(entry["safeRegularFile"], Value::Bool(false));
        assert!(entry["contentFingerprint"].is_null());
        assert_eq!(
            entry["sensitiveCategories"][0],
            Value::String("unsafe-artifact-path".to_string())
        );
    }

    #[test]
    fn raw_local_entry_does_not_treat_missing_optional_file_as_sensitive() {
        let root = unique_tmp("nrr-inspect-missing-raw");
        fs::create_dir_all(&root).unwrap();
        let missing = root.join("transcript.final.txt");
        let artifact = artifact_spec("transcript-final", &missing, "raw-local-private");

        let entry = raw_local_entry(&artifact);

        assert_eq!(entry["exists"], Value::Bool(false));
        assert_eq!(entry["safeRegularFile"], Value::Bool(false));
        assert_eq!(entry["containsSensitivePatterns"], Value::Bool(false));
        assert!(entry["sensitiveCategories"].as_array().unwrap().is_empty());
    }

    #[test]
    fn leak_scan_allows_expected_local_artifact_references() {
        let root = unique_tmp("nrr-inspect-local-reference");
        fs::create_dir_all(&root).unwrap();
        let packet = root.join("skill-refinement-packet.md");
        fs::write(
            &packet,
            format!(
                "packet path {} and event path /var/folders/example/sky/event_stream/session/events.jsonl",
                packet.display()
            ),
        )
        .unwrap();
        let artifact = artifact_spec(
            "skill-refinement-packet",
            &packet,
            "generated-review-candidate",
        );

        let scan = leak_scan(&[artifact], &[PathBuf::from("/var/folders/example/sky/event_stream/session/events.jsonl")]);

        assert_eq!(scan["status"], "expected-local-references-only");
        assert_eq!(scan["findings"][0]["blocksShare"], Value::Bool(false));
        assert_eq!(
            scan["findings"][0]["disposition"],
            Value::String("expected-local-artifact-reference".to_string())
        );
    }

    #[test]
    fn leak_scan_blocks_unexpected_private_path() {
        let root = unique_tmp("nrr-inspect-unexpected-private-path");
        fs::create_dir_all(&root).unwrap();
        let packet = root.join("skill-refinement-packet.md");
        fs::write(&packet, "unexpected /Users/tree/private-note.txt").unwrap();
        let artifact = artifact_spec(
            "skill-refinement-packet",
            &packet,
            "generated-review-candidate",
        );

        let scan = leak_scan(&[artifact], &[]);

        assert_eq!(scan["status"], "blocked");
        assert_eq!(scan["findings"][0]["blocksShare"], Value::Bool(true));
    }

    #[test]
    fn leak_scan_does_not_block_when_optional_generated_artifact_is_omitted() {
        let root = unique_tmp("nrr-inspect-optional-generated");
        fs::create_dir_all(&root).unwrap();
        let packet = root.join("skill-refinement-packet.md");
        fs::write(&packet, "review text").unwrap();
        let artifact = artifact_spec(
            "skill-refinement-packet",
            &packet,
            "generated-review-candidate",
        );

        let scan = leak_scan(&[artifact], &[]);

        assert_eq!(scan["status"], "no-obvious-sensitive-patterns-detected");
        assert!(scan["findings"].as_array().unwrap().is_empty());
    }

    #[test]
    fn artifact_entries_report_policy_path_and_presence() {
        let root = unique_tmp("nrr-inspect-artifact-entry");
        fs::create_dir_all(&root).unwrap();
        let packet = root.join("skill-refinement-packet.md");
        fs::write(&packet, "review text").unwrap();
        let artifact = artifact_spec("skill-refinement-packet", &packet, "generated-review");

        let entries = artifact_entries(&[artifact]);

        assert_eq!(entries[0]["name"], "skill-refinement-packet");
        assert_eq!(entries[0]["exists"], Value::Bool(true));
        assert_eq!(entries[0]["policy"], "generated-review");
        assert!(
            entries[0]["path"]
                .as_str()
                .unwrap()
                .ends_with("skill-refinement-packet.md")
        );
    }

    #[test]
    fn raw_local_entries_include_line_count_bytes_and_fingerprint() {
        let root = unique_tmp("nrr-inspect-raw-stats");
        fs::create_dir_all(&root).unwrap();
        let transcript = root.join("transcript.timeline.jsonl");
        fs::write(&transcript, "one\ntwo").unwrap();
        let artifact = artifact_spec("transcript-timeline", &transcript, "raw-local-private");

        let entries = raw_local_entries(&[artifact]);

        assert_eq!(entries[0]["safeRegularFile"], Value::Bool(true));
        assert_eq!(entries[0]["bytes"], Value::Number(7.into()));
        assert_eq!(entries[0]["lineCount"], Value::Number(2.into()));
        assert!(
            entries[0]["contentFingerprint"]
                .as_str()
                .unwrap()
                .starts_with("sha256:")
        );
    }

    #[test]
    fn leak_scan_allows_opaque_token_without_blocking_share() {
        let root = unique_tmp("nrr-inspect-opaque-token");
        fs::create_dir_all(&root).unwrap();
        let packet = root.join("review-artifact.html");
        fs::write(&packet, "Ab1Cd2Ef3Gh4Ij5Kl6Mn7Op8Qr9St0UvWxYz").unwrap();
        let artifact = artifact_spec("review-artifact", &packet, "generated-review-candidate");

        let scan = leak_scan(&[artifact], &[]);

        assert_eq!(scan["status"], "expected-local-references-only");
        assert_eq!(scan["findings"][0]["blocksShare"], Value::Bool(false));
        assert_eq!(
            scan["findings"][0]["disposition"],
            Value::String("expected-local-opaque-token-reference".to_string())
        );
    }

    #[test]
    fn leak_scan_blocks_canonical_short_credential() {
        let root = unique_tmp("nrr-inspect-short-credential");
        fs::create_dir_all(&root).unwrap();
        let packet = root.join("review-artifact.html");
        fs::write(&packet, "AKIAIOSFODNN7EXAMPLE").unwrap();
        let artifact = artifact_spec("review-artifact", &packet, "generated-review-candidate");

        let scan = leak_scan(&[artifact], &[]);

        assert_eq!(scan["status"], "blocked");
        assert!(scan["findings"][0]["categories"]
            .as_array()
            .unwrap()
            .iter()
            .any(|category| category == "secret-token"));
        assert_eq!(scan["findings"][0]["blocksShare"], Value::Bool(true));
    }

    #[test]
    fn leak_scan_allows_private_tmp_and_var_folder_references_only() {
        let root = unique_tmp("nrr-inspect-private-tmp-reference");
        fs::create_dir_all(&root).unwrap();
        let packet = root.join("review-artifact.html");
        fs::write(
            &packet,
            "paths (/private/tmp/nrr-audio.wav), [/private/var/folders/a/b/c], ~/outside",
        )
        .unwrap();
        let artifact = artifact_spec("review-artifact", &packet, "generated-review-candidate");

        let scan = leak_scan(&[artifact], &[]);

        assert_eq!(scan["status"], "blocked");
        assert_eq!(scan["findings"][0]["blocksShare"], Value::Bool(true));
    }

    #[test]
    fn raw_local_entry_counts_empty_file_lines_as_zero() {
        let root = unique_tmp("nrr-inspect-empty-raw");
        fs::create_dir_all(&root).unwrap();
        let transcript = root.join("transcript.timeline.jsonl");
        fs::write(&transcript, "").unwrap();
        let artifact = artifact_spec("transcript-timeline", &transcript, "raw-local-private");

        let entry = raw_local_entry(&artifact);

        assert_eq!(entry["bytes"], Value::Number(0.into()));
        assert_eq!(entry["lineCount"], Value::Number(0.into()));
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
