use super::*;
use std::process::Command;

#[cfg(unix)]
#[test]
fn status_read_keeps_verified_handle_when_path_is_replaced() {
    let root = unique_tmp("nrr-status-handle");
    fs::create_dir_all(&root).unwrap();
    let status_path = root.join("status.json");
    let sensitive_path = root.join("sensitive.json");
    fs::write(&status_path, r#"{"state":"recording"}"#).unwrap();
    fs::write(&sensitive_path, r#"{"secret":"must-not-read"}"#).unwrap();

    let text = read_regular_text_after_open(&status_path, || {
        fs::remove_file(&status_path).unwrap();
        std::os::unix::fs::symlink(&sensitive_path, &status_path).unwrap();
    })
    .unwrap();

    assert_eq!(text, r#"{"state":"recording"}"#);
    assert!(!text.contains("must-not-read"));
}

#[test]
fn status_read_rejects_oversized_artifact() {
    let root = unique_tmp("nrr-status-oversized");
    fs::create_dir_all(&root).unwrap();
    let status_path = root.join("status.json");
    fs::write(&status_path, vec![b'x'; 64 * 1024 + 1]).unwrap();

    let error = read_regular_text(&status_path).unwrap_err().to_string();

    assert!(error.contains("exceeds 65536 bytes"));
}

#[test]
fn session_allocation_retries_collisions_without_reusing_existing_directory() {
    let root = unique_tmp("nrr-session-collision");
    create_private_dir_all(&root).unwrap();
    let occupied = root.join("occupied-session");
    create_private_dir_all(&occupied).unwrap();
    fs::write(occupied.join("sentinel"), "existing").unwrap();
    let mut names = ["occupied-session", "fresh-session"].into_iter();

    let allocated =
        allocate_session_dir_with(&root, || Ok(names.next().unwrap().to_owned())).unwrap();

    assert_eq!(allocated, root.join("fresh-session"));
    assert_eq!(
        fs::read_to_string(occupied.join("sentinel")).unwrap(),
        "existing"
    );
}

#[test]
fn session_nonce_uses_unpredictable_128_bit_identifier() {
    let first = random_session_nonce().unwrap();
    let second = random_session_nonce().unwrap();

    assert_eq!(first.len(), 32);
    assert!(first.bytes().all(|byte| byte.is_ascii_hexdigit()));
    assert_ne!(first, second);
}

#[tokio::test]
async fn stop_does_not_accept_terminal_words_outside_typed_state() {
    let root = unique_tmp("nrr-stop-state-confusion");
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("status.json"),
        r#"{"state":"recording","device":"stopped","error":"failed"}"#,
    )
    .unwrap();
    let args = custom_session_args("stop", &root);

    let error = stop_with_poll(&args, 1, Duration::from_millis(0))
        .await
        .unwrap_err()
        .to_string();
    let receipt: Value =
        serde_json::from_str(&read_regular_text(&root.join("stop-timeout.json")).unwrap()).unwrap();

    assert!(error.contains("stop timed out"));
    assert_eq!(receipt["lastStatus"]["state"], "stop-requested");
}

#[cfg(unix)]
#[test]
fn setup_env_keeps_shell_syntax_inert_and_replaces_symlink() {
    let root = unique_tmp("nrr-worktree-env");
    let state_root = root.join(".codex-worktree");
    fs::create_dir_all(&state_root).unwrap();
    let victim = root.join("victim.txt");
    let marker = root.join("injection-marker");
    fs::write(&victim, "original").unwrap();
    std::os::unix::fs::symlink(&victim, state_root.join("env.sh")).unwrap();
    let cargo_target = root.join(format!(
        "cargo $(touch {}) `touch {}` \"quote\"\nnewline",
        marker.display(),
        marker.with_extension("legacy").display()
    ));
    let setup = Path::new(env!("CARGO_MANIFEST_DIR")).join(".codex/setup-worktree-env.sh");

    let output = Command::new("/bin/bash")
        .arg(&setup)
        .env("CODEX_WORKTREE_PATH", &root)
        .env("CARGO_TARGET_DIR", &cargo_target)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(fs::read_to_string(&victim).unwrap(), "original");
    assert!(
        !fs::symlink_metadata(state_root.join("env.sh"))
            .unwrap()
            .file_type()
            .is_symlink()
    );
    let sourced = Command::new("/bin/bash")
        .arg("-c")
        .arg(". \"$1\"; printf '%s' \"$CARGO_TARGET_DIR\"")
        .arg("nrr-test")
        .arg(state_root.join("env.sh"))
        .output()
        .unwrap();
    assert!(sourced.status.success());
    assert_eq!(sourced.stdout, cargo_target.as_os_str().as_encoded_bytes());
    assert!(!marker.exists());
    assert!(!marker.with_extension("legacy").exists());
}

#[test]
fn cleanup_rejects_declared_worktree_that_differs_from_runner_directory() {
    let runner = unique_tmp("nrr-cleanup-runner");
    let victim = unique_tmp("nrr-cleanup-victim");
    fs::create_dir_all(runner.join(".codex-worktree")).unwrap();
    fs::create_dir_all(victim.join(".codex-worktree")).unwrap();
    fs::write(runner.join(".codex-worktree/sentinel"), "runner").unwrap();
    fs::write(victim.join(".codex-worktree/sentinel"), "victim").unwrap();
    let config = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join(".codex/environments/environment.toml"),
    )
    .unwrap();
    let cleanup = config
        .split("[cleanup]")
        .nth(1)
        .unwrap()
        .split("[[actions]]")
        .next()
        .unwrap()
        .split("script = '''\n")
        .nth(1)
        .unwrap()
        .strip_suffix("'''\n\n")
        .unwrap();

    let rejected = Command::new("/bin/bash")
        .arg("-c")
        .arg(cleanup)
        .current_dir(&runner)
        .env("CODEX_WORKTREE_PATH", &victim)
        .output()
        .unwrap();

    assert_eq!(rejected.status.code(), Some(2));
    assert!(runner.join(".codex-worktree/sentinel").is_file());
    assert!(victim.join(".codex-worktree/sentinel").is_file());
}

fn unique_tmp(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    PathBuf::from("/private/tmp").join(format!("{prefix}-{nanos}"))
}

fn custom_session_args(command: &str, root: &Path) -> crate::config::Args {
    crate::config::parse_args_from([
        "nrr",
        command,
        "--session-dir",
        root.to_str().unwrap(),
        "--i-consent-to-custom-runtime-paths",
    ])
    .unwrap()
}
