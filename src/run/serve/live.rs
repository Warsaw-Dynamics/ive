//! Live reload: a script in each HTML page listening for changes under the served folder.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError, mpsc};

use notify::Watcher;
use std::time::{Duration, Instant};

/// Where the live-reload script and its event stream are served; no page of the site can use these paths.
pub const SCRIPT_PATH: &str = "/__ive/live.js";
pub const EVENTS_PATH: &str = "/__ive/live";
/// What HTML responses get before `</body>`.
pub const LIVE_TAG: &str = "<script src=\"/__ive/live.js\"></script>";
/// Swaps stylesheets on `css`, reloads on anything else; reconnects after the server restarts.
pub const LIVE_JS: &str = r#"(() => {
  const events = new EventSource("/__ive/live");
  events.onmessage = (event) => {
    if (event.data !== "css") { location.reload(); return; }
    for (const link of document.querySelectorAll('link[rel="stylesheet"]')) {
      const url = new URL(link.href);
      url.searchParams.set("ive-live", Date.now());
      link.href = url.toString();
    }
  };
})();
"#;

/// What a batch of changes asks the page to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Change {
    /// only stylesheets changed: swap them in place
    Css,
    Reload,
}

impl Change {
    /// The event's data, as `LIVE_JS` reads it.
    pub fn data(self) -> &'static str {
        match self {
            Self::Css => "css",
            Self::Reload => "reload",
        }
    }
}

/// `html` with [`LIVE_TAG`] before its last `</body>`, or at its end without one.
pub fn inject(html: &str) -> String {
    match html.to_ascii_lowercase().rfind("</body>") {
        Some(at) => format!("{}{LIVE_TAG}{}", &html[..at], &html[at..]),
        None => format!("{html}{LIVE_TAG}"),
    }
}

/// The files the server sent: only their changes can show on a page, so only their folders are watched, as they are
/// served. Watching the whole root instead would reload pages on every edit under it (a root wide enough for shared
/// styles holds sources too) and receive every read of it.
pub struct Served {
    watcher: Mutex<notify::RecommendedWatcher>,
    seen: Mutex<Seen>,
}

#[derive(Default)]
struct Seen {
    files: HashSet<PathBuf>,
    folders: HashSet<PathBuf>,
}

impl Served {
    /// Nothing served yet, and where the events of the folders it will watch arrive.
    pub fn start() -> Result<(Self, mpsc::Receiver<notify::Event>), String> {
        let (events, received) = mpsc::channel();
        let watcher = notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
            if let Ok(event) = event {
                let _ = events.send(event);
            }
        })
        .map_err(|error| format!("cannot start watching for live reload: {error}"))?;
        Ok((Self { watcher: Mutex::new(watcher), seen: Mutex::default() }, received))
    }

    /// Records `file` (canonical) as served, watching its folder from its first served file on.
    pub fn add(&self, file: &Path) -> Result<(), String> {
        let mut seen = self.seen.lock().unwrap_or_else(PoisonError::into_inner);
        seen.files.insert(file.to_path_buf());
        let Some(folder) = file.parent() else { return Ok(()) };
        if seen.folders.contains(folder) {
            return Ok(());
        }
        let mut watcher = self.watcher.lock().unwrap_or_else(PoisonError::into_inner);
        watcher.watch(folder, notify::RecursiveMode::NonRecursive).map_err(|error| format!("cannot watch {} for live reload: {error}", folder.display()))?;
        seen.folders.insert(folder.to_path_buf());
        Ok(())
    }

    pub fn contains(&self, path: &Path) -> bool {
        self.seen.lock().unwrap_or_else(PoisonError::into_inner).files.contains(path)
    }
}

/// The files `event` changed, leaving out the `ignored` ones; none when it only read them (opening a file to serve
/// it is an event too, and reloading on it would reload forever).
pub fn changed_paths(event: &notify::Event, ignored: impl Fn(&Path) -> bool) -> Vec<PathBuf> {
    use notify::event::{MetadataKind, ModifyKind};
    let read_only = matches!(event.kind, notify::EventKind::Modify(ModifyKind::Metadata(MetadataKind::AccessTime)));
    if read_only || !(event.kind.is_create() || event.kind.is_modify() || event.kind.is_remove()) {
        return Vec::new();
    }
    event.paths.iter().filter(|path| !ignored(path)).cloned().collect()
}

