//! Which profile this IVE runs as: every file it keeps (state, settings, scratches) lives in the profile's folder,
//! and its terminals in the profile's registry, so several IVEs never share them.

use std::path::PathBuf;
use std::sync::OnceLock;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Profile {
    name: String,
}

const MAX_NAME_LENGTH: usize = 64;

static CURRENT: OnceLock<Profile> = OnceLock::new();

impl Profile {
    pub const DEFAULT: &str = "default";

    /// A profile name: letters, digits, `-` and `_`, 1-64 characters, so it cannot leave its folder.
    pub fn parse(name: &str) -> Result<Self, String> {
        let allowed = |c: char| c.is_ascii_alphanumeric() || c == '-' || c == '_';
        if name.is_empty() || name.len() > MAX_NAME_LENGTH || !name.chars().all(allowed) {
            return Err(format!("profile name {name:?} must be 1-{MAX_NAME_LENGTH} letters, digits, - or _"));
        }
        Ok(Self { name: name.to_owned() })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// `<OS config>/ive/<name>`, e.g. `~/.config/ive/default` on Linux; `None` when the OS reports no config folder.
    /// On Windows the local AppData, not the roaming one: the state holds this machine's paths, and `mcp.toml` a token.
    pub fn config_dir(&self) -> Option<PathBuf> {
        dirs::config_local_dir().map(|dir| dir.join("ive").join(&self.name))
    }

    pub fn state_path(&self) -> Option<PathBuf> {
        self.config_dir().map(|dir| dir.join("state.toml"))
    }

    pub fn settings_path(&self) -> Option<PathBuf> {
        self.config_dir().map(|dir| dir.join("settings.toml"))
    }

    /// The MCP server's port and token, see [`crate::mcp::endpoint`].
    pub fn mcp_path(&self) -> Option<PathBuf> {
        self.config_dir().map(|dir| dir.join("mcp.toml"))
    }

    pub fn scratches_dir(&self) -> Option<PathBuf> {
        self.config_dir().map(|dir| dir.join("scratches"))
    }

    /// Images pasted into terminals, `~/.cache/ive/<name>/pastes` on Linux: a cache, IVE deletes old ones.
    pub fn pastes_dir(&self) -> Option<PathBuf> {
        dirs::cache_dir().map(|dir| dir.join("ive").join(&self.name).join("pastes"))
    }
}

impl Default for Profile {
    fn default() -> Self {
        Self { name: Self::DEFAULT.to_owned() }
    }
}

/// Sets the profile of this process; the first call wins.
pub fn init(profile: Profile) {
    let _ = CURRENT.set(profile);
}

/// The profile set by [`init`], or the default one before that (as in unit tests).
pub fn current() -> &'static Profile {
    CURRENT.get_or_init(Profile::default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_keep_pastes_in_the_profile_cache_folder() {
        // given
        let profile = Profile::parse("work").unwrap();

        // when
        let dir = profile.pastes_dir();

        // then
        assert!(dir.is_some_and(|dir| dir.ends_with("ive/work/pastes")));
    }

    #[test]
    fn should_accept_letters_digits_dash_and_underscore() {
        // given
        let name = "ive-dev_2";

        // when
        let profile = Profile::parse(name);

        // then
        assert_eq!(profile.map(|profile| profile.name().to_owned()), Ok(name.to_owned()));
    }

    #[test]
    fn should_reject_names_that_could_leave_the_folder() {
        // given
        let long = "a".repeat(65);
        let names = ["", "..", "a/b", "a b", long.as_str()];

        // when
        let errors: Vec<String> = names.iter().filter_map(|name| Profile::parse(name).err()).collect();

        // then
        assert_eq!(errors.len(), names.len());
        assert!(errors[2].contains("\"a/b\""), "{}", errors[2]);
    }

    #[test]
    fn should_keep_everything_under_the_profile_folder() {
        // given
        let profile = Profile::parse("work").unwrap();

        // when
        let paths = [profile.state_path(), profile.settings_path(), profile.scratches_dir(), profile.mcp_path()].map(Option::unwrap);

        // then
        assert!(paths[0].ends_with("ive/work/state.toml"), "{}", paths[0].display());
        assert!(paths[1].ends_with("ive/work/settings.toml"), "{}", paths[1].display());
        assert!(paths[2].ends_with("ive/work/scratches"), "{}", paths[2].display());
        assert!(paths[3].ends_with("ive/work/mcp.toml"), "{}", paths[3].display());
    }

    #[test]
    fn should_use_the_default_profile_until_one_is_set() {
        // given
        let profile = current();

        // when
        let name = profile.name();

        // then
        assert_eq!(name, Profile::DEFAULT);
    }
}
