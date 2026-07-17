fn blocking_leak_count(leak_scan: &Value) -> usize {
    leak_scan
        .get("findings")
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

fn blockers(
    temporal_path: &Path,
    evidence_path: &Path,
    review_contract_path: &Path,
    packet_path: &Path,
    evidence: &Value,
    conflict_warnings: usize,
    leak_count: usize,
) -> Vec<&'static str> {
    let mut blockers = Vec::new();
    if !regular_file_exists(packet_path) {
        blockers.push("skill-refinement-packet.md is missing");
    }
    if !regular_file_exists(temporal_path) {
        blockers.push("temporal-context.json is missing");
    }
    if !regular_file_exists(evidence_path) {
        blockers.push("evidence-boundary-report.json is missing");
    }
    if !regular_file_exists(review_contract_path) {
        blockers.push("review-contract.json is missing");
    }
    if provided_external_artifact_unusable(
        evidence,
        "/evidenceSurfaces/recordReplayArtifacts/metadata",
    ) {
        blockers.push("provided Record & Replay metadata artifact is missing or empty");
    }
    if provided_external_artifact_unusable(
        evidence,
        "/evidenceSurfaces/recordReplayArtifacts/events",
    ) {
        blockers.push("provided Record & Replay events artifact is missing or empty");
    }
    if conflict_warnings > 0 {
        blockers.push("conflict warnings require operator review");
    }
    if leak_count > 0 {
        blockers.push("generated review candidates contain obvious unredacted sensitive patterns");
    }
    blockers.push("real non-toy workflow packet usefulness inspection is still owed");
    blockers.push("raw-private leakage inspection is still owed before sharing");
    blockers
}

fn provided_external_artifact_unusable(evidence: &Value, pointer: &str) -> bool {
    let artifact = evidence.pointer(pointer).unwrap_or(&Value::Null);
    artifact
        .get("provided")
        .and_then(Value::as_bool)
        .unwrap_or(false)
        && !artifact
            .get("usableForLiveProof")
            .and_then(Value::as_bool)
            .unwrap_or(false)
}

fn path_exists(path: &Path) -> Value {
    json!({
        "path": path.display().to_string(),
        "exists": regular_file_exists(path)
    })
}

fn existing(path: &Path) -> Option<&Path> {
    if regular_file_exists(path) {
        Some(path)
    } else {
        None
    }
}

fn read_json(path: &Path) -> Result<Value> {
    Ok(serde_json::from_str(&read_regular_text_bounded(
        path,
        MAX_INSPECT_JSON_BYTES,
    )?)?)
}

fn regular_file_exists(path: &Path) -> bool {
    regular_file_metadata(path).is_ok()
}

fn approved_record_replay_paths(evidence: &Value) -> Vec<PathBuf> {
    [
        "/evidenceSurfaces/recordReplayArtifacts/metadata",
        "/evidenceSurfaces/recordReplayArtifacts/events",
    ]
    .iter()
    .filter_map(|pointer| evidence.pointer(pointer))
    .filter(|artifact| {
        artifact.get("provided").and_then(Value::as_bool) == Some(true)
            && artifact.get("safeRegularFile").and_then(Value::as_bool) == Some(true)
            && artifact.get("usableForLiveProof").and_then(Value::as_bool) == Some(true)
    })
    .filter_map(|artifact| artifact.get("path").and_then(Value::as_str))
    .map(PathBuf::from)
    .filter(|path| regular_file_metadata(path).is_ok())
    .collect()
}