/// The files the next burst of changes touched: from the first change until `quiet` passes without another. Events
/// that change nothing (reads, ignored files) neither start a burst nor extend it. None once the watcher is gone.
pub fn next_burst(received: &mpsc::Receiver<notify::Event>, quiet: Duration, ignored: impl Fn(&Path) -> bool) -> Option<Vec<PathBuf>> {
    let mut paths = Vec::new();
    while paths.is_empty() {
        paths = changed_paths(&received.recv().ok()?, &ignored);
    }
    let mut ends = Instant::now() + quiet;
    while let Ok(more) = received.recv_timeout(ends.saturating_duration_since(Instant::now())) {
        let more = changed_paths(&more, &ignored);
        if more.is_empty() {
            continue;
        }
        paths.extend(more);
        ends = Instant::now() + quiet;
    }
    Some(paths)
}

/// The log line for a burst, its paths relative to `root`: `mock/app.css changed: stylesheet swapped`.
pub fn describe(paths: &[PathBuf], root: &Path) -> String {
    const NAMED: usize = 3;
    let mut changed: Vec<String> = paths.iter().map(|path| path.strip_prefix(root).unwrap_or(path).display().to_string()).collect();
    changed.sort();
    changed.dedup();
    let more = changed.len().saturating_sub(NAMED);
    changed.truncate(NAMED);
    let files = match more {
        0 => changed.join(", "),
        _ => format!("{} and {more} more", changed.join(", ")),
    };
    let done = match change_of(paths) {
        Change::Css => "stylesheet swapped",
        Change::Reload => "page reloaded",
    };
    format!("{files} changed: {done}")
}

