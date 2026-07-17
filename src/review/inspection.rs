use crate::safe_path::read_regular_text_bounded;
use serde_json::Value;
#[cfg(test)]
use std::fs;
use std::path::Path;

const MAX_PACKET_INSPECTION_BYTES: u64 = 8 * 1024 * 1024;

pub(super) fn read_packet_inspection(path: &Path) -> Value {
    read_regular_text_bounded(path, MAX_PACKET_INSPECTION_BYTES)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or(Value::Null)
}

pub(super) fn inspection_status(packet_inspection: &Value) -> &str {
    packet_inspection
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("not-generated")
}

pub(super) fn leak_scan_status(packet_inspection: &Value) -> &str {
    packet_inspection
        .pointer("/privacyBoundary/generatedArtifactLeakScan/status")
        .and_then(Value::as_str)
        .unwrap_or("not-generated")
}

pub(super) fn leak_finding_count(packet_inspection: &Value) -> usize {
    packet_inspection
        .pointer("/privacyBoundary/generatedArtifactLeakScan/findings")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0)
}

pub(super) fn blocking_leak_finding_count(packet_inspection: &Value) -> usize {
    packet_inspection
        .pointer("/privacyBoundary/generatedArtifactLeakScan/findings")
        .and_then(Value::as_array)
        .map(|findings| {
            findings
                .iter()
                .filter(|finding| {
                    finding
                        .get("blocksShare")
                        .and_then(Value::as_bool)
                        .unwrap_or(true)
                })
                .count()
        })
        .unwrap_or(0)
}

pub(super) fn leak_categories(packet_inspection: &Value) -> Vec<String> {
    let mut categories = Vec::new();
    let Some(findings) = packet_inspection
        .pointer("/privacyBoundary/generatedArtifactLeakScan/findings")
        .and_then(Value::as_array)
    else {
        return categories;
    };
    for finding in findings {
        let Some(items) = finding.get("categories").and_then(Value::as_array) else {
            continue;
        };
        for item in items {
            let Some(category) = item.as_str() else {
                continue;
            };
            if !categories.iter().any(|existing| existing == category) {
                categories.push(category.to_string());
            }
        }
    }
    categories
}

pub(super) fn raw_local_sensitive_artifact_count(packet_inspection: &Value) -> usize {
    let Some(artifacts) = packet_inspection
        .pointer("/privacyBoundary/rawLocalOnly")
        .and_then(Value::as_array)
    else {
        return 0;
    };
    artifacts
        .iter()
        .filter(|artifact| {
            artifact
                .get("containsSensitivePatterns")
                .and_then(Value::as_bool)
                .unwrap_or(false)
        })
        .count()
}

pub(super) fn raw_local_sensitive_categories(packet_inspection: &Value) -> Vec<String> {
    let mut categories = Vec::new();
    let Some(artifacts) = packet_inspection
        .pointer("/privacyBoundary/rawLocalOnly")
        .and_then(Value::as_array)
    else {
        return categories;
    };
    for artifact in artifacts {
        let Some(items) = artifact
            .get("sensitiveCategories")
            .and_then(Value::as_array)
        else {
            continue;
        };
        for item in items {
            let Some(category) = item.as_str() else {
                continue;
            };
            if !categories.iter().any(|existing| existing == category) {
                categories.push(category.to_string());
            }
        }
    }
    categories
}

pub(super) fn narration_density_status(packet_inspection: &Value) -> &str {
    packet_inspection
        .pointer("/packetUsefulnessReview/signals/narrationDensityStatus")
        .and_then(Value::as_str)
        .unwrap_or("not-generated")
}

pub(super) fn transcript_word_count(packet_inspection: &Value) -> u64 {
    packet_inspection
        .pointer("/packetUsefulnessReview/signals/transcriptWordCount")
        .and_then(Value::as_u64)
        .unwrap_or(0)
}

pub(super) fn transcript_char_count(packet_inspection: &Value) -> u64 {
    packet_inspection
        .pointer("/packetUsefulnessReview/signals/transcriptCharCount")
        .and_then(Value::as_u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[cfg(unix)]
    #[test]
    fn packet_inspection_reader_ignores_symlinked_json() {
        let root = unique_tmp("nrr-review-inspection-symlink");
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("target.json"), r#"{"status":"leaked"}"#).unwrap();
        std::os::unix::fs::symlink(
            root.join("target.json"),
            root.join("packet-inspection.json"),
        )
        .unwrap();

        assert_eq!(
            read_packet_inspection(&root.join("packet-inspection.json")),
            Value::Null
        );
    }

    #[test]
    fn inspection_helpers_default_without_generated_packet() {
        let empty = Value::Null;

        assert_eq!(inspection_status(&empty), "not-generated");
        assert_eq!(leak_scan_status(&empty), "not-generated");
        assert_eq!(leak_finding_count(&empty), 0);
        assert_eq!(blocking_leak_finding_count(&empty), 0);
        assert!(leak_categories(&empty).is_empty());
        assert_eq!(raw_local_sensitive_artifact_count(&empty), 0);
        assert!(raw_local_sensitive_categories(&empty).is_empty());
        assert_eq!(narration_density_status(&empty), "not-generated");
        assert_eq!(transcript_word_count(&empty), 0);
        assert_eq!(transcript_char_count(&empty), 0);
    }

    #[test]
    fn inspection_helpers_deduplicate_categories_and_count_blockers() {
        let packet = serde_json::json!({
            "status": "passed",
            "privacyBoundary": {
                "generatedArtifactLeakScan": {
                    "status": "blocked",
                    "findings": [
                        {"blocksShare": true, "categories": ["path", "secret"]},
                        {"blocksShare": false, "categories": ["path", 7]},
                        {"categories": ["default-blocking"]}
                    ]
                },
                "rawLocalOnly": [
                    {"containsSensitivePatterns": true, "sensitiveCategories": ["audio"]},
                    {"containsSensitivePatterns": false, "sensitiveCategories": ["audio"]},
                    {"containsSensitivePatterns": true, "sensitiveCategories": ["audio", "path"]}
                ]
            },
            "packetUsefulnessReview": {
                "signals": {
                    "narrationDensityStatus": "sufficient",
                    "transcriptWordCount": 12,
                    "transcriptCharCount": 34
                }
            }
        });

        assert_eq!(inspection_status(&packet), "passed");
        assert_eq!(leak_scan_status(&packet), "blocked");
        assert_eq!(leak_finding_count(&packet), 3);
        assert_eq!(blocking_leak_finding_count(&packet), 2);
        assert_eq!(
            leak_categories(&packet),
            ["path", "secret", "default-blocking"]
        );
        assert_eq!(raw_local_sensitive_artifact_count(&packet), 2);
        assert_eq!(raw_local_sensitive_categories(&packet), ["audio", "path"]);
        assert_eq!(narration_density_status(&packet), "sufficient");
        assert_eq!(transcript_word_count(&packet), 12);
        assert_eq!(transcript_char_count(&packet), 34);
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
