//! One update check from start to staged release, applying a staged release, and the state Settings shows.

use std::fs;
use std::path::{Path, PathBuf};

use super::Platform;
use super::checksum::{parse_sums, verify};
use super::extract::extract;
use super::fetch::{fetch_file, fetch_text};
use super::install::{Install, swap};
use super::release::{MAX_ASSET_BYTES, Update, find_update, parse_release};
use super::staging::{Lock, STAGED_NAME, Staged, others_running, staged_protocol};

/// The newest release, as GitHub names it.
pub const FEED: &str = "https://api.github.com/repos/Warsaw-Dynamics/ive/releases/latest";

const DOWNLOAD_DIR: &str = "download";
const EXTRACTED_DIR: &str = "extracted";

/// What an update check works with.
#[derive(Debug, Clone, Copy)]
pub struct Check<'a> {
    pub install: &'a Install,
    pub own_tag: &'a str,
    /// the URL of the latest release's description
    pub feed: &'a str,
    pub platform: Platform,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckOutcome {
    UpToDate,
    Staged(Staged),
    /// another IVE on this install is staging or swapping
    Busy,
}

/// Finds the latest release and, when it is not this build, downloads, verifies and stages it beside the install.
/// Blocks for as long as the download takes.
pub fn check_and_stage(check: &Check) -> Result<CheckOutcome, String> {
    let dir = check.install.staging_dir();
    fs::create_dir_all(&dir).map_err(|error| format!("cannot create {}: {error}", dir.display()))?;
    let Some(_lock) = Lock::take(&dir)? else {
        return Ok(CheckOutcome::Busy);
    };
    let release = parse_release(&fetch_text(check.feed)?)?;
    let Some(update) = find_update(&release, check.own_tag, &check.platform)? else {
        // a staged release the feed no longer names was withdrawn
        discard(&dir);
        return Ok(CheckOutcome::UpToDate);
    };
    if let Some(staged) = Staged::load(&dir).filter(|staged| staged.tag == update.tag) {
        return Ok(CheckOutcome::Staged(staged));
    }
    discard(&dir);
    let staged = stage(&update, check.install, &dir);
    let _ = fs::remove_dir_all(dir.join(DOWNLOAD_DIR));
    if staged.is_err() {
        discard(&dir);
    }
    staged.map(CheckOutcome::Staged)
}

fn stage(update: &Update, install: &Install, dir: &Path) -> Result<Staged, String> {
    let downloads = dir.join(DOWNLOAD_DIR);
    fs::create_dir_all(&downloads).map_err(|error| format!("cannot create {}: {error}", downloads.display()))?;
    let archive = downloads.join(&update.archive.name);
    fetch_file(&update.archive.url, &archive, MAX_ASSET_BYTES)?;
    verify(&archive, &update.archive.name, &parse_sums(&fetch_text(&update.sums.url)?))?;
    let extracted = dir.join(EXTRACTED_DIR);
    extract(&archive, &extracted)?;
    let path = super::staging::find_staged_install(&extracted, install)?;
    let staged = Staged { tag: update.tag.clone(), protocol: staged_protocol(&executable_of(install, &path)), path };
    staged.save(dir)?;
    Ok(staged)
}

/// The executable of a staged release at `path`: itself, or the one inside a `.app`.
fn executable_of(install: &Install, path: &Path) -> PathBuf {
    match install {
        Install::Binary(_) => path.to_path_buf(),
        Install::App(_) => path.join("Contents/MacOS/ive"),
    }
}

