use crate::config::Args;
use std::path::Path;

pub(super) fn quote_token(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

pub(super) fn join_tokens(tokens: &[String]) -> String {
    tokens
        .iter()
        .map(|token| quote_token(token))
        .collect::<Vec<_>>()
        .join(" ")
}

pub(super) fn capture_template(
    helper_exe: &Path,
    session_dir: &Path,
    args: &Args,
    max_seconds: u64,
) -> String {
    let mut tokens = vec![
        helper_exe.to_string_lossy().into_owned(),
        "capture".into(),
        "--session-dir".into(),
        session_dir.to_string_lossy().into_owned(),
        "--delay".into(),
        args.delay.clone(),
        "--input".into(),
        args.input.clone(),
        "--max-seconds".into(),
        max_seconds.to_string(),
        "--record-replay-status".into(),
        "idle".into(),
        "--audio-retention-mode".into(),
        args.audio_retention_mode.clone(),
        "--audio-filter".into(),
        args.audio_filter.clone(),
    ];
    if let Some(path) = &args.audio_retention_path {
        tokens.extend([
            "--audio-retention-path".into(),
            path.to_string_lossy().into_owned(),
        ]);
    }
    tokens.push("--i-consent-to-microphone-capture".into());
    join_tokens(&tokens)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn capture_template_keeps_substitution_inert_and_round_trips_quotes() {
        let marker = marker_path("capture");
        let legacy_marker = marker_path("capture-legacy");
        let injection = format!(
            "$(touch {}) `touch {}`",
            marker.display(),
            legacy_marker.display()
        );
        let root = format!("/private/tmp/nrr quoted' root {injection}");
        let input = format!(":Studio mic's input {injection}");
        let filter = format!("highpass=f=80,volume='1.2' {injection}");
        let retention = format!("{root}/retained audio's.wav");
        let args = crate::config::parse_args_from([
            "nrr",
            "prepare-coordinated-session",
            "--root",
            &root,
            "--input",
            &input,
            "--audio-filter",
            &filter,
            "--audio-retention-path",
            &retention,
            "--i-consent-to-custom-runtime-paths",
            "--i-consent-to-custom-audio-filter",
        ])
        .unwrap();

        let rendered = capture_template(
            Path::new("/private/tmp/helper's binary"),
            Path::new(&format!("{root}/session's dir")),
            &args,
            30,
        );
        let parsed = parse_without_execution(&rendered);

        assert!(!marker.exists());
        assert!(!legacy_marker.exists());
        assert!(parsed.contains(&input));
        assert!(parsed.contains(&filter));
        assert!(parsed.contains(&retention));
        assert!(parsed.contains("/private/tmp/helper's binary"));
    }

    fn parse_without_execution(command: &str) -> String {
        let output = Command::new("/bin/sh")
            .arg("-c")
            .arg(format!("set -- {command}; printf '%s\\n' \"$@\""))
            .output()
            .unwrap();
        assert!(output.status.success());
        String::from_utf8(output.stdout).unwrap()
    }

    fn marker_path(label: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::path::PathBuf::from(format!("/private/tmp/nrr-{label}-{nanos}"))
    }
}
