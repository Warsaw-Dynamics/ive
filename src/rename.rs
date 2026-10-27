//! Renames that never replace what is at the target.

#[cfg(unix)]
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

/// `MoveFileExW` without `MOVEFILE_REPLACE_EXISTING` fails when `target` exists, but onto another hard link of the
/// same file it succeeds and takes `source` away: such a target is refused first, unless it differs only in case.
#[cfg(windows)]
pub fn rename_new(source: &Path, target: &Path) -> io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    let case_only = source.as_os_str().eq_ignore_ascii_case(target.as_os_str());
    if !case_only && std::fs::symlink_metadata(target).is_ok() {
        return Err(io::ErrorKind::AlreadyExists.into());
    }
    let wide = |path: &Path| path.as_os_str().encode_wide().chain(Some(0)).collect::<Vec<u16>>();
    let (from, to) = (wide(source), wide(target));
    // SAFETY: both pointers are NUL-terminated paths that outlive the call
    if unsafe { windows_sys::Win32::Storage::FileSystem::MoveFileExW(from.as_ptr(), to.as_ptr(), 0) } != 0 {
        return Ok(());
    }
    Err(io::Error::last_os_error())
}

/// Checks, then renames: another program can still create `target` in between.
#[cfg(any(target_os = "linux", target_os = "macos"))]
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
