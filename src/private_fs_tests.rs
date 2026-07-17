use super::*;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn atomic_write_replaces_owned_file_but_exclusive_create_refuses_it() {
    let file = unique_tmp("nrr-private-owned").join("artifact.txt");
    write_private(&file, "one").unwrap();
    write_private(&file, "two").unwrap();
    assert_eq!(fs::read_to_string(&file).unwrap(), "two");
    assert!(create_private_file(&file).is_err());
}

#[cfg(unix)]
#[test]
fn write_private_rejects_symlinked_ancestor_and_final_component() {
    let root = unique_tmp("nrr-private-symlink");
    fs::create_dir_all(root.join("target")).unwrap();
    std::os::unix::fs::symlink(root.join("target"), root.join("link")).unwrap();
    assert!(write_private(root.join("link/file"), "blocked").is_err());
    fs::write(root.join("target.txt"), "target").unwrap();
    std::os::unix::fs::symlink(root.join("target.txt"), root.join("final.txt")).unwrap();
    assert!(write_private(root.join("final.txt"), "blocked").is_err());
    assert_eq!(
        fs::read_to_string(root.join("target.txt")).unwrap(),
        "target"
    );
}

#[cfg(unix)]
#[test]
fn private_modes_are_applied_to_open_descriptors() {
    let root = unique_tmp("nrr-private-modes");
    let file = root.join("nested/artifact.txt");
    write_private(&file, "secret").unwrap();
    assert_eq!(
        fs::metadata(root.join("nested"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o700
    );
    assert_eq!(
        fs::metadata(file).unwrap().permissions().mode() & 0o777,
        0o600
    );
}

#[cfg(unix)]
#[test]
fn opened_parent_descriptor_cannot_be_redirected_by_path_replacement() {
    let root = unique_tmp("nrr-private-parent-race");
    let safe = root.join("safe");
    let moved = root.join("moved");
    let attacker = root.join("attacker");
    fs::create_dir_all(&safe).unwrap();
    fs::create_dir_all(&attacker).unwrap();
    let dir = open_dir_chain(&safe, false).unwrap();
    fs::rename(&safe, &moved).unwrap();
    std::os::unix::fs::symlink(&attacker, &safe).unwrap();
    let name = CString::new("artifact.txt").unwrap();
    let mut file = open_at(&dir, &name, OpenKind::Exclusive).unwrap();
    file.write_all(b"descriptor-bound").unwrap();
    assert_eq!(
        fs::read_to_string(moved.join("artifact.txt")).unwrap(),
        "descriptor-bound"
    );
    assert!(!attacker.join("artifact.txt").exists());
}

#[cfg(unix)]
#[test]
fn permission_changes_remain_bound_to_open_file_after_replacement() {
    let root = unique_tmp("nrr-private-chmod-race");
    fs::create_dir_all(&root).unwrap();
    let selected = root.join("selected.txt");
    let opened = root.join("opened.txt");
    let victim = root.join("victim.txt");
    fs::write(&victim, "victim").unwrap();
    fs::set_permissions(&victim, fs::Permissions::from_mode(0o644)).unwrap();
    let file = create_private_file(&selected).unwrap();
    fs::rename(&selected, &opened).unwrap();
    std::os::unix::fs::symlink(&victim, &selected).unwrap();
    file.set_permissions(fs::Permissions::from_mode(0o600))
        .unwrap();
    assert_eq!(
        fs::metadata(victim).unwrap().permissions().mode() & 0o777,
        0o644
    );
    assert_eq!(
        fs::metadata(opened).unwrap().permissions().mode() & 0o777,
        0o600
    );
}

#[cfg(unix)]
#[test]
fn private_write_refuses_non_regular_socket_destination() {
    let root = unique_tmp("nrr-private-socket");
    fs::create_dir_all(&root).unwrap();
    let socket = root.join("destination.sock");
    let _listener = std::os::unix::net::UnixListener::bind(&socket).unwrap();
    assert!(append_private(&socket, "blocked").is_err());
}

#[test]
fn create_private_file_rejects_parent_escape() {
    assert!(create_private_file(Path::new("/tmp/nrr/../escape.txt")).is_err());
}

fn unique_tmp(prefix: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::path::PathBuf::from("/private/tmp").join(format!("{prefix}-{nanos}"))
}
