fn live_dogfood_plan(prepare_command: &str) -> serde_json::Value {
    let session_dir = "<session-dir-from-start>";
    let metadata_path = "<metadata-path-from-event-stream-stop>";
    let events_path = "<events-path-from-event-stream-stop>";
    json!({
        "schema": "narrated-record-replay.live-dogfood-plan.v1",
        "status": "plan-only-not-executed",
        "doesNotStartRecordReplay": true,
        "opensMicrophone": false,
        "startCoordination": {
            "required": true,
            "manualSequentialStartAllowedForLiveProof": false,
            "requiredBehavior": "Record & Replay and microphone capture must be launched by one coordinated operation so their clocks and proof artifacts share a start boundary.",
            "currentCliLimitation": "The Rust helper can start microphone capture, but the app-visible Record & Replay start surface is currently exposed through MCP. A live proof must use an orchestrator that starts both surfaces as one operation or a future local Record & Replay API.",
            "claimCeiling": "Record & Replay and microphone capture must start from one coordinated operation before CLAIM-008 can close"
        },
        "requiredExternalArtifacts": {
            "recordingMetadata": metadata_path,
            "recordingEvents": events_path
        },
        "privacyBoundary": {
            "copyRawTranscriptIntoSkillFiles": false,
            "copyRawAudioIntoSkillFiles": false,
            "copyRawRecordReplayEventsIntoSkillFiles": false
        },
        "narrationQualityTargets": narration_quality_targets(),
        "steps": [
            {
                "order": 1,
                "actor": "agent",
                "action": "confirm-record-replay-idle",
                "command": "mcp__event_stream.event_stream_status",
                "proof": "isRecording=false"
            },
            {
                "order": 2,
                "actor": "agent",
                "action": "prepare-coordinated-session",
                "command": prepare_command,
                "proof": "session manifest records coordinated-orchestrator start mode before microphone opens"
            },
            {
                "order": 3,
                "actor": "agent",
                "action": "start-coordinated-recording-and-narration",
                "recordReplayCommand": "mcp__event_stream.event_stream_start",
                "narrationCommand": "capture command returned by prepare-coordinated-session",
                "proof": "Record & Replay metadata/events and narration session are created by the same orchestrated operation; manual sequential starts do not close live proof"
            },
            {
                "order": 4,
                "actor": "operator",
                "action": "narrate-workflow",
                "command": "speak workflow intent, decisions, failure modes, and replay success criteria",
                "proof": "transcript timeline contains completed segments"
            },
            {
                "order": 5,
                "actor": "agent",
                "action": "stop-record-replay",
                "command": "mcp__event_stream.event_stream_stop",
                "proof": "metadataPath and eventsPath point to non-empty files"
            },
            {
                "order": 6,
                "actor": "agent",
                "action": "stop-narrated-capture",
                "command": format!(
                    "{} -- stop --session-dir {}",
                    app_helper_command_prefix(), quote_token(session_dir)
                ),
                "proof": "status.json state is stopped"
            },
            {
                "order": 7,
                "actor": "agent",
                "action": "build-packet",
                "command": format!(
                    "{} -- packet --session-dir {} --recording-metadata {} --recording-events {} --i-consent-to-openai-postprocessing",
                    app_helper_command_prefix(), quote_token(session_dir),
                    quote_token(metadata_path), quote_token(events_path)
                ),
                "proof": "temporal-context.json and evidence-boundary-report.json are written"
            },
            {
                "order": 8,
                "actor": "agent",
                "action": "inspect-packet",
                "command": format!(
                    "{} -- inspect --session-dir {}",
                    app_helper_command_prefix(), quote_token(session_dir)
                ),
                "proof": "packet-inspection.json records blockers and privacy boundary"
            },
            {
                "order": 9,
                "actor": "agent",
                "action": "write-dogfood-receipt",
                "command": format!(
                    "{} -- receipt --session-dir {} --recording-metadata {} --recording-events {}",
                    app_helper_command_prefix(), quote_token(session_dir),
                    quote_token(metadata_path), quote_token(events_path)
                ),
                "proof": "dogfood-receipt.json records metadata-only artifact evidence and refreshes review-contract.json plus review-artifact.html"
            },
            {
                "order": 10,
                "actor": "operator",
                "action": "review-generated-artifacts",
                "command": "review packet-inspection.json, dogfood-receipt.json, review-contract.json, and review-artifact.html",
                "proof": "operator review result is recorded without raw transcript text"
            }
        ]
    })
}

fn app_helper_command_prefix() -> String {
    format!(
        "CARGO_TARGET_DIR={} {}",
        quote_token(APP_HELPER_CARGO_TARGET),
        join_tokens(&[
            "cargo".into(),
            "run".into(),
            "--manifest-path".into(),
            APP_HELPER_MANIFEST.into(),
        ])
    )
}
