use std::ffi::OsString;
use std::fmt;
use std::path::PathBuf;

use crate::profile::Profile;

/// Ports below this need privileges on Unix; IVE never wants one.
pub const MIN_PORT: u16 = 1024;

pub const USAGE: &str = "usage: ive [--profile NAME] [--restart SCRIPT] [--mcp PORT | --no-mcp] [--background] [--stop-processes-on-quit] [FOLDER | FILE]";

/// Command line: `ive [--profile NAME] [--restart SCRIPT] [--mcp PORT | --no-mcp] [--background] [--stop-processes-on-quit] [FOLDER | FILE]`.
#[derive(Debug, PartialEq, Eq)]
pub struct Args {
    pub path: Option<PathBuf>,
    pub mcp: McpPort,
    /// open the window without taking focus
    pub background: bool,
    /// quit without the prompt, stopping the running processes
    pub stop_processes_on_quit: bool,
    pub profile: Profile,
    /// the start script the Restart button runs
    pub restart: Option<PathBuf>,
    /// `--replacing PID`, not in the usage: an update restart started this IVE in place of IVE `PID`
    pub replacing: Option<u32>,
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
    InvalidReplacing(String),
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
            Self::InvalidReplacing(value) => write!(f, "--replacing needs the process id of an IVE, got {value:?}"),
            Self::ExtraPath(path) => write!(f, "only one folder or file can be opened, got also {}", path.display()),
        }
    }
}

impl Args {
    pub fn parse(arguments: impl IntoIterator<Item = OsString>) -> Result<Self, ArgsError> {
        let mut args = Self { path: None, mcp: McpPort::Remembered, background: false, stop_processes_on_quit: false, profile: Profile::default(), restart: None, replacing: None };
        let mut arguments = arguments.into_iter();
        while let Some(argument) = arguments.next() {
            match argument.to_str() {
                Some("--mcp") => args.set_mcp(McpPort::Given(parse_port(arguments.next())?))?,
                Some("--no-mcp") => args.set_mcp(McpPort::Off)?,
                Some("--background") => args.background = true,
                Some("--stop-processes-on-quit") => args.stop_processes_on_quit = true,
                Some("--restart") => args.restart = Some(arguments.next().map(PathBuf::from).ok_or(ArgsError::MissingRestartScript)?),
                Some(flag) if flag.starts_with("--restart=") => args.restart = Some(PathBuf::from(&flag["--restart=".len()..])),
                Some("--replacing") => args.replacing = Some(parse_pid(arguments.next())?),
                Some(flag) if flag.starts_with("--replacing=") => args.replacing = Some(parse_pid(Some(flag["--replacing=".len()..].into()))?),
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

/// The Restart button runs this script through PowerShell, so it must be a `.ps1` file.
#[cfg(windows)]
pub fn check_restart_script(path: &std::path::Path) -> Result<(), String> {
    let is_powershell = path.extension().is_some_and(|extension| extension.eq_ignore_ascii_case("ps1"));
    if is_powershell && path.is_file() {
        return Ok(());
    }
    Err(format!("--restart script {} cannot be used: not an existing .ps1 file", path.display()))
}

fn parse_profile(value: Option<OsString>) -> Result<Profile, ArgsError> {
    let value = value.ok_or(ArgsError::MissingProfile)?;
    Profile::parse(&value.to_string_lossy()).map_err(ArgsError::InvalidProfile)
}

fn parse_pid(value: Option<OsString>) -> Result<u32, ArgsError> {
    let text = value.map(|value| value.to_string_lossy().into_owned()).unwrap_or_default();
    match text.parse::<u32>() {
        Ok(pid) if pid > 0 => Ok(pid),
        _ => Err(ArgsError::InvalidReplacing(text)),
    }
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
        assert_eq!(args, Ok(Args { path: Some(PathBuf::from("~/code")), mcp: McpPort::Given(7461), background: true, stop_processes_on_quit: false, profile: Profile::default(), restart: None, replacing: None }));
    }

    #[test]
    fn should_default_to_welcome_screen_with_remembered_mcp_port() {
        // given
        let line = "";

        // when
        let args = parse(line);

        // then
        assert_eq!(args, Ok(Args { path: None, mcp: McpPort::Remembered, background: false, stop_processes_on_quit: false, profile: Profile::default(), restart: None, replacing: None }));
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
    fn should_stop_processes_on_quit_only_when_asked() {
        // given
        let lines = ["--stop-processes-on-quit ~/code", "~/code"];

        // when
        let stops: Vec<_> = lines.iter().map(|line| parse(line).map(|args| args.stop_processes_on_quit)).collect();

        // then
        assert_eq!(stops, [Ok(true), Ok(false)]);
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
    fn should_parse_replacing_pid() {
        // given
        let lines = ["--replacing 4242", "--replacing=4242"];

        // when
        let replacing: Vec<_> = lines.iter().map(|line| parse(line).map(|args| args.replacing)).collect();

        // then
        assert_eq!(replacing, [Ok(Some(4242)), Ok(Some(4242))]);
    }

    #[test]
    fn should_reject_a_replacing_that_is_not_a_pid() {
        // given
        let lines = ["--replacing", "--replacing ive", "--replacing=0"];

        // when
        let errors: Vec<_> = lines.iter().map(|line| parse(line).err()).collect();

        // then
        assert_eq!(
            errors,
            [Some(ArgsError::InvalidReplacing(String::new())), Some(ArgsError::InvalidReplacing("ive".to_owned())), Some(ArgsError::InvalidReplacing("0".to_owned()))]
        );
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

    #[cfg(windows)]
    #[test]
    fn should_accept_only_an_existing_powershell_restart_script() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let (script, plain) = (dir.path().join("restart.ps1"), dir.path().join("restart.sh"));
        std::fs::write(&script, "").unwrap();
        std::fs::write(&plain, "").unwrap();

        // when
        let checked = [check_restart_script(&script), check_restart_script(&plain), check_restart_script(&dir.path().join("gone.ps1"))];

        // then
        assert_eq!(checked[0], Ok(()));
        assert!(checked[1].as_ref().is_err_and(|error| error.contains("restart.sh")), "{checked:?}");
        assert!(checked[2].as_ref().is_err_and(|error| error.contains("gone.ps1")), "{checked:?}");
    }
}
