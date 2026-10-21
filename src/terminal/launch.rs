//! What a terminal runs: the user's shell, or one command through it.

#[cfg(unix)]
use std::ffi::CStr;

/// The program a terminal starts and what it adds to the terminal's environment.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Launch {
    /// program and arguments; empty for the user's shell, as alacritty picks it
    pub program: Vec<String>,
    /// added after the terminal's own variables; may hold secrets, so it never goes on a command line
    pub env: Vec<(String, String)>,
}

impl Launch {
    /// The user's interactive shell.
    pub fn shell() -> Self {
        Self::default()
    }

    /// `command` run by the user's login shell, so their `PATH` (mvn, nvm, ngrok) applies.
    pub fn command(command: &str, env: Vec<(String, String)>) -> Self {
        Self { program: vec![login_shell(), "-lc".to_owned(), command.to_owned()], env }
    }
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

#[cfg(not(unix))]
pub fn login_shell() -> String {
    "sh".to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn should_start_the_default_shell_without_extra_env() {
        // given
        // when
        let launch = Launch::shell();

        // then
        assert!(launch.program.is_empty() && launch.env.is_empty());
    }
}
