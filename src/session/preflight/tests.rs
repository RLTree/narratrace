#[cfg(test)]
mod tests {
    use super::{app_helper_command_prefix, audio_preview_resolved, preflight_payload};
    use serde_json::json;
    use std::process::Command;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn app_helper_command_prefix_uses_installed_absolute_manifest() {
        let command = app_helper_command_prefix();
        assert!(command.contains(
            "/Users/terrynoblin/.codex/plugins/narrated-record-replay/skills/narrated-record-replay/Cargo.toml"
        ));
        assert!(!command.contains(".codex/skills/narrated-record-replay/Cargo.toml"));
    }

    #[test]
    fn audio_preview_ready_requires_resolved_non_rejected_input() {
        assert!(audio_preview_resolved(&json!({
            "status": "resolved",
            "rejectsIphoneOrVirtualInput": false
        })));
        assert!(!audio_preview_resolved(&json!({
            "status": "unresolved",
            "rejectsIphoneOrVirtualInput": false
        })));
        assert!(!audio_preview_resolved(&json!({
            "status": "resolved",
            "rejectsIphoneOrVirtualInput": true
        })));
    }

    #[test]
    fn live_dogfood_plan_requires_coordinated_start_and_private_artifacts() {
        let plan = super::live_dogfood_plan("prepare-command");

        assert_eq!(plan["status"], "plan-only-not-executed");
        assert_eq!(
            plan["startCoordination"]["manualSequentialStartAllowedForLiveProof"],
            false
        );
        assert_eq!(plan["privacyBoundary"]["copyRawAudioIntoSkillFiles"], false);
        assert_eq!(plan["steps"].as_array().unwrap().len(), 10);
        assert!(
            plan["steps"][2]["proof"]
                .as_str()
                .unwrap()
                .contains("same orchestrated operation")
        );
    }

    #[test]
    fn preflight_payload_requires_all_capture_prerequisites() {
        let args = crate::config::parse_args_from([
            "nrr",
            "preflight",
            "--goal",
            "demo",
            "--record-replay-status",
            "idle",
            "--json",
        ])
        .unwrap();
        let preview = json!({
            "status": "resolved",
            "deviceName": "MacBook Pro Microphone",
            "rejectsIphoneOrVirtualInput": false
        });

        let payload = preflight_payload(&args, true, true, preview);

        assert_eq!(payload["readyForLiveNarratedCapture"], true);
        assert_eq!(payload["doesNotStartRecordReplay"], true);
        assert_eq!(payload["opensMicrophone"], false);
        assert_eq!(payload["callsOpenAI"], false);
        assert_eq!(
            payload["transcriptionQualityPipeline"]["realtimeTimingSpine"]["wordAuthority"],
            "timing-only"
        );
        assert!(
            payload["recommendedCommand"]
                .as_str()
                .unwrap()
                .contains("prepare-coordinated-session")
        );
    }

    #[test]
    fn recommended_command_keeps_substitution_inert_and_round_trips_quotes() {
        let marker = marker_path();
        let legacy_marker = marker_path();
        let injection = format!(
            "$(touch {}) `touch {}`",
            marker.display(),
            legacy_marker.display()
        );
        let goal = format!("Demo's spaced goal {injection}");
        let root = format!("/private/tmp/nrr root's {injection}");
        let input = format!(":Studio mic's input {injection}");
        let args = crate::config::parse_args_from([
            "nrr",
            "preflight",
            "--goal",
            &goal,
            "--root",
            &root,
            "--input",
            &input,
            "--i-consent-to-custom-runtime-paths",
        ])
        .unwrap();

        let payload = preflight_payload(&args, true, true, json!({}));
        let command = payload["recommendedCommand"].as_str().unwrap();
        let output = Command::new("/bin/sh")
            .arg("-c")
            .arg(format!("set -- {command}; printf '%s\\n' \"$@\""))
            .output()
            .unwrap();
        let parsed = String::from_utf8(output.stdout).unwrap();

        assert!(output.status.success());
        assert!(!marker.exists());
        assert!(!legacy_marker.exists());
        assert!(parsed.contains(&goal));
        assert!(parsed.contains(&root));
        assert!(parsed.contains(&input));
    }

    #[test]
    fn preflight_payload_surfaces_blockers_without_fake_readiness() {
        let args = crate::config::parse_args_from(["nrr", "preflight"]).unwrap();
        let preview = json!({
            "status": "resolved",
            "deviceName": "Terry's iPhone Microphone",
            "rejectsIphoneOrVirtualInput": true
        });

        let payload = preflight_payload(&args, false, false, preview);

        assert_eq!(payload["readyForLiveNarratedCapture"], false);
        assert_eq!(payload["localPrerequisitesReady"], false);
        assert_eq!(payload["recordReplayReady"], false);
        assert_eq!(payload["recordReplayStatus"]["status"], "not-confirmed");
        assert!(payload["blockers"].as_array().unwrap().len() >= 3);
    }

    #[tokio::test]
    async fn preflight_command_runs_without_starting_capture_json() {
        let args = crate::config::parse_args_from(["nrr", "preflight", "--json"]).unwrap();

        super::preflight(&args).await.unwrap();
    }

    #[tokio::test]
    async fn preflight_command_runs_without_starting_capture_text() {
        let args = crate::config::parse_args_from(["nrr", "preflight"]).unwrap();

        super::preflight(&args).await.unwrap();
    }

    fn marker_path() -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::path::PathBuf::from(format!("/private/tmp/nrr-preflight-{nanos}"))
    }
}
