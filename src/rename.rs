//! Renames that never replace what is at the target.

use std::fs;
use std::io;
use std::path::Path;

/// rename(2) that fails with `AlreadyExists` instead of replacing `target`, in one step where the OS offers it.
#[cfg(target_os = "linux")]
pub fn rename_new(source: &Path, target: &Path) -> io::Result<()> {
    let (from, to) = (c_path(source)?, c_path(target)?);
    // SAFETY: both pointers are NUL-terminated paths that outlive the call
    let result = unsafe { libc::renameat2(libc::AT_FDCWD, from.as_ptr(), libc::AT_FDCWD, to.as_ptr(), libc::RENAME_NOREPLACE) };
    if result == 0 {
        return Ok(());
    }
    let error = io::Error::last_os_error();
    match error.raw_os_error() {
        // a file system without RENAME_NOREPLACE
        Some(libc::EINVAL | libc::ENOSYS) => checked_rename(source, target),
        _ => Err(error),
    }
}

#[cfg(target_os = "macos")]
pub fn rename_new(source: &Path, target: &Path) -> io::Result<()> {
    let (from, to) = (c_path(source)?, c_path(target)?);
    // SAFETY: both pointers are NUL-terminated paths that outlive the call
    let result = unsafe { libc::renamex_np(from.as_ptr(), to.as_ptr(), libc::RENAME_EXCL) };
    if result == 0 {
        return Ok(());
    }
    let error = io::Error::last_os_error();
    match error.raw_os_error() {
        // a file system without RENAME_EXCL
        Some(libc::ENOTSUP) => checked_rename(source, target),
        _ => Err(error),
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub fn rename_new(source: &Path, target: &Path) -> io::Result<()> {
    checked_rename(source, target)
}

/// Checks, then renames: another program can still create `target` in between.
fn checked_rename(source: &Path, target: &Path) -> io::Result<()> {
    if fs::symlink_metadata(target).is_ok() {
        return Err(io::ErrorKind::AlreadyExists.into());
    }
    fs::rename(source, target)
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn c_path(path: &Path) -> io::Result<std::ffi::CString> {
    use std::os::unix::ffi::OsStrExt;
    std::ffi::CString::new(path.as_os_str().as_bytes())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, format!("{} holds a NUL byte", path.display())))
}
