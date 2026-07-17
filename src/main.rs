mod audio_input;
mod audio_retention;
mod batch_transcribe;
mod bundle;
mod config;
mod coverage;
mod delay_eval;
mod inspect;
mod packet;
mod parent_operation;
mod private_fs;
mod realtime;
mod receipt;
mod redaction;
mod replay;
mod review;
mod safe_path;
mod session;
mod timeline;
mod transcript_alignment;
mod transcript_cleanup;
mod voice;

use anyhow::Result;
use config::{parse_args, usage};

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let args = parse_args()?;
    run_with_args(args).await
}

async fn run_with_args(args: config::Args) -> Result<()> {
    let command = args.command.clone();
    match command.as_str() {
        "help" => {
            println!("{}", usage());
            Ok(())
        }
        "preflight" => session::preflight(&args).await,
        "validate" => session::validate(&args).await,
        "validate-bundle" => bundle::validate_bundle(&args),
        "refresh-bundle-receipt" => bundle::refresh_bundle_receipt(&args),
        "check-coverage-policy" => coverage::check_coverage_policy(&args),
        "coverage-receipt" => coverage::write_coverage_receipt(&args),
        "prepare-coordinated-session" => session::prepare_coordinated_session(&args),
        "start" => session::start(&args),
        "status" => session::status(&args),
        "stop" => session::stop(&args).await,
        "packet" => tokio::task::spawn_blocking(move || packet::make_packet(&args)).await?,
        "parent-operation-receipt" => parent_operation::write_parent_operation_receipt(&args),
        "receipt" => receipt::write_receipt(&args),
        "delay-eval" => delay_eval::write_delay_evaluation(&args),
        "delay-compare" => delay_eval::write_delay_comparison(&args),
        "inspect" => inspect::inspect_packet(&args),
        "review" => review::make_review(&args),
        "replay-voice-preview" => replay::preview_replay_voice(&args),
        "capture" => realtime::capture(&args).await,
        other => anyhow::bail!("unknown command: {other}\n{}", usage()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::parse_args_from;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[tokio::test]
    async fn run_with_args_accepts_help_command() {
        let args = parse_args_from(["nrr", "help"]).unwrap();

        run_with_args(args).await.unwrap();
    }

    #[tokio::test]
    async fn run_with_args_rejects_unknown_command() {
        let args = parse_args_from(["nrr", "not-a-command"]).unwrap();
        let error = run_with_args(args).await.unwrap_err().to_string();

        assert!(error.contains("unknown command: not-a-command"));
    }

    #[tokio::test]
    async fn run_with_args_routes_status_and_stop_commands() {
        let root = unique_tmp("nrr-test/nrr-main-status-stop");
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("status.json"), r#"{"state":"stopped"}"#).unwrap();
        let session_dir = root.to_str().unwrap();

        run_with_args(custom_session_args("status", session_dir))
            .await
            .unwrap();
        run_with_args(custom_session_args("stop", session_dir))
            .await
            .unwrap();

        assert!(root.join(".stop").is_file());
    }

    #[tokio::test]
    async fn run_with_args_routes_review_and_replay_preview_commands() {
        let root = unique_tmp("nrr-test/nrr-main-review-replay");
        fs::create_dir_all(&root).unwrap();
        fs::write(
            root.join("temporal-context.json"),
            r#"{"alignmentDiagnostics":{"claimCeiling":"dispatch test"},"conflictDiagnostics":{"warnings":[]}}"#,
        )
        .unwrap();
        fs::write(
            root.join("replay-voice-parameters.json"),
            r#"{"segmentBindings":[{"timelineBinding":{"startMs":0,"endMs":1000},"voice":{"style":"neutral","pace":"normal","emphasis":"balanced"}}]}"#,
        )
        .unwrap();
        let session_dir = root.to_str().unwrap();

        run_with_args(custom_session_args("review", session_dir))
            .await
            .unwrap();
        run_with_args(custom_session_args("replay-voice-preview", session_dir))
            .await
            .unwrap();

        assert!(root.join("review-contract.json").is_file());
        assert!(root.join("replay-voice-execution-plan.json").is_file());
    }

    #[tokio::test]
    async fn run_with_args_routes_preflight_validate_and_bundle_commands() {
        run_with_args(parse_args_from(["nrr", "preflight", "--json"]).unwrap())
            .await
            .unwrap();
        run_with_args(parse_args_from(["nrr", "validate"]).unwrap())
            .await
            .unwrap();

        let error = run_with_args(parse_args_from(["nrr", "validate-bundle"]).unwrap())
            .await
            .unwrap_err()
            .to_string();
        assert!(error.contains("--skill-dir is required"));
    }

    #[tokio::test]
    async fn run_with_args_routes_coverage_policy_receipt_and_delay_commands() {
        let receipt_error = run_with_args(parse_args_from(["nrr", "coverage-receipt"]).unwrap())
            .await
            .unwrap_err()
            .to_string();
        assert!(receipt_error.contains("--skill-dir is required"));
        let policy_error =
            run_with_args(parse_args_from(["nrr", "check-coverage-policy"]).unwrap())
                .await
                .unwrap_err()
                .to_string();
        assert!(policy_error.contains("--skill-dir is required"));

        let error = run_with_args(parse_args_from(["nrr", "delay-eval"]).unwrap())
            .await
            .unwrap_err()
            .to_string();
        assert!(error.contains("--session-dir is required"));
        let error = run_with_args(parse_args_from(["nrr", "delay-compare"]).unwrap())
            .await
            .unwrap_err()
            .to_string();
        assert!(!error.is_empty());
    }

    fn unique_tmp(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        PathBuf::from("/private/tmp").join(format!("{prefix}-{nanos}"))
    }

    fn custom_session_args(command: &str, session_dir: &str) -> crate::config::Args {
        parse_args_from([
            "nrr",
            command,
            "--session-dir",
            session_dir,
            "--i-consent-to-custom-runtime-paths",
        ])
        .unwrap()
    }
}

#[cfg(test)]
mod main_extra_tests;
