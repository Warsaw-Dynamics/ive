//! The latest GitHub release, as `releases/latest` describes it, and the asset of it this platform installs.

use serde::Deserialize;

use super::Platform;

/// The checksums asset the release workflow publishes beside the archives, in `sha256sum`'s format.
pub const SUMS_NAME: &str = "SHA256SUMS";

/// A release archive is about 30 MB; anything near this is not one.
pub const MAX_ASSET_BYTES: u64 = 512 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Release {
    #[serde(rename = "tag_name")]
    pub tag: String,
    pub assets: Vec<Asset>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Asset {
    pub name: String,
    #[serde(rename = "browser_download_url")]
    pub url: String,
    pub size: u64,
}

/// A release to install: its archive for this platform and the checksums to verify it with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Update {
    pub tag: String,
    pub archive: Asset,
    pub sums: Asset,
}

pub fn parse_release(json: &str) -> Result<Release, String> {
    serde_json::from_str(json).map_err(|error| format!("the latest release's description is not what GitHub sends: {error}"))
}

/// The update `release` offers a build tagged `own_tag` on `platform`: none when it is this build, or when it has
/// nothing for this platform. Any other tag counts as newer, so deleting a bad release rolls builds back.
pub fn find_update(release: &Release, own_tag: &str, platform: &Platform) -> Result<Option<Update>, String> {
    if release.tag == own_tag {
        return Ok(None);
    }
    let prefix = platform.asset_prefix();
    let Some(archive) = release.assets.iter().find(|asset| asset.name.starts_with(&prefix)) else {
        return Ok(None);
    };
    if archive.size > MAX_ASSET_BYTES {
        return Err(format!("release {} asset {} is {} bytes, over the {MAX_ASSET_BYTES} limit", release.tag, archive.name, archive.size));
    }
    let sums = release.assets.iter().find(|asset| asset.name == SUMS_NAME).ok_or_else(|| format!("release {} has no {SUMS_NAME} to verify {} with", release.tag, archive.name))?;
    Ok(Some(Update { tag: release.tag.clone(), archive: archive.clone(), sums: sums.clone() }))
}

#[cfg(test)]
mod tests {
    use super::*;

    const LINUX: Platform = Platform { os: "linux", arch: "x86_64" };

    fn release_json(tag: &str, archive_size: u64) -> String {
        format!(
            r#"{{
                "tag_name": "{tag}",
                "name": "{tag}",
                "assets": [
                    {{ "name": "ive-linux-x86_64-abc.tar.gz", "size": {archive_size}, "browser_download_url": "https://example.com/linux.tar.gz" }},
                    {{ "name": "ive-macos-arm64-abc.zip", "size": 20, "browser_download_url": "https://example.com/macos.zip" }},
                    {{ "name": "SHA256SUMS", "size": 200, "browser_download_url": "https://example.com/SHA256SUMS" }}
                ]
            }}"#
        )
    }

    #[test]
    fn should_parse_tag_and_assets_from_github_json() {
        // given
        let json = release_json("ive-0.1.0-abc", 10);

        // when
        let release = parse_release(&json).unwrap();

        // then
        assert_eq!(release.tag, "ive-0.1.0-abc");
        assert_eq!(release.assets.len(), 3);
        assert_eq!(release.assets[0], Asset { name: "ive-linux-x86_64-abc.tar.gz".to_owned(), url: "https://example.com/linux.tar.gz".to_owned(), size: 10 });
    }

    #[test]
    fn should_find_the_update_for_this_platform() {
        // given
        let release = parse_release(&release_json("ive-0.1.0-abc", 10)).unwrap();

        // when
        let update = find_update(&release, "ive-0.1.0-old", &LINUX).unwrap().unwrap();

        // then
        assert_eq!(update.tag, "ive-0.1.0-abc");
        assert_eq!(update.archive.name, "ive-linux-x86_64-abc.tar.gz");
        assert_eq!(update.sums.name, SUMS_NAME);
    }

    #[test]
    fn should_find_no_update_for_our_own_tag() {
        // given
        let release = parse_release(&release_json("ive-0.1.0-abc", 10)).unwrap();

        // when
        let update = find_update(&release, "ive-0.1.0-abc", &LINUX);

        // then
        assert_eq!(update, Ok(None));
    }

    #[test]
    fn should_find_no_update_without_an_asset_for_this_platform() {
        // given
        let release = parse_release(&release_json("ive-0.1.0-abc", 10)).unwrap();
        let windows = Platform { os: "windows", arch: "x86_64" };

        // when
        let update = find_update(&release, "ive-0.1.0-old", &windows);

        // then
        assert_eq!(update, Ok(None));
    }

    #[test]
    fn should_refuse_an_asset_larger_than_the_limit() {
        // given
        let release = parse_release(&release_json("ive-0.1.0-abc", MAX_ASSET_BYTES + 1)).unwrap();

        // when
        let update = find_update(&release, "ive-0.1.0-old", &LINUX);

        // then
        assert!(update.unwrap_err().contains("ive-linux-x86_64-abc.tar.gz"));
    }

    #[test]
    fn should_refuse_a_release_without_checksums() {
        // given
        let mut release = parse_release(&release_json("ive-0.1.0-abc", 10)).unwrap();
        release.assets.retain(|asset| asset.name != SUMS_NAME);

        // when
        let update = find_update(&release, "ive-0.1.0-old", &LINUX);

        // then
        assert!(update.unwrap_err().contains(SUMS_NAME));
    }

    #[test]
    fn should_name_the_asset_prefix_of_a_platform() {
        // given
        let platform = Platform { os: "macos", arch: "arm64" };

        // when
        let prefix = platform.asset_prefix();

        // then
        assert_eq!(prefix, "ive-macos-arm64-");
    }
}
