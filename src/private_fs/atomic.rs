use super::{OpenKind, TEMP_SEQUENCE};
use anyhow::Result;
use std::fs::Permissions;
use std::io::Write;
use std::path::Path;
use std::sync::atomic::Ordering;

pub(super) fn replace(
    path: &Path,
    contents: &[u8],
    permissions: Option<Permissions>,
    label: &str,
) -> Result<()> {
    #[cfg(unix)]
    {
        use super::{component_name, open_at, open_dir_chain, renameat, unlinkat};
        use std::ffi::CString;
        use std::os::fd::AsRawFd;

        let parent = path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("atomic write path has no parent"))?;
        let dir = open_dir_chain(parent, false)?;
        let name = component_name(path)?;
        let temp = CString::new(format!(
            ".nrr-{label}-tmp-{}-{}",
            std::process::id(),
            TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ))?;
        let mut file = open_at(&dir, &temp, OpenKind::Exclusive)?;
        let result = (|| {
            file.write_all(contents)?;
            if let Some(mode) = permissions {
                file.set_permissions(mode)?;
            }
            file.sync_all()?;
            if unsafe {
                renameat(
                    dir.as_raw_fd(),
                    temp.as_ptr(),
                    dir.as_raw_fd(),
                    name.as_ptr(),
                )
            } != 0
            {
                return Err(std::io::Error::last_os_error().into());
            }
            dir.sync_all()?;
            Ok(())
        })();
        if result.is_err() {
            unsafe { unlinkat(dir.as_raw_fd(), temp.as_ptr(), 0) };
        }
        result
    }
    #[cfg(not(unix))]
    {
        std::fs::write(path, contents)?;
        if let Some(mode) = permissions {
            std::fs::set_permissions(path, mode)?;
        }
        Ok(())
    }
}
