//! Running a shell scratch: which shell, and what is typed into the new terminal.

use std::path::Path;

/// The shell a scratch's extension names: `.sh` runs with `sh`, `.bash` with `bash`, `.zsh` with `zsh`; other
/// kinds do not run (they would need databases or run configurations).
pub fn shell_for(name: &str) -> Option<&'static str> {
    let extension = Path::new(name).extension().map(|extension| extension.to_string_lossy().to_ascii_lowercase());
    match extension.as_deref() {
        Some("sh") => Some("sh"),
        Some("bash") => Some("bash"),
        Some("zsh") => Some("zsh"),
        _ => None,
    }
}

/// Single-quoted for a POSIX shell: `'` becomes `'\''`.
pub fn quote(text: &str) -> String {
    format!("'{}'", text.replace('\'', "'\\''"))
}

/// `<shell> '<path>'` and Enter. A path holding control characters is refused: typed into a terminal they act as
/// keys, and Ctrl+U would clear the line so that the rest of the name runs as a command.
pub fn command(shell: &str, path: &Path) -> Result<String, String> {
    let path = path.to_string_lossy();
    if path.chars().any(char::is_control) {
        return Err(format!("cannot run {path:?}: its path holds control characters"));
    }
    Ok(format!("{shell} {}\r", quote(&path)))
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn should_run_each_shell_scratch_with_the_shell_its_extension_names() {
        // given
        let names = ["a.sh", "b.BASH", "c.zsh", "d.sql", "sh", "e.http"];

        // when
        let shells = names.map(shell_for);

        // then
        assert_eq!(shells, [Some("sh"), Some("bash"), Some("zsh"), None, None, None]);
    }
}