/// Removes a staged release and its downloads from staging folder `dir`, keeping the lock and the running list.
pub fn discard(dir: &Path) {
    let _ = fs::remove_dir_all(dir.join(DOWNLOAD_DIR));
    let _ = fs::remove_dir_all(dir.join(EXTRACTED_DIR));
    let _ = fs::remove_file(dir.join(STAGED_NAME));
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Applied {
    /// the install is now this release
    Swapped(String),
    NothingStaged,
    /// not now, and why
    NotNow(&'static str),
}

/// Swaps the staged release in for `install`, unless another IVE runs from it or is staging.
pub fn apply_staged(install: &Install) -> Result<Applied, String> {
    let dir = install.staging_dir();
    let Some(staged) = Staged::load(&dir) else {
        return Ok(Applied::NothingStaged);
    };
    let Some(_lock) = Lock::take(&dir)? else {
        return Ok(Applied::NotNow("another IVE is updating this install"));
    };
    if others_running(&dir) {
        return Ok(Applied::NotNow("another IVE runs from this install"));
    }
    swap(install, &staged.path)?;
    discard(&dir);
    Ok(Applied::Swapped(staged.tag))
}

/// What the Settings tab says about updates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateState {
    Off,
    NotChecked,
    Checking,
    UpToDate,
    Ready(Staged),
    Failed(String),
}

impl UpdateState {
    pub fn describe(&self) -> String {
        match self {
            Self::Off => "Automatic updates are off".to_owned(),
            Self::NotChecked => "Not checked yet".to_owned(),
            Self::Checking => "Checking for updates…".to_owned(),
            Self::UpToDate => "Up to date".to_owned(),
            Self::Ready(staged) => format!("{} is ready: IVE switches to it when it quits or after a while idle", staged.tag),
            Self::Failed(reason) => format!("Last check failed: {reason}"),
        }
    }
}

#[cfg(all(test, unix))]
mod tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::Path;
    use std::process::Command;

    use super::*;

    const LINUX: Platform = Platform { os: "linux", arch: "x86_64" };

    struct Fixture {
        _dir: tempfile::TempDir,
        install: Install,
        feed: String,
        releases: PathBuf,
    }

    fn write(path: &Path, text: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    /// An installed `ive` and a release `ive-0.1.0-new` of a new one, served from files.
    fn fixture() -> Fixture {
        let dir = tempfile::tempdir().unwrap();
        let install = Install::Binary(dir.path().join("install/ive"));
        write(install.path(), "old");
        let releases = dir.path().join("releases");
        let binary = releases.join("ive-linux-x86_64-new/ive");
        write(&binary, "#!/bin/sh\necho 1\n");
        fs::set_permissions(&binary, fs::Permissions::from_mode(0o755)).unwrap();
        let archive = releases.join("ive-linux-x86_64-new.tar.gz");
        let packed = Command::new("tar").arg("-C").arg(&releases).arg("-czf").arg(&archive).arg("ive-linux-x86_64-new").status().unwrap();
        assert!(packed.success());
        let sums = Command::new("sha256sum").arg("ive-linux-x86_64-new.tar.gz").current_dir(&releases).output().unwrap();
        fs::write(releases.join("SHA256SUMS"), sums.stdout).unwrap();
        let size = fs::metadata(&archive).unwrap().len();
        let json = format!(
            r#"{{ "tag_name": "ive-0.1.0-new", "assets": [
                {{ "name": "ive-linux-x86_64-new.tar.gz", "size": {size}, "browser_download_url": "file://{}" }},
                {{ "name": "SHA256SUMS", "size": 100, "browser_download_url": "file://{}" }} ] }}"#,
            archive.display(),
            releases.join("SHA256SUMS").display()
        );
        let feed_file = releases.join("latest.json");
        fs::write(&feed_file, json).unwrap();
        Fixture { feed: format!("file://{}", feed_file.display()), install, releases, _dir: dir }
    }

    fn check(fixture: &Fixture) -> Result<CheckOutcome, String> {
        check_and_stage(&Check { install: &fixture.install, own_tag: "ive-0.1.0-old", feed: &fixture.feed, platform: LINUX })
    }

    #[test]
    fn should_stage_a_new_release_from_a_file_feed() {
        // given
        let fixture = fixture();

        // when
        let outcome = check(&fixture).unwrap();

        // then
        let CheckOutcome::Staged(staged) = outcome else {
            panic!("expected a staged release, got {outcome:?}");
        };
        assert_eq!(staged.tag, "ive-0.1.0-new");
        assert_eq!(staged.protocol, Some(1));
        assert!(staged.path.starts_with(fixture.install.staging_dir()));
        assert_eq!(Staged::load(&fixture.install.staging_dir()), Some(staged));
    }

    #[test]
    fn should_not_download_again_when_already_staged() {
        // given
        let fixture = fixture();
        check(&fixture).unwrap();
        fs::remove_file(fixture.releases.join("ive-linux-x86_64-new.tar.gz")).unwrap();

        // when
        let outcome = check(&fixture).unwrap();

        // then
        assert!(matches!(outcome, CheckOutcome::Staged(_)), "{outcome:?}");
    }

    #[test]
    fn should_discard_a_bad_download() {
        // given
        let fixture = fixture();
        fs::write(fixture.releases.join("SHA256SUMS"), format!("{}  ive-linux-x86_64-new.tar.gz\n", "0".repeat(64))).unwrap();

        // when
        let outcome = check(&fixture);

        // then
        assert!(outcome.unwrap_err().contains("SHA-256"));
        assert_eq!(Staged::load(&fixture.install.staging_dir()), None);
        assert!(!fixture.install.staging_dir().join(DOWNLOAD_DIR).exists());
    }

    #[test]
    fn should_be_up_to_date_on_the_latest_tag() {
        // given
        let fixture = fixture();

        // when
        let outcome = check_and_stage(&Check { install: &fixture.install, own_tag: "ive-0.1.0-new", feed: &fixture.feed, platform: LINUX });

        // then
        assert_eq!(outcome, Ok(CheckOutcome::UpToDate));
    }

    #[test]
    fn should_apply_a_staged_binary() {
        // given
        let fixture = fixture();
        check(&fixture).unwrap();

        // when
        let applied = apply_staged(&fixture.install);

        // then
        assert_eq!(applied, Ok(Applied::Swapped("ive-0.1.0-new".to_owned())));
        assert_eq!(fs::read_to_string(fixture.install.path()).unwrap(), "#!/bin/sh\necho 1\n");
        assert_eq!(Staged::load(&fixture.install.staging_dir()), None);
    }

    #[test]
    fn should_not_apply_while_another_ive_runs() {
        // given
        let fixture = fixture();
        check(&fixture).unwrap();
        let mut other = Command::new("sleep").arg("5").spawn().unwrap();
        write(&fixture.install.staging_dir().join("running").join(other.id().to_string()), "");

        // when
        let applied = apply_staged(&fixture.install);

        // then
        other.kill().unwrap();
        other.wait().unwrap();
        assert_eq!(applied, Ok(Applied::NotNow("another IVE runs from this install")));
        assert_eq!(fs::read_to_string(fixture.install.path()).unwrap(), "old");
    }

    #[test]
    fn should_discard_what_is_staged() {
        // given
        let fixture = fixture();
        check(&fixture).unwrap();

        // when
        discard(&fixture.install.staging_dir());

        // then
        assert_eq!(Staged::load(&fixture.install.staging_dir()), None);
        assert!(!fixture.install.staging_dir().join(EXTRACTED_DIR).exists());
    }
}
