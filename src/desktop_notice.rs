//! Desktop notifications for processes that call for the user while IVE is in the background: the freedesktop
//! notification server through `notify-send` (a click shows the process) and `gdbus` (to withdraw one), macOS's
//! notification centre through `osascript` (no click, not withdrawn). Elsewhere, none.

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use futures::channel::mpsc::UnboundedSender;

use crate::process::ProcessId;

/// The desktop notification shown for each process, and where a click on one goes.
pub struct DesktopNotices {
    /// the server's id of each process's notification, once `notify-send` printed it
    sent: HashMap<ProcessId, Arc<OnceLock<u32>>>,
    clicks: UnboundedSender<ProcessId>,
}

impl DesktopNotices {
    pub fn new(clicks: UnboundedSender<ProcessId>) -> Self {
        Self { sent: HashMap::new(), clicks }
    }

    /// Shows `body` titled `title` for process `id`, in place of the one it had.
    pub fn send(&mut self, id: ProcessId, title: &str, body: &str) {
        let replace = self.sent.get(&id).and_then(|sent| sent.get().copied());
        let shown = Arc::new(OnceLock::new());
        self.sent.insert(id, shown.clone());
        platform::send(Notice { process: id, title: title.to_owned(), body: body.to_owned(), replace, shown, clicks: self.clicks.clone() });
    }

    /// Takes process `id`'s notification off the desktop.
    pub fn withdraw(&mut self, id: ProcessId) {
        if let Some(number) = self.sent.remove(&id).and_then(|sent| sent.get().copied()) {
            platform::close(number);
        }
    }

    pub fn withdraw_all(&mut self) {
        let ids: Vec<ProcessId> = self.sent.keys().copied().collect();
        for id in ids {
            self.withdraw(id);
        }
    }
}

/// One notification to show, and where its id and click go; only freedesktop servers read them all.
#[cfg_attr(not(all(unix, not(target_os = "macos"))), allow(dead_code))]
struct Notice {
    process: ProcessId,
    title: String,
    body: String,
    /// the server's id of the notification this one replaces
    replace: Option<u32>,
    shown: Arc<OnceLock<u32>>,
    clicks: UnboundedSender<ProcessId>,
}

/// What `notify-send --print-id --action=default=Show` printed on one line.
#[cfg(any(all(unix, not(target_os = "macos")), test))]
#[derive(Debug, PartialEq, Eq)]
enum Printed {
    /// the notification's id, printed first
    Id(u32),
    /// the user clicked the notification
    Clicked,
    Other,
}

/// IVE's name in the notification, and the action a click on it runs.
#[cfg(any(all(unix, not(target_os = "macos")), test))]
const APP_NAME: &str = "IVE";
#[cfg(any(all(unix, not(target_os = "macos")), test))]
const CLICK_ACTION: &str = "default";

#[cfg(any(all(unix, not(target_os = "macos")), test))]
fn notify_send_args(title: &str, body: &str, replace: Option<u32>) -> Vec<String> {
    let mut args = vec![format!("--app-name={APP_NAME}"), "--print-id".to_owned()];
    args.extend(replace.map(|id| format!("--replace-id={id}")));
    args.push(format!("--action={CLICK_ACTION}=Show"));
    // `--` keeps a title or body starting with `-` from reading as an option
    args.extend(["--".to_owned(), title.to_owned(), body.to_owned()]);
    args
}

#[cfg(any(all(unix, not(target_os = "macos")), test))]
fn read_printed(line: &str) -> Printed {
    let line = line.trim();
    if line == CLICK_ACTION {
        return Printed::Clicked;
    }
    line.parse().map_or(Printed::Other, Printed::Id)
}

#[cfg(any(all(unix, not(target_os = "macos")), test))]
fn close_args(id: u32) -> Vec<String> {
    let call = ["call", "--session", "--dest", "org.freedesktop.Notifications", "--object-path", "/org/freedesktop/Notifications"];
    let method = ["--method", "org.freedesktop.Notifications.CloseNotification"];
    call.iter().chain(method.iter()).map(|arg| (*arg).to_owned()).chain([id.to_string()]).collect()
}

#[cfg(any(target_os = "macos", test))]
fn osascript_args(title: &str, body: &str) -> Vec<String> {
    let quoted = |text: &str| format!("\"{}\"", text.replace('\\', "\\\\").replace('"', "\\\""));
    vec!["-e".to_owned(), format!("display notification {} with title {}", quoted(body), quoted(title))]
}