/// Css when every changed file is a stylesheet, else Reload.
pub fn change_of(paths: &[PathBuf]) -> Change {
    let all_css = paths.iter().all(|path| path.extension().is_some_and(|extension| extension.eq_ignore_ascii_case("css")));
    if all_css && !paths.is_empty() { Change::Css } else { Change::Reload }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn should_inject_live_script_before_body_end() {
        // given
        let html = "<html><body><p>hi</p></BODY></html>";

        // when
        let injected = inject(html);

        // then
        assert_eq!(injected, format!("<html><body><p>hi</p>{LIVE_TAG}</BODY></html>"));
    }

    #[test]
    fn should_append_live_script_without_body() {
        // given
        let html = "<p>hi</p>";

        // when
        let injected = inject(html);

        // then
        assert_eq!(injected, format!("<p>hi</p>{LIVE_TAG}"));
    }

    #[test]
    fn should_swap_stylesheets_only_when_only_css_changed() {
        // given
        let css = vec![PathBuf::from("/s/app.css"), PathBuf::from("/s/theme.CSS")];
        let mixed = vec![PathBuf::from("/s/app.css"), PathBuf::from("/s/index.html")];

        // when
        let kinds = [change_of(&css), change_of(&mixed)];

        // then
        assert_eq!(kinds, [Change::Css, Change::Reload]);
    }

    fn event(kind: notify::EventKind, path: &str) -> notify::Event {
        notify::Event::new(kind).add_path(PathBuf::from(path))
    }

    #[test]
    fn should_not_count_reading_a_file_as_a_change() {
        // given
        let opened = event(notify::EventKind::Access(notify::event::AccessKind::Open(notify::event::AccessMode::Read)), "/s/index.html");
        let read = event(notify::EventKind::Access(notify::event::AccessKind::Close(notify::event::AccessMode::Read)), "/s/app.css");
        let accessed = event(notify::EventKind::Modify(notify::event::ModifyKind::Metadata(notify::event::MetadataKind::AccessTime)), "/s/a.js");

        // when
        let changed = [&opened, &read, &accessed].map(|event| changed_paths(event, |_| false));

        // then
        assert_eq!(changed, [Vec::<PathBuf>::new(), Vec::new(), Vec::new()]);
    }

    #[test]
    fn should_count_written_created_and_removed_files_unless_ignored() {
        // given
        let written = event(notify::EventKind::Modify(notify::event::ModifyKind::Data(notify::event::DataChange::Content)), "/s/app.css");
        let created = event(notify::EventKind::Create(notify::event::CreateKind::File), "/s/new.html");
        let removed = event(notify::EventKind::Remove(notify::event::RemoveKind::File), "/s/old.html");
        let built = event(notify::EventKind::Create(notify::event::CreateKind::File), "/s/target/debug/ive");
        let ignored = |path: &std::path::Path| path.starts_with("/s/target");

        // when
        let changed = [&written, &created, &removed, &built].map(|event| changed_paths(event, ignored));

        // then
        assert_eq!(changed, [vec![PathBuf::from("/s/app.css")], vec![PathBuf::from("/s/new.html")], vec![PathBuf::from("/s/old.html")], Vec::new()]);
    }

    fn written(path: &str) -> notify::Event {
        event(notify::EventKind::Modify(notify::event::ModifyKind::Data(notify::event::DataChange::Content)), path)
    }

    #[test]
    fn should_join_changes_closer_than_the_quiet_time_into_one_burst() {
        // given
        let (events, received) = mpsc::channel();
        events.send(written("/s/index.html")).unwrap();
        events.send(written("/s/app.css")).unwrap();

        // when
        let burst = next_burst(&received, Duration::from_millis(50), |_| false);

        // then
        assert_eq!(burst, Some(vec![PathBuf::from("/s/index.html"), PathBuf::from("/s/app.css")]));
    }

    #[test]
    fn should_end_a_burst_while_files_keep_being_read() {
        // given
        let (events, received) = mpsc::channel();
        events.send(written("/s/app.css")).unwrap();
        let reads = std::thread::spawn(move || {
            let opened = event(notify::EventKind::Access(notify::event::AccessKind::Open(notify::event::AccessMode::Read)), "/s");
            for _ in 0..500 {
                let _ = events.send(opened.clone());
                std::thread::sleep(Duration::from_millis(2));
            }
        });
        let started = Instant::now();

        // when
        let burst = next_burst(&received, Duration::from_millis(50), |_| false);

        // then
        let waited = started.elapsed();
        drop(received);
        reads.join().unwrap();
        assert_eq!(burst, Some(vec![PathBuf::from("/s/app.css")]));
        assert!(waited < Duration::from_millis(500), "the burst ended after {waited:?}, not 50 ms after its last change");
    }

    #[test]
    fn should_name_the_changed_files_relative_to_the_root() {
        // given
        let saved_twice = ["/s/src/rows.rs", "/s/mock/app.css", "/s/src/rows.rs"].map(PathBuf::from);
        let checked_out = ["/s/a.html", "/s/b.html", "/s/c.html", "/s/d.html", "/s/e.html"].map(PathBuf::from);

        // when
        let lines = [describe(&saved_twice, Path::new("/s")), describe(&checked_out, Path::new("/s"))];

        // then
        assert_eq!(lines, ["mock/app.css, src/rows.rs changed: page reloaded", "a.html, b.html, c.html and 2 more changed: page reloaded"]);
    }

    #[test]
    fn should_count_only_changes_to_served_files() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        std::fs::create_dir(root.join("other")).unwrap();
        for file in ["page.html", "unserved.rs", "other/app.css"] {
            std::fs::write(root.join(file), "old").unwrap();
        }
        let (served, received) = Served::start().unwrap();
        served.add(&root.join("page.html")).unwrap();

        // when
        for file in ["unserved.rs", "other/app.css", "page.html"] {
            std::fs::write(root.join(file), "new").unwrap();
        }
        let burst = next_burst(&received, Duration::from_millis(100), |path| !served.contains(path));

        // then
        let mut burst = burst.unwrap();
        burst.dedup();
        assert_eq!(burst, [root.join("page.html")]);
    }

    #[test]
    fn should_end_once_the_watcher_is_gone() {
        // given
        let (events, received) = mpsc::channel::<notify::Event>();
        drop(events);

        // when
        let burst = next_burst(&received, Duration::from_millis(50), |_| false);

        // then
        assert_eq!(burst, None);
    }
}
