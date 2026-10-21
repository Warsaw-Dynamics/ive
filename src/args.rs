use std::ffi::OsString;
use std::fmt;
use std::path::PathBuf;

use crate::profile::Profile;

/// Ports below this need privileges on Unix; IVE never wants one.
pub const MIN_PORT: u16 = 1024;

pub const USAGE: &str = "usage: ive [--profile NAME] [--restart SCRIPT] [--mcp PORT | --no-mcp] [--background] [FOLDER | FILE]";

/// Command line: `ive [--profile NAME] [--restart SCRIPT] [--mcp PORT | --no-mcp] [--background] [FOLDER | FILE]`.
#[derive(Debug, PartialEq, Eq)]
pub struct Args {
    pub path: Option<PathBuf>,
    pub mcp: McpPort,
    /// open the window without taking focus
    pub background: bool,
    pub profile: Profile,
    /// the start script the Restart button runs
    pub restart: Option<PathBuf>,
}

/// Where IVE serves MCP on 127.0.0.1.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum McpPort {
    /// the port remembered in the profile's `mcp.toml`, picked on the first run
    #[default]
    Remembered,
    /// `--mcp PORT`, remembered from then on
    Given(u16),
    /// `--no-mcp`
    Off,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ArgsError {
    MissingPort,
    McpAndNoMcp,
    InvalidPort(String),
    UnknownFlag(String),
    ExtraPath(PathBuf),
    MissingProfile,
    MissingRestartScript,
    InvalidProfile(String),
}

impl fmt::Display for ArgsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingPort => write!(f, "--mcp needs a port"),
            Self::InvalidPort(value) => write!(f, "--mcp port must be {MIN_PORT}-65535, got {value:?}"),
            Self::McpAndNoMcp => write!(f, "--mcp and --no-mcp cannot be used together"),
            Self::UnknownFlag(flag) => write!(f, "unknown option {flag:?}"),
            Self::MissingProfile => write!(f, "--profile needs a name"),
            Self::MissingRestartScript => write!(f, "--restart needs the path of a start script"),
            Self::InvalidProfile(reason) => write!(f, "{reason}"),
            Self::ExtraPath(path) => write!(f, "only one folder or file can be opened, got also {}", path.display()),
        }
    }
}

impl Args {
    pub fn parse(arguments: impl IntoIterator<Item = OsString>) -> Result<Self, ArgsError> {
        let mut args = Self { path: None, mcp: McpPort::Remembered, background: false, profile: Profile::default(), restart: None };
        let mut arguments = arguments.into_iter();
        while let Some(argument) = arguments.next() {
            match argument.to_str() {
                Some("--mcp") => args.set_mcp(McpPort::Given(parse_port(arguments.next())?))?,
                Some("--no-mcp") => args.set_mcp(McpPort::Off)?,
                Some("--background") => args.background = true,
                Some("--restart") => args.restart = Some(arguments.next().map(PathBuf::from).ok_or(ArgsError::MissingRestartScript)?),
                Some(flag) if flag.starts_with("--restart=") => args.restart = Some(PathBuf::from(&flag["--restart=".len()..])),
                Some("--profile") => args.profile = parse_profile(arguments.next())?,
                Some(flag) if flag.starts_with("--profile=") => args.profile = parse_profile(Some(flag["--profile=".len()..].into()))?,
                Some(flag) if flag.starts_with("--") => return Err(ArgsError::UnknownFlag(flag.to_owned())),
                _ => args.set_path(PathBuf::from(argument))?,
            }
        }
        Ok(args)
    }

    fn set_mcp(&mut self, mcp: McpPort) -> Result<(), ArgsError> {
        let conflicting = matches!((self.mcp, mcp), (McpPort::Off, McpPort::Given(_)) | (McpPort::Given(_), McpPort::Off));
        if conflicting {
            return Err(ArgsError::McpAndNoMcp);
        }
        self.mcp = mcp;
        Ok(())
    }

    fn set_path(&mut self, path: PathBuf) -> Result<(), ArgsError> {
        if self.path.is_some() {
            return Err(ArgsError::ExtraPath(path));
        }
        self.path = Some(path);
        Ok(())
    }
}

/// The Restart button runs this script, so it must be a file IVE may execute.
#[cfg(unix)]
pub fn check_restart_script(path: &std::path::Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    let metadata = std::fs::metadata(path).map_err(|error| format!("--restart script {} cannot be used: {error}", path.display()))?;
    if metadata.is_file() && metadata.permissions().mode() & 0o111 != 0 {
        return Ok(());
    }
    Err(format!("--restart script {} cannot be used: not executable", path.display()))
}

fn parse_profile(value: Option<OsString>) -> Result<Profile, ArgsError> {
    let value = value.ok_or(ArgsError::MissingProfile)?;
    Profile::parse(&value.to_string_lossy()).map_err(ArgsError::InvalidProfile)
}