/// Logs the first failure of a session to `ive.log`; later ones would only repeat it.
#[cfg(unix)]
fn report_once(failure: String) {
    static REPORTED: OnceLock<()> = OnceLock::new();
    if REPORTED.set(()).is_ok() {
        crate::notices::log_only(&format!("desktop notifications: {failure}; IVE shows waiting processes only in its window"));
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
mod platform {
    use std::io::{BufRead, BufReader};
    use std::process::{Command, Stdio};

    use super::{Notice, Printed, close_args, notify_send_args, read_printed, report_once};

    /// Runs `notify-send` on a thread of its own, which waits for the notification's id and a click.
    pub(super) fn send(notice: Notice) {
        let spawned = Command::new("notify-send")
            .args(notify_send_args(&notice.title, &notice.body, notice.replace))
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();
        let mut child = match spawned {
            Ok(child) => child,
            Err(error) => return report_once(format!("cannot run notify-send: {error}")),
        };
        let Some(stdout) = child.stdout.take() else {
            return;
        };
        let started = std::thread::Builder::new().name("ive-desktop-notice".to_owned()).spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                match read_printed(&line) {
                    Printed::Id(id) => {
                        let _ = notice.shown.set(id);
                    }
                    Printed::Clicked => {
                        let _ = notice.clicks.unbounded_send(notice.process);
                    }
                    Printed::Other => {}
                }
            }
            match child.wait_with_output() {
                Ok(output) if output.status.success() => {}
                Ok(output) => report_once(format!("notify-send failed ({}): {}", output.status, String::from_utf8_lossy(&output.stderr).trim())),
                Err(error) => report_once(format!("notify-send failed: {error}")),
            }
        });
        if let Err(error) = started {
            report_once(format!("cannot start a thread for notify-send: {error}"));
        }
    }

    /// Closes notification `id` through `gdbus`, on a thread that waits for it.
    pub(super) fn close(id: u32) {
        let _ = std::thread::Builder::new().name("ive-desktop-notice-close".to_owned()).spawn(move || {
            let status = Command::new("gdbus").args(close_args(id)).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).status();
            if let Err(error) = status {
                report_once(format!("cannot run gdbus to withdraw a notification: {error}"));
            }
        });
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use std::process::{Command, Stdio};

    use super::{Notice, osascript_args, report_once};

    /// Runs `osascript` on a thread that waits for it; macOS gives no click back.
    pub(super) fn send(notice: Notice) {
        let _ = std::thread::Builder::new().name("ive-desktop-notice".to_owned()).spawn(move || {
            let status = Command::new("osascript").args(osascript_args(&notice.title, &notice.body)).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).status();
            match status {
                Ok(status) if status.success() => {}
                Ok(status) => report_once(format!("osascript failed ({status})")),
                Err(error) => report_once(format!("cannot run osascript: {error}")),
            }
        });
    }

    pub(super) fn close(_id: u32) {}
}

#[cfg(not(unix))]
mod platform {
    use super::Notice;

    pub(super) fn send(_notice: Notice) {}

    pub(super) fn close(_id: u32) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_build_notify_send_args() {
        // when
        let first = notify_send_args("Claude Code — dev", "-needs you", None);
        let again = notify_send_args("t", "b", Some(7));

        // then
        assert_eq!(first, ["--app-name=IVE", "--print-id", "--action=default=Show", "--", "Claude Code — dev", "-needs you"]);
        assert_eq!(again, ["--app-name=IVE", "--print-id", "--replace-id=7", "--action=default=Show", "--", "t", "b"]);
    }

    #[test]
    fn should_read_id_and_click() {
        // when
        let printed = ["42\n", "default\n", "", "1"].map(read_printed);

        // then
        assert_eq!(printed, [Printed::Id(42), Printed::Clicked, Printed::Other, Printed::Id(1)]);
    }

    #[test]
    fn should_close_by_id_over_dbus() {
        // when
        let args = close_args(42);

        // then
        assert_eq!(args.last().map(String::as_str), Some("42"));
        assert!(args.contains(&"org.freedesktop.Notifications.CloseNotification".to_owned()), "{args:?}");
    }

    #[test]
    fn should_escape_osascript_strings() {
        // when
        let args = osascript_args(r#"say "hi""#, r"a\b");

        // then
        assert_eq!(args, ["-e", r#"display notification "a\\b" with title "say \"hi\"""#]);
    }
}
