use super::*;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn private_dir_accepts_existing_directory_and_missing_target() {
    let root = unique_tmp("nrr-safe-extra-dir");
    fs::create_dir_all(&root).unwrap();

    assert!(validate_private_dir(&root).is_ok());
    assert!(validate_private_dir(&root.join("future-dir")).is_ok());
}

#[test]
fn private_write_path_accepts_regular_file_and_missing_target() {
    let root = unique_tmp("nrr-safe-extra-write");
    fs::create_dir_all(&root).unwrap();
    let file = root.join("artifact.json");
    fs::write(&file, "{}").unwrap();

    assert!(validate_private_write_path(&file).is_ok());
    assert!(validate_private_write_path(&root.join("future.json")).is_ok());
}

#[cfg(unix)]
#[test]
fn private_write_and_regular_file_reject_final_symlink() {
    let root = unique_tmp("nrr-safe-extra-final-link");
    fs::create_dir_all(&root).unwrap();
    let target = root.join("target.json");
    let link = root.join("link.json");
    fs::write(&target, "{}").unwrap();
    std::os::unix::fs::symlink(&target, &link).unwrap();

    assert!(validate_private_write_path(&link).is_err());
    assert!(regular_file_metadata(&link).is_err());
}

#[test]
fn bounded_reader_rejects_oversize_and_invalid_utf8() {
    let root = unique_tmp("nrr-safe-extra-bounded");
    fs::create_dir_all(&root).unwrap();
    let oversize = root.join("oversize.bin");
    fs::write(&oversize, b"12345").unwrap();
    assert!(read_regular_file_bounded(&oversize, 4).is_err());
    assert_eq!(read_regular_file_bounded(&oversize, 5).unwrap(), b"12345");
    let invalid = root.join("invalid.txt");
    fs::write(&invalid, [0xff]).unwrap();
    assert!(read_regular_text_bounded(&invalid, 4).is_err());
}

#[test]
fn path_validation_and_private_runtime_boundaries_hold() {
    let root = unique_tmp("nrr-safe-extra-file");
    fs::create_dir_all(&root).unwrap();
    let file = root.join("not-dir");
    fs::write(&file, "not a dir").unwrap();
    assert!(validate_cli_path("--session-dir", Path::new("/tmp/nrr/../escape")).is_err());
    assert!(validate_private_dir(&file).is_err());
    assert!(validate_private_write_path(&root).is_err());
    assert!(regular_file_metadata(&root.join("missing.txt")).is_err());
    assert!(regular_file_metadata(&root).is_err());
    assert!(
        validate_private_runtime_path(
            "--session-dir",
            Path::new("/private/tmp/narrated-record-replay/run"),
            false
        )
        .is_ok()
    );
    assert!(
        validate_private_runtime_path("--session-dir", Path::new("/private/tmp/other"), false)
            .is_err()
    );
}

#[cfg(unix)]
#[test]
fn rejects_symlinked_path_component() {
    let root = unique_tmp("nrr-safe-extra-component-link");
    fs::create_dir_all(&root).unwrap();
    let target = root.join("target");
    fs::create_dir(&target).unwrap();
    let link = root.join("link");
    std::os::unix::fs::symlink(&target, &link).unwrap();
    assert!(validate_cli_path("--session-dir", &link.join("session")).is_err());
}

#[cfg(target_os = "macos")]
#[test]
fn normalizes_macos_var_alias() {
    assert_eq!(
        normalize_system_temp(Path::new("/var/folders/example")),
        Path::new("/private/var/folders/example")
    );
}

fn unique_tmp(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}-{nanos}"))
}