fn parse_port(value: Option<OsString>) -> Result<u16, ArgsError> {
    let value = value.ok_or(ArgsError::MissingPort)?;
    let text = value.to_string_lossy().into_owned();
    match text.parse::<u16>() {
        Ok(port) if port >= MIN_PORT => Ok(port),
        _ => Err(ArgsError::InvalidPort(text)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(line: &str) -> Result<Args, ArgsError> {
        Args::parse(line.split_whitespace().map(OsString::from))
    }

    #[test]
    fn should_read_flags_in_any_order_around_path() {
        // given
        let line = "--background ~/code --mcp 7461";

        // when
        let args = parse(line);

        // then
        assert_eq!(args, Ok(Args { path: Some(PathBuf::from("~/code")), mcp: McpPort::Given(7461), background: true, profile: Profile::default(), restart: None }));
    }

    #[test]
    fn should_default_to_welcome_screen_with_remembered_mcp_port() {
        // given
        let line = "";

        // when
        let args = parse(line);

        // then
        assert_eq!(args, Ok(Args { path: None, mcp: McpPort::Remembered, background: false, profile: Profile::default(), restart: None }));
    }

    #[test]
    fn should_reject_bad_ports_unknown_flags_and_second_path() {
        // given
        let lines = ["--mcp", "--mcp 80", "--mcp http", "--verbose", "a b"];

        // when
        let errors: Vec<_> = lines.iter().map(|line| parse(line).err()).collect();

        // then
        assert_eq!(
            errors,
            [
                Some(ArgsError::MissingPort),
                Some(ArgsError::InvalidPort("80".to_owned())),
                Some(ArgsError::InvalidPort("http".to_owned())),
                Some(ArgsError::UnknownFlag("--verbose".to_owned())),
                Some(ArgsError::ExtraPath(PathBuf::from("b"))),
            ]
        );
    }

    #[test]
    fn should_turn_mcp_off_with_no_mcp() {
        // given
        let line = "--no-mcp ~/code";

        // when
        let args = parse(line);

        // then
        assert_eq!(args.map(|args| args.mcp), Ok(McpPort::Off));
    }

    #[test]
    fn should_reject_mcp_with_no_mcp_in_either_order() {
        // given
        let lines = ["--mcp 7461 --no-mcp", "--no-mcp --mcp 7461"];

        // when
        let errors: Vec<_> = lines.iter().map(|line| parse(line).err()).collect();

        // then
        assert_eq!(errors, [Some(ArgsError::McpAndNoMcp), Some(ArgsError::McpAndNoMcp)]);
    }

    #[test]
    fn should_parse_profile_in_both_forms() {
        // given
        let lines = ["--profile work", "--profile=work"];

        // when
        let profiles: Vec<_> = lines.iter().map(|line| parse(line).map(|args| args.profile.name().to_owned())).collect();

        // then
        assert_eq!(profiles, [Ok("work".to_owned()), Ok("work".to_owned())]);
    }

    #[test]
    fn should_default_to_the_default_profile() {
        // given
        let line = "~/code";

        // when
        let args = parse(line);

        // then
        assert_eq!(args.map(|args| args.profile.name().to_owned()), Ok(Profile::DEFAULT.to_owned()));
    }

    #[test]
    fn should_reject_an_invalid_profile() {
        // given
        let lines = ["--profile ../x", "--profile"];

        // when
        let errors: Vec<_> = lines.iter().map(|line| parse(line).err()).collect();

        // then
        assert!(matches!(&errors[0], Some(ArgsError::InvalidProfile(reason)) if reason.contains("../x")), "{errors:?}");
        assert_eq!(errors[1], Some(ArgsError::MissingProfile));
    }

    #[test]
    fn should_parse_restart_in_both_forms() {
        // given
        let lines = ["--restart /home/me/ive.sh", "--restart=/home/me/ive.sh", ""];

        // when
        let scripts: Vec<_> = lines.iter().map(|line| parse(line).map(|args| args.restart)).collect();

        // then
        let script = Some(PathBuf::from("/home/me/ive.sh"));
        assert_eq!(scripts, [Ok(script.clone()), Ok(script), Ok(None)]);
    }

    #[test]
    fn should_reject_restart_without_a_script() {
        // given
        let line = "--restart";

        // when
        let args = parse(line);

        // then
        assert_eq!(args, Err(ArgsError::MissingRestartScript));
    }

    #[cfg(unix)]
    #[test]
    fn should_accept_only_an_executable_restart_script() {
        // given
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let (runnable, plain) = (dir.path().join("run.sh"), dir.path().join("notes.txt"));
        std::fs::write(&runnable, "#!/bin/sh\n").unwrap();
        std::fs::set_permissions(&runnable, std::fs::Permissions::from_mode(0o755)).unwrap();
        std::fs::write(&plain, "").unwrap();

        // when
        let checked = [check_restart_script(&runnable), check_restart_script(&plain), check_restart_script(&dir.path().join("gone.sh"))];

        // then
        assert_eq!(checked[0], Ok(()));
        assert!(checked[1].as_ref().is_err_and(|error| error.contains("notes.txt") && error.contains("not executable")), "{checked:?}");
        assert!(checked[2].as_ref().is_err_and(|error| error.contains("gone.sh")), "{checked:?}");
    }
}
