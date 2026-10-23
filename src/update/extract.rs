//! Unpacking a release archive with the OS's own tools: `tar` for `.tar.gz`, `ditto` (macOS) or `tar` (Windows'
//! bsdtar) for `.zip`.

use std::path::Path;
use std::process::Command;

/// Unpacks `archive` into the folder `into`, creating it.
pub fn extract(archive: &Path, into: &Path) -> Result<(), String> {
    let name = archive.file_name().map(|name| name.to_string_lossy()).unwrap_or_default();
    let mut command = if name.ends_with(".tar.gz") {
        tar("-xzf", archive, into)
    } else if name.ends_with(".zip") {
        unzip(archive, into)
    } else {
        return Err(format!("cannot unpack {}: not a .tar.gz or .zip", archive.display()));
    };
    std::fs::create_dir_all(into).map_err(|error| format!("cannot create {}: {error}", into.display()))?;
    let output = command.output().map_err(|error| format!("cannot unpack {}: {error}", archive.display()))?;
    if output.status.success() {
        return Ok(());
    }
    Err(format!("cannot unpack {}: {} ({})", archive.display(), String::from_utf8_lossy(&output.stderr).trim(), output.status))
}

fn tar(mode: &str, archive: &Path, into: &Path) -> Command {
    let mut command = Command::new("tar");
    command.arg(mode).arg(archive).arg("-C").arg(into);
    command
}

/// `ditto` keeps an app bundle's symlinks, permissions and signature intact.
#[cfg(target_os = "macos")]
fn unzip(archive: &Path, into: &Path) -> Command {
    let mut command = Command::new("ditto");
    command.arg("-x").arg("-k").arg(archive).arg(into);
    command
}

/// Windows 10 and later ship bsdtar, which reads zip. (GNU tar does not, but Linux releases are `.tar.gz`.)
#[cfg(not(target_os = "macos"))]
fn unzip(archive: &Path, into: &Path) -> Command {
    tar("-xf", archive, into)
}

#[cfg(all(test, unix))]
mod tests {
    use std::process::Command;

    use super::*;

    #[test]
    fn should_extract_a_tarball() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let folder = dir.path().join("ive-linux-x86_64-abc");
        std::fs::create_dir(&folder).unwrap();
        std::fs::write(folder.join("ive"), "binary").unwrap();
        let archive = dir.path().join("ive-linux-x86_64-abc.tar.gz");
        let packed = Command::new("tar").arg("-C").arg(dir.path()).arg("-czf").arg(&archive).arg("ive-linux-x86_64-abc").status().unwrap();
        assert!(packed.success());
        let into = dir.path().join("out");

        // when
        let extracted = extract(&archive, &into);

        // then
        assert_eq!(extracted, Ok(()));
        assert_eq!(std::fs::read_to_string(into.join("ive-linux-x86_64-abc/ive")).unwrap(), "binary");
    }

    #[test]
    fn should_refuse_an_archive_of_unknown_kind() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let archive = dir.path().join("ive.rar");
        std::fs::write(&archive, "x").unwrap();

        // when
        let extracted = extract(&archive, &dir.path().join("out"));

        // then
        assert!(extracted.unwrap_err().contains("ive.rar"));
    }
}
