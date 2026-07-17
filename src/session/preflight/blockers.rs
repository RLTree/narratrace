fn preflight_blockers(
    ffmpeg: bool,
    has_openai_key: bool,
    goal_provided: bool,
    record_replay_status: Option<&str>,
) -> Vec<&'static str> {
    let mut blockers = Vec::new();
    if !ffmpeg {
        blockers.push("ffmpeg is missing");
    }
    if !has_openai_key {
        blockers.push("OPENAI_API_KEY is missing");
    }
    let _ = goal_provided;
    match record_replay_status {
        Some("idle") => {}
        Some("recording") => blockers.push("Record & Replay is already recording"),
        Some("unavailable") => blockers.push("Record & Replay event-stream status is unavailable"),
        _ => {
            blockers.push("Record & Replay event-stream status must be confirmed outside this CLI")
        }
    }
    blockers
}

#[cfg(test)]
mod blocker_tests {
    use super::preflight_blockers;

    #[test]
    fn preflight_blocks_missing_local_prerequisites_and_unknown_record_replay() {
        let blockers = preflight_blockers(false, false, false, None);

        assert!(blockers.contains(&"ffmpeg is missing"));
        assert!(blockers.contains(&"OPENAI_API_KEY is missing"));
        assert!(blockers
            .contains(&"Record & Replay event-stream status must be confirmed outside this CLI"));
    }

    #[test]
    fn preflight_accepts_idle_record_replay_when_local_prerequisites_exist() {
        let blockers = preflight_blockers(true, true, true, Some("idle"));

        assert!(blockers.is_empty());
    }

    #[test]
    fn preflight_blocks_recording_or_unavailable_record_replay() {
        assert_eq!(
            preflight_blockers(true, true, true, Some("recording")),
            vec!["Record & Replay is already recording"]
        );
        assert_eq!(
            preflight_blockers(true, true, true, Some("unavailable")),
            vec!["Record & Replay event-stream status is unavailable"]
        );
    }
}
