use crate::safe_path::{normalize_system_temp, validate_private_dir, validate_private_write_path};
use anyhow::{Context, Result, bail};
#[cfg(unix)]
use std::ffi::CString;
use std::fs::{self, File};
use std::io::Write;
#[cfg(unix)]
use std::os::fd::{AsRawFd, FromRawFd, RawFd};
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::sync::atomic::AtomicU64;
mod atomic;
static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);
#[cfg(target_os = "macos")]
const DIRECTORY_FLAGS: i32 = 0x0010_0000 | 0x0000_0100 | 0x0100_0000;
#[cfg(target_os = "macos")]
const FILE_SAFETY_FLAGS: i32 = 0x0000_0100 | 0x0100_0000 | 0x0000_0004;
#[cfg(all(unix, not(target_os = "macos")))]
const DIRECTORY_FLAGS: i32 = 0o200000 | 0o400000 | 0o2000000;
#[cfg(all(unix, not(target_os = "macos")))]
const FILE_SAFETY_FLAGS: i32 = 0o400000 | 0o2000000 | 0o4000;
#[cfg(unix)]
unsafe extern "C" {
    fn openat(dirfd: i32, path: *const i8, flags: i32, mode: u32) -> i32;
    fn mkdirat(dirfd: i32, path: *const i8, mode: u32) -> i32;
    fn renameat(fromfd: i32, from: *const i8, tofd: i32, to: *const i8) -> i32;
    fn unlinkat(dirfd: i32, path: *const i8, flags: i32) -> i32;
}
pub fn create_private_dir_all(path: impl AsRef<Path>) -> Result<()> {
    let path = normalize_system_temp(path.as_ref());
    validate_private_dir(&path)?;
    #[cfg(unix)]
    {
        let dir = open_dir_chain(&path, true)?;
        dir.set_permissions(fs::Permissions::from_mode(0o700))?;
    }
    #[cfg(not(unix))]
    fs::create_dir_all(&path)?;
    Ok(())
}
pub fn write_private(path: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> Result<()> {
    write_private_atomic(path.as_ref(), contents.as_ref())
}

pub fn write_atomic_preserving_mode(
    path: impl AsRef<Path>,
    contents: impl AsRef<[u8]>,
) -> Result<()> {
    let path = normalize_system_temp(path.as_ref());
    validate_private_write_path(&path)?;
    let metadata = fs::symlink_metadata(&path)
        .with_context(|| format!("atomic destination must already exist: {}", path.display()))?;
    if !metadata.is_file() {
        bail!(
            "atomic destination must be a regular file: {}",
            path.display()
        );
    }
    atomic::replace(
        &path,
        contents.as_ref(),
        Some(metadata.permissions()),
        "source",
    )
}
pub fn write_private_new(path: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> Result<()> {
    let mut file = create_private_file(path)?;
    file.write_all(contents.as_ref())?;
    file.sync_all()?;
    Ok(())
}
pub fn append_private(path: impl AsRef<Path>, text: &str) -> Result<()> {
    let path = normalize_system_temp(path.as_ref());
    let mut file = open_private_write(&path, OpenKind::Append)?;
    file.write_all(text.as_bytes())?;
    file.sync_data()?;
    Ok(())
}

pub fn create_private_file(path: impl AsRef<Path>) -> Result<File> {
    let path = normalize_system_temp(path.as_ref());
    open_private_write(&path, OpenKind::Exclusive)
}

fn write_private_atomic(path: &Path, contents: &[u8]) -> Result<()> {
    let path = normalize_system_temp(path);
    validate_private_write_path(&path)?;
    let parent = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("private write path has no parent"))?;
    create_private_dir_all(parent)?;
    atomic::replace(&path, contents, None, "private")
}

#[derive(Clone, Copy)]
enum OpenKind {
    Append,
    Exclusive,
}

fn open_private_write(path: &Path, kind: OpenKind) -> Result<File> {
    validate_private_write_path(path)?;
    let parent = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("private write path has no parent"))?;
    create_private_dir_all(parent)?;
    #[cfg(unix)]
    {
        let dir = open_dir_chain(parent, false)?;
        open_at(&dir, &component_name(path)?, kind)
    }
    #[cfg(not(unix))]
    {
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create(true);
        match kind {
            OpenKind::Append => options.append(true),
            OpenKind::Exclusive => options.create_new(true),
        };
        Ok(options.open(path)?)
    }
}

#[cfg(unix)]
fn open_at(dir: &File, name: &CString, kind: OpenKind) -> Result<File> {
    const O_WRONLY: i32 = 1;
    #[cfg(target_os = "macos")]
    const O_CREAT: i32 = 0x0200;
    #[cfg(target_os = "macos")]
    const O_EXCL: i32 = 0x0800;
    #[cfg(target_os = "macos")]
    const O_APPEND: i32 = 0x0008;
    #[cfg(all(unix, not(target_os = "macos")))]
    const O_CREAT: i32 = 0o100;
    #[cfg(all(unix, not(target_os = "macos")))]
    const O_EXCL: i32 = 0o200;
    #[cfg(all(unix, not(target_os = "macos")))]
    const O_APPEND: i32 = 0o2000;
    let kind_flags = match kind {
        OpenKind::Append => O_APPEND,
        OpenKind::Exclusive => O_EXCL,
    };
    let fd = unsafe {
        openat(
            dir.as_raw_fd(),
            name.as_ptr(),
            O_WRONLY | O_CREAT | kind_flags | FILE_SAFETY_FLAGS,
            0o600,
        )
    };
    if fd < 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    let file = unsafe { File::from_raw_fd(fd) };
    if !file.metadata()?.is_file() {
        bail!("private write destination must be a regular file");
    }
    file.set_permissions(fs::Permissions::from_mode(0o600))?;
    Ok(file)
}

#[cfg(unix)]
fn open_dir_chain(path: &Path, create: bool) -> Result<File> {
    use std::path::Component;
    if !path.is_absolute() {
        bail!(
            "private directory path must be absolute: {}",
            path.display()
        );
    }
    let mut dir = File::open("/")?;
    for component in path.components() {
        let Component::Normal(part) = component else {
            continue;
        };
        let name = CString::new(part.as_bytes())?;
        match open_dir_at(dir.as_raw_fd(), &name) {
            Ok(next) => dir = next,
            Err(error) if create && error.raw_os_error() == Some(2) => {
                if unsafe { mkdirat(dir.as_raw_fd(), name.as_ptr(), 0o700) } != 0 {
                    let mkdir_error = std::io::Error::last_os_error();
                    if mkdir_error.raw_os_error() != Some(17) {
                        return Err(mkdir_error.into());
                    }
                }
                dir = open_dir_at(dir.as_raw_fd(), &name)?;
            }
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("unsafe directory component in {}", path.display()));
            }
        }
    }
    Ok(dir)
}

#[cfg(unix)]
fn open_dir_at(parent: RawFd, name: &CString) -> std::io::Result<File> {
    let fd = unsafe { openat(parent, name.as_ptr(), DIRECTORY_FLAGS, 0) };
    if fd < 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(unsafe { File::from_raw_fd(fd) })
    }
}

#[cfg(unix)]
fn component_name(path: &Path) -> Result<CString> {
    let name = path
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("private write path has no file name"))?;
    Ok(CString::new(name.as_bytes())?)
}

#[cfg(test)]
#[path = "private_fs_tests.rs"]
mod tests;
