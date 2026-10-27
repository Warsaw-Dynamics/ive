//! Running a shell scratch: which shell, and what is typed into the new terminal.

use std::path::Path;
#[cfg(windows)]
use std::path::PathBuf;
#[cfg(windows)]
use std::sync::OnceLock;

use crate::pasted::single_quoted;

/// The shell a scratch's extension names: `.sh` runs with `sh`, `.bash` with `bash`, `.zsh` with `zsh`; other
/// kinds do not run (they would need databases or run configurations).
#[cfg(unix)]
pub fn shell_for(name: &str) -> Option<String> {
    match extension_of(name).as_deref() {
        Some("sh") => Some("sh".to_owned()),
        Some("bash") => Some("bash".to_owned()),
        Some("zsh") => Some("zsh".to_owned()),
        _ => None,
    }
}

/// Windows: `.ps1` runs with PowerShell, `.cmd` and `.bat` with cmd, `.sh` and `.bash` with Git Bash when it is
/// installed. The result is typed into a PowerShell terminal.
#[cfg(windows)]
pub fn shell_for(name: &str) -> Option<String> {
    match extension_of(name).as_deref() {
        Some("ps1") => Some("powershell -NoProfile -ExecutionPolicy Bypass -File".to_owned()),
        Some("cmd" | "bat") => Some("cmd /D /C".to_owned()),
        Some("sh" | "bash") => git_bash().map(|bash| format!("& {}", single_quoted(&bash.to_string_lossy()))),
        _ => None,
    }
}

fn extension_of(name: &str) -> Option<String> {
    Path::new(name).extension().map(|extension| extension.to_string_lossy().to_ascii_lowercase())
}

/// `CLAUDE_CODE_GIT_BASH_PATH`, else the `bin\bash.exe` of the Git whose `cmd\git.exe` is on `PATH`; never
/// `System32\bash.exe`, which starts WSL.
#[cfg(windows)]
fn git_bash() -> Option<&'static Path> {
    static BASH: OnceLock<Option<PathBuf>> = OnceLock::new();
    BASH.get_or_init(|| {
        let configured = std::env::var_os("CLAUDE_CODE_GIT_BASH_PATH").map(PathBuf::from).filter(|path| path.is_file());
        configured.or_else(|| {
            let path = std::env::var_os("PATH")?;
            std::env::split_paths(&path)
                .filter(|dir| dir.join("git.exe").is_file())
                .filter_map(|dir| Some(dir.parent()?.join("bin").join("bash.exe")))
                .find(|bash| bash.is_file())
        })
    })
    .as_deref()
}

/// `<shell> '<path>'` and Enter. A path holding control characters is refused: typed into a terminal they act as
/// keys, and Ctrl+U would clear the line so that the rest of the name runs as a command.
pub fn command(shell: &str, path: &Path) -> Result<String, String> {
    let path = path.to_string_lossy();
    if path.chars().any(char::is_control) {
        return Err(format!("cannot run {path:?}: its path holds control characters"));
    }
    Ok(format!("{shell} {}\r", single_quoted(&path)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn should_quote_the_path_for_the_shell() {
        // given
        let path = Path::new("/home/kb/.config/ive/scratches/app/it's $HOME here.sh");

        // when
        let typed = command("sh", path);

        // then
        assert_eq!(typed.as_deref(), Ok("sh '/home/kb/.config/ive/scratches/app/it'\\''s $HOME here.sh'\r"));
    }

    #[test]
    fn should_refuse_to_type_a_path_with_control_characters() {
        // given
        let path = Path::new("/home/kb/.config/ive/scratches/app/\u{15}touch pwned #.sh");

        // when
        let typed = command("sh", path);

        // then
        assert_eq!(typed, Err(r#"cannot run "/home/kb/.config/ive/scratches/app/\u{15}touch pwned #.sh": its path holds control characters"#.to_owned()));
    }

    #[cfg(unix)]
    #[test]
    fn should_run_each_shell_scratch_with_the_shell_its_extension_names() {
        // given
        let names = ["a.sh", "b.BASH", "c.zsh", "d.sql", "sh", "e.http"];

        // when
        let shells = names.map(shell_for);

        // then
        assert_eq!(shells.each_ref().map(Option::as_deref), [Some("sh"), Some("bash"), Some("zsh"), None, None, None]);
    }

    #[cfg(windows)]
    #[test]
    fn should_run_powershell_and_cmd_scratches_and_no_zsh() {
        // given
        let path = Path::new(r"C:\Users\kb\it's.ps1");

        // when
        let typed = shell_for("a.PS1").map(|shell| command(&shell, path));

        // then
        assert_eq!(typed, Some(Ok(r"powershell -NoProfile -ExecutionPolicy Bypass -File 'C:\Users\kb\it''s.ps1'".to_owned() + "\r")));
        assert_eq!([shell_for("b.bat"), shell_for("c.zsh")], [Some("cmd /D /C".to_owned()), None]);
    }
}
