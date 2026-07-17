use super::*;
use crate::config::parse_args_from;

#[tokio::test]
async fn run_with_args_routes_early_validation_failures_without_external_io() {
    for (command, expected) in [
        ("start", "--i-consent-to-microphone-capture"),
        ("packet", "--session-dir is required"),
        ("parent-operation-receipt", "--session-dir is required"),
        ("receipt", "--session-dir is required"),
        ("inspect", "--session-dir is required"),
        ("capture", "--session-dir is required"),
    ] {
        let args = parse_args_from(["nrr", command]).unwrap();
        let error = run_with_args(args).await.unwrap_err().to_string();
        assert!(error.contains(expected), "{command}: {error}");
    }
}
