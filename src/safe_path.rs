use anyhow::{Context, Result, bail};
use std::fs::{self, File, Metadata};
use std::io::Read;
use std::path::{Component, Path, PathBuf};

pub const PRIVATE_RUNTIME_ROOT: &str = "/private/tmp/narrated-record-replay";

pub fn validate_cli_path(flag: &str, path: &Path) -> Result<()> {
    let normalized = normalize_system_temp(path);
    let path = normalized.as_path();
    if !path.is_absolute() {
        bail!("{flag} must be an absolute path");
    }
    reject_parent_components(flag, path)?;
    reject_existing_symlink_components(flag, path)
}

pub fn validate_private_runtime_path(flag: &str, path: &Path, custom_allowed: bool) -> Result<()> {
    validate_cli_path(flag, path)?;
    let normalized = normalize_system_temp(path);
    if !normalized.starts_with(PRIVATE_RUNTIME_ROOT) && !custom_allowed {
        bail!(
            "{flag} must stay under {PRIVATE_RUNTIME_ROOT} unless --i-consent-to-custom-runtime-paths is present: {}",
            normalized.display()
        );
    }
    Ok(())
}

pub fn validate_private_dir(path: &Path) -> Result<()> {
    validate_cli_path("private directory path", path)?;
    if let Ok(metadata) = fs::symlink_metadata(path)
        && !metadata.is_dir()
    {
        bail!(
            "private directory path exists but is not a directory: {}",
            path.display()
        );
    }
    Ok(())
}

pub fn validate_private_write_path(path: &Path) -> Result<()> {
    validate_cli_path("private write path", path)?;
    if let Ok(metadata) = fs::symlink_metadata(path) {
        if metadata.file_type().is_symlink() {
            bail!(
                "private write path must not be a symlink: {}",
                path.display()
            );
        }
        if metadata.is_dir() {
            bail!("private write path is a directory: {}", path.display());
        }
    }
    Ok(())
}

pub fn regular_file_metadata(path: &Path) -> Result<Metadata> {
    Ok(open_regular_file(path)?.metadata()?)
}

pub fn open_regular_file(path: &Path) -> Result<File> {
    let normalized = normalize_system_temp(path);
    let path = normalized.as_path();
    validate_cli_path("artifact path", path)?;
    let before = fs::symlink_metadata(path)
        .with_context(|| format!("artifact path does not exist: {}", path.display()))?;
    if before.file_type().is_symlink() {
        bail!("artifact path must not be a symlink: {}", path.display());
    }
    if !before.is_file() {
        bail!("artifact path must be a regular file: {}", path.display());
    }
    let file = File::open(path)
        .with_context(|| format!("artifact path could not be opened: {}", path.display()))?;
    let opened = file.metadata()?;
    let after = fs::symlink_metadata(path)
        .with_context(|| format!("artifact path changed while opening: {}", path.display()))?;
    if !same_file(&before, &opened) || !same_file(&opened, &after) {
        bail!("artifact path changed while opening: {}", path.display());
    }
    Ok(file)
}

pub fn read_regular_file_bounded(path: &Path, max_bytes: u64) -> Result<Vec<u8>> {
    let mut file = open_regular_file(path)?;
    let len = file.metadata()?.len();
    if len > max_bytes {
        bail!(
            "artifact exceeds byte limit: {} bytes, max {max_bytes}: {}",
            len,
            path.display()
        );
    }
    let mut bytes = Vec::with_capacity(len as usize);
    file.by_ref().take(max_bytes + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > max_bytes {
        bail!(
            "artifact grew beyond byte limit while reading: {}",
            path.display()
        );
    }
    Ok(bytes)
}

pub fn read_regular_text_bounded(path: &Path, max_bytes: u64) -> Result<String> {
    String::from_utf8(read_regular_file_bounded(path, max_bytes)?)
        .with_context(|| format!("artifact is not valid UTF-8: {}", path.display()))
}

#[cfg(unix)]
fn same_file(left: &Metadata, right: &Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;
    left.dev() == right.dev() && left.ino() == right.ino()
}

#[cfg(not(unix))]
fn same_file(left: &Metadata, right: &Metadata) -> bool {
    left.is_file() && right.is_file() && left.len() == right.len()
}

pub fn normalize_system_temp(path: &Path) -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        if let Ok(stripped) = path.strip_prefix("/tmp") {
            return Path::new("/private/tmp").join(stripped);
        }
        if let Ok(stripped) = path.strip_prefix("/var") {
            return Path::new("/private/var").join(stripped);
        }
    }
    path.to_path_buf()
}

fn reject_parent_components(flag: &str, path: &Path) -> Result<()> {
    for component in path.components() {
        if matches!(component, Component::ParentDir) {
            bail!(
                "{flag} must not contain parent directory components: {}",
                path.display()
            );
        }
    }
    Ok(())
}

fn reject_existing_symlink_components(flag: &str, path: &Path) -> Result<()> {
    let mut current = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => current.push(prefix.as_os_str()),
            Component::RootDir => current.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                bail!(
                    "{flag} must not contain parent directory components: {}",
                    path.display()
                )
            }
            Component::Normal(part) => current.push(part),
        }
        if current.as_os_str().is_empty() {
            continue;
        }
        if let Ok(metadata) = fs::symlink_metadata(&current)
            && metadata.file_type().is_symlink()
        {
            bail!(
                "{flag} must not contain symlink components: {} contains {}",
                path.display(),
                current.display()
            );
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "safe_path_extra_tests.rs"]
mod safe_path_extra_tests;
