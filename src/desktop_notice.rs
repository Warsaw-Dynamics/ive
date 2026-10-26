//! Desktop notifications for processes that call for the user while IVE is in the background: the freedesktop
//! notification server through `notify-send` (a click shows the process) and `gdbus` (to withdraw one), macOS's
//! notification centre through `osascript` (no click, not withdrawn). Elsewhere, none.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};

use futures::channel::mpsc::UnboundedSender;

use crate::process::ProcessId;

/// The desktop notification shown for each process, and where a click on one goes.
pub struct DesktopNotices {
    /// each process's notification, its server id once `notify-send` printed it
    sent: HashMap<ProcessId, Arc<Shown>>,
    clicks: UnboundedSender<ProcessId>,
}

impl DesktopNotices {
    pub fn new(clicks: UnboundedSender<ProcessId>) -> Self {
        Self { sent: HashMap::new(), clicks }
    }

    /// Shows `body` titled `title` for process `id`, in place of the one it had: replaced on the server when its id
    /// is known, else taken back once it is.
    pub fn send(&mut self, id: ProcessId, title: &str, body: &str) {
        let shown = Arc::new(Shown::new(platform::close));
        let replace = match self.sent.insert(id, shown.clone()) {
            Some(before) => before.id.get().copied().or_else(|| {
                before.withdraw();
                None
            }),
            None => None,
        };
        platform::send(Notice { process: id, title: title.to_owned(), body: body.to_owned(), replace, shown, clicks: self.clicks.clone() });
    }

    /// Takes process `id`'s notification off the desktop, now or once its id is known.
    pub fn withdraw(&mut self, id: ProcessId) {
        if let Some(shown) = self.sent.remove(&id) {
            shown.withdraw();
        }
    }

    pub fn withdraw_all(&mut self) {
        for (_, shown) in self.sent.drain() {
            shown.withdraw();
        }
    }
}

/// A notification's server id once `notify-send` printed it, and whether IVE took it back before that.
struct Shown {
    id: OnceLock<u32>,
    withdrawn: AtomicBool,
    close: fn(u32),
}

impl Shown {
    fn new(close: fn(u32)) -> Self {
        Self { id: OnceLock::new(), withdrawn: AtomicBool::new(false), close }
    }

    /// Closes the notification now, or once its id comes.
    fn withdraw(&self) {
        self.withdrawn.store(true, Ordering::SeqCst);
        if let Some(id) = self.id.get() {
            (self.close)(*id);
        }
    }

    /// The server's id came; a notification taken back meanwhile closes now.
    #[cfg_attr(not(all(unix, not(target_os = "macos"))), allow(dead_code))]
    fn learn(&self, id: u32) {
        let _ = self.id.set(id);
        if self.withdrawn.load(Ordering::SeqCst) {
            (self.close)(id);
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
    shown: Arc<Shown>,
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

    /// Runs `notify-send` from a thread of its own, which waits for the notification's id and a click, and reaps it.
    pub(super) fn send(notice: Notice) {
        let started = std::thread::Builder::new().name("ive-desktop-notice".to_owned()).spawn(move || show(&notice));
        if let Err(error) = started {
            report_once(format!("cannot start a thread for notify-send: {error}"));
        }
    }

    fn show(notice: &Notice) {
        let spawned = Command::new("notify-send")
            .args(notify_send_args(&notice.title, &notice.body, notice.replace))
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn();
        let mut child = match spawned {
            Ok(child) => child,
            Err(error) => return report_once(format!("cannot run notify-send: {error}")),
        };
        if let Some(stdout) = child.stdout.take() {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                answer(notice, read_printed(&line));
            }
        }
        match child.wait() {
            Ok(status) if status.success() => {}
            Ok(status) => report_once(format!("notify-send failed ({status})")),
            Err(error) => report_once(format!("notify-send failed: {error}")),
        }
    }

    fn answer(notice: &Notice, printed: Printed) {
        match printed {
            Printed::Id(id) => notice.shown.learn(id),
            Printed::Clicked => {
                let _ = notice.clicks.unbounded_send(notice.process);
            }
            Printed::Other => {}
        }
    }

    /// Closes notification `id` through `gdbus`, started at once so it runs even as IVE quits; a thread reaps it.
    pub(super) fn close(id: u32) {
        let spawned = Command::new("gdbus").args(close_args(id)).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn();
        match spawned {
            Ok(mut child) => {
                let _ = std::thread::Builder::new().name("ive-desktop-notice-close".to_owned()).spawn(move || child.wait());
            }
            Err(error) => report_once(format!("cannot run gdbus to withdraw a notification: {error}")),
        }
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
    use std::cell::RefCell;

    use super::*;

    thread_local! {
        static CLOSED: RefCell<Vec<u32>> = const { RefCell::new(Vec::new()) };
    }

    fn record_close(id: u32) {
        CLOSED.with(|closed| closed.borrow_mut().push(id));
    }

    fn closed() -> Vec<u32> {
        CLOSED.with(|closed| closed.borrow().clone())
    }

    #[test]
    fn should_close_a_notification_withdrawn_before_its_id_came() {
        // given
        let shown = Shown::new(record_close);

        // when
        shown.withdraw();
        let before = closed();
        shown.learn(5);

        // then
        assert_eq!((before, closed()), (vec![], vec![5]));
    }

    #[test]
    fn should_close_at_once_when_the_id_is_known() {
        // given
        let shown = Shown::new(record_close);
        shown.learn(7);

        // when
        shown.withdraw();

        // then
        assert_eq!(closed(), [7]);
    }

    #[test]
    fn should_leave_a_notification_not_withdrawn() {
        // given
        let shown = Shown::new(record_close);

        // when
        shown.learn(9);

        // then
        assert_eq!(closed(), Vec::<u32>::new());
    }

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
