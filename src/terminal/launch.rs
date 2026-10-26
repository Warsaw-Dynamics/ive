//! What a terminal runs: the user's shell, or one command through it.

#[cfg(unix)]
use std::ffi::CStr;
#[cfg(windows)]
use std::sync::OnceLock;

/// The program a terminal starts and what it adds to the terminal's environment.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Launch {
    /// program and arguments; on Unix empty for the user's shell, as alacritty picks it
    pub program: Vec<String>,
    /// added after the terminal's own variables; may hold secrets, so it never goes on a command line
    pub env: Vec<(String, String)>,
}

impl Launch {
    /// The user's interactive shell; PowerShell on Windows.
    pub fn shell() -> Self {
        Self { program: interactive_shell(), env: Vec::new() }
    }

    /// `command` run by the user's login shell, so their `PATH` (mvn, nvm, ngrok) applies; by cmd on Windows.
    pub fn command(command: &str, env: Vec<(String, String)>) -> Self {
        Self { program: shell_command(command), env }
    }
}

#[cfg(unix)]
fn interactive_shell() -> Vec<String> {
    Vec::new()
}

/// PowerShell 7 when it is on `PATH`, else Windows PowerShell; without the banner either prints first.
#[cfg(windows)]
fn interactive_shell() -> Vec<String> {
    static SHELL: OnceLock<&str> = OnceLock::new();
    let shell = SHELL.get_or_init(|| if on_path("pwsh.exe") { "pwsh.exe" } else { "powershell.exe" });
    vec![(*shell).to_owned(), "-NoLogo".to_owned()]
}

#[cfg(windows)]
fn on_path(program: &str) -> bool {
    std::env::var_os("PATH").is_some_and(|path| std::env::split_paths(&path).any(|dir| dir.join(program).is_file()))
}

#[cfg(unix)]
fn shell_command(command: &str) -> Vec<String> {
    vec![login_shell(), "-lc".to_owned(), command.to_owned()]
}

/// `cmd /D /S /C "<command>"`, as Node.js runs a shell command: `/D` skips cmd's AutoRun commands, and `/S` takes
/// what lies between the outer quotes as it is, since the PTY passes the arguments on without escaping them.
#[cfg(windows)]
fn shell_command(command: &str) -> Vec<String> {
    let cmd = std::env::var("ComSpec").unwrap_or_else(|_| "cmd.exe".to_owned());
    vec![cmd, "/D".to_owned(), "/S".to_owned(), "/C".to_owned(), format!("\"{command}\"")]
}

/// The user's shell from the password database, as alacritty picks it on Linux; `/bin/sh` without one.
#[cfg(unix)]
pub fn login_shell() -> String {
    // SAFETY: getpwuid returns a pointer into static storage or null; it is read at once, on this thread
    unsafe {
        let entry = libc::getpwuid(libc::getuid());
        if entry.is_null() || (*entry).pw_shell.is_null() {
            return "/bin/sh".to_owned();
        }
        CStr::from_ptr((*entry).pw_shell).to_string_lossy().into_owned()
    }
}

/// The PowerShell a terminal starts.
#[cfg(windows)]
pub fn login_shell() -> String {
    interactive_shell().swap_remove(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn should_run_command_through_login_shell() {
        // given
        let env = vec![("SECRET".to_owned(), "s3cret".to_owned())];

        // when
        let launch = Launch::command("mvn test", env.clone());

        // then
        assert_eq!(launch.program[1..], ["-lc", "mvn test"]);
        assert_eq!(launch.env, env);
    }

    #[cfg(windows)]
    #[test]
    fn should_run_command_through_cmd_as_written() {
        // given
        let env = vec![("SECRET".to_owned(), "s3cret".to_owned())];

        // when
        let launch = Launch::command("npm run build && npm test", env.clone());

        // then
        assert_eq!(launch.program[1..], ["/D", "/S", "/C", "\"npm run build && npm test\""]);
        assert_eq!(launch.env, env);
    }

    #[cfg(unix)]
    #[test]
    fn should_start_the_default_shell_without_extra_env() {
        // given
        // when
        let launch = Launch::shell();

        // then
        assert!(launch.program.is_empty() && launch.env.is_empty());
    }

    #[cfg(windows)]
    #[test]
    fn should_start_powershell_without_its_banner() {
        // given
        // when
        let launch = Launch::shell();

        // then
        assert!(matches!(launch.program.as_slice(), [shell, logo] if (shell == "pwsh.exe" || shell == "powershell.exe") && logo == "-NoLogo"), "{launch:?}");
        assert!(launch.env.is_empty());
    }
}
