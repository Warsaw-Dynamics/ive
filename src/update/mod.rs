//! Updating a release build of IVE from GitHub Releases: find the newer release, stage it beside the install, swap it
//! in on quit or before a restart. Builds from source (no `IVE_RELEASE_TAG` at build time) never update.

pub mod release;

/// The release this build is, `ive-<version>-<commit>`; set by the release workflow, absent in builds from source.
pub const RELEASE_TAG: Option<&str> = option_env!("IVE_RELEASE_TAG");

/// An OS and CPU as release assets name them: `ive-<os>-<arch>-<commit>.<ext>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Platform {
    pub os: &'static str,
    pub arch: &'static str,
}

impl Platform {
    /// This build's platform; `None` where no release is built.
    pub fn current() -> Option<Self> {
        let os = match std::env::consts::OS {
            "linux" => "linux",
            "macos" => "macos",
            "windows" => "windows",
            _ => return None,
        };
        let arch = match std::env::consts::ARCH {
            "x86_64" => "x86_64",
            "aarch64" => "arm64",
            _ => return None,
        };
        Some(Self { os, arch })
    }

    pub fn asset_prefix(&self) -> String {
        format!("ive-{}-{}-", self.os, self.arch)
    }
}
