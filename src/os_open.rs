//! Handing a path to the OS: its file manager with the path selected, its default application, its browser.

use std::path::Path;
use std::process::{Command, Stdio};

/// A program to start and its arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Launch {
    pub program: String,
    pub args: Vec<String>,
}

impl Launch {
    pub fn new(program: &str, args: &[&str]) -> Self {
        Self { program: program.to_owned(), args: args.iter().map(|arg| (*arg).to_owned()).collect() }
    }
}

/// The file manager with `path` selected, as launches to try in order: on Linux the freedesktop file manager
/// interface, else the folder it is in.
pub fn reveal(path: &Path) -> Vec<Launch> {
    let text = path.to_string_lossy();
    if cfg!(windows) {
        return vec![Launch::new("explorer", &[&format!("/select,{text}")])];
    }
    if cfg!(target_os = "macos") {
        return vec![Launch::new("open", &["-R", &text])];
    }
    let item = format!("array:string:{}", file_url(path));
    let show = ["--session", "--print-reply", "--dest=org.freedesktop.FileManager1", "--type=method_call", "/org/freedesktop/FileManager1", "org.freedesktop.FileManager1.ShowItems", &item, "string:"];
    let parent = path.parent().unwrap_or(path).to_string_lossy();
    vec![Launch::new("dbus-send", &show), Launch::new("xdg-open", &[&parent])]
}

/// `path` in the application the OS opens it with.
pub fn default_app(path: &Path) -> Launch {
    Launch::new(OPENER, &[&path.to_string_lossy()])
}

/// What hands a path or URL to its default application.
const OPENER: &str = if cfg!(windows) {
    "explorer"
} else if cfg!(target_os = "macos") {
    "open"
} else {
    "xdg-open"
};

/// `path`'s `file://` URL in the default browser.
pub fn browser(path: &Path) -> Launch {
    Launch::new(OPENER, &[&file_url(path)])
}

/// `file://` and `path`, its bytes outside the unreserved characters and `/` percent-encoded.
pub fn file_url(path: &Path) -> String {
    let encoded: String = url_path(path).iter().map(|&byte| match byte {
        b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' | b'/' => char::from(byte).to_string(),
        // a drive's colon, as in `file:///C:/Users`
        b':' if cfg!(windows) => ":".to_owned(),
        other => format!("%{other:02X}"),
    }).collect();
    format!("file://{encoded}")
}

#[cfg(unix)]
fn url_path(path: &Path) -> Vec<u8> {
    use std::os::unix::ffi::OsStrExt as _;
    path.as_os_str().as_bytes().to_vec()
}

/// `C:\a` as `/C:/a`.
#[cfg(windows)]
fn url_path(path: &Path) -> Vec<u8> {
    let slashed = path.to_string_lossy().replace('\\', "/");
    let rooted = if slashed.starts_with('/') { slashed } else { format!("/{slashed}") };
    rooted.into_bytes()
}

/// Starts the first of `launches` that succeeds, off the calling thread; when none does, stderr says why.
pub fn start(launches: Vec<Launch>) {
    std::thread::spawn(move || {
        let failures: Vec<String> = launches.iter().map_while(|launch| run(launch).err()).collect();
        if failures.len() == launches.len() {
            crate::notices::log_only(&format!("cannot hand a path to the OS: {}", failures.join("; ")));
        }
    });
}

/// Runs `launch` to its end; the error names the program and what went wrong.
fn run(launch: &Launch) -> Result<(), String> {
    let status = Command::new(&launch.program).args(&launch.args).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).status();
    match status {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => Err(format!("{} {:?} ended with {status}", launch.program, launch.args)),
        Err(error) => Err(format!("cannot start {}: {error}", launch.program)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_build_reveal_commands_for_this_os() {
        // when
        let launches = reveal(Path::new("/w/a b.rs"));

        // then
        if cfg!(windows) {
            assert_eq!(launches, [Launch::new("explorer", &["/select,/w/a b.rs"])]);
        } else if cfg!(target_os = "macos") {
            assert_eq!(launches, [Launch::new("open", &["-R", "/w/a b.rs"])]);
        } else {
            assert_eq!(launches[0].program, "dbus-send");
            assert!(launches[0].args.contains(&"array:string:file:///w/a%20b.rs".to_owned()), "{:?}", launches[0]);
            assert_eq!(launches[1], Launch::new("xdg-open", &["/w"]));
        }
    }

    #[test]
    fn should_open_with_the_default_app_of_this_os() {
        // when
        let launch = default_app(Path::new("/w/a.png"));

        // then
        assert_eq!(launch, Launch::new(OPENER, &["/w/a.png"]));
    }

    #[test]
    fn should_open_a_file_url_in_the_browser() {
        // when
        let launch = browser(Path::new("/w/index.html"));

        // then
        assert_eq!(launch.args, ["file:///w/index.html"]);
    }

    #[test]
    fn should_percent_encode_a_file_url() {
        // when
        let url = file_url(Path::new("/a b/ä#.html"));

        // then
        assert_eq!(url, "file:///a%20b/%C3%A4%23.html");
    }
}
