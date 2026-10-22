//! The built-in path actions of menus and bindings: copying paths, a terminal in a folder, the OS's file manager,
//! default application and browser, a file in a window of its own.

use std::path::Path;

use gpui::{ClipboardItem, Context, Window};

use super::Workspace;
use crate::menu::resolve::{Applies, Choice};
use crate::menu::{PathKind, Subject, SubjectPath, Within};
use crate::os_open;
use crate::project::CheckoutRef;
use crate::script::keymap::Builtin;

/// Files the browser shows.
const WEB_EXTENSIONS: [&str; 3] = ["html", "htm", "svg"];

/// What path action `builtin` does for `subject`; `None` when `builtin` is not one of them.
pub fn path_applies(builtin: Builtin, subject: &Subject) -> Option<Applies> {
    let paths = &subject.paths;
    let one = |accept: fn(&SubjectPath) -> bool| matches!(&paths[..], [only] if accept(only));
    let shown = match builtin {
        Builtin::CopyFullPath | Builtin::CopyRelativePath | Builtin::CopyFileName | Builtin::FindInFiles => !paths.is_empty(),
        Builtin::OpenTerminalHere | Builtin::RevealInFileManager => one(|_| true),
        Builtin::OpenInNewWindow => one(|path| path.kind == PathKind::File),
        Builtin::OpenInBrowser => one(is_web_file),
        Builtin::OpenWithDefaultApp => !paths.is_empty() && paths.iter().all(|path| path.kind == PathKind::File),
        _ => return None,
    };
    Some(if shown { Applies::Shown { enabled: true } } else { Applies::Hidden })
}

/// What Run does for `subject`: a shell file runs; a folder lists the configurations `configs` gives for it, as
/// `(key, name)`; anything else hides it.
pub fn run_applies(subject: &Subject, configs: impl Fn(&Path) -> Vec<(String, String)>) -> Applies {
    match &subject.paths[..] {
        [file] if file.kind == PathKind::File && crate::scratch::run::shell_for(&file.path.to_string_lossy()).is_some() => Applies::Shown { enabled: true },
        [folder] if folder.kind == PathKind::Folder => {
            let choices: Vec<Choice> = configs(&folder.path).into_iter().map(|(key, name)| Choice::Item { label: name, choice: key, enabled: true }).collect();
            if choices.is_empty() { Applies::Hidden } else { Applies::Choices(choices) }
        }
        _ => Applies::Hidden,
    }
}

/// Whether `builtin` is a path action, which runs on a subject.
pub fn is_path_action(builtin: Builtin) -> bool {
    builtin == Builtin::SendPathsToTerminal || path_applies(builtin, &Subject::empty()).is_some()
}

/// What a Copy action puts on the clipboard: one path per line, in the subject's order.
pub fn copied_text(builtin: Builtin, subject: &Subject) -> String {
    let line = |path: &SubjectPath| match (builtin, &path.within) {
        (Builtin::CopyRelativePath, Within::Checkout(relative)) => relative.clone(),
        (Builtin::CopyFileName, _) => path.path.file_name().map_or_else(|| path.path.display().to_string(), |name| name.to_string_lossy().into_owned()),
        _ => path.path.display().to_string(),
    };
    subject.paths.iter().map(line).collect::<Vec<_>>().join("\n")
}

fn is_web_file(path: &SubjectPath) -> bool {
    let extension = path.path.extension().map(|extension| extension.to_string_lossy().to_lowercase());
    path.kind == PathKind::File && extension.is_some_and(|extension| WEB_EXTENSIONS.contains(&extension.as_str()))
}

impl Workspace {
    /// Runs path action `builtin` on `subject`; one that does not apply to it does nothing.
    pub(super) fn run_path_action(&mut self, builtin: Builtin, subject: &Subject, window: &mut Window, cx: &mut Context<Self>) {
        if !matches!(path_applies(builtin, subject), Some(Applies::Shown { enabled: true })) {
            return;
        }
        let first = &subject.paths[0];
        match builtin {
            Builtin::CopyFullPath | Builtin::CopyRelativePath | Builtin::CopyFileName => cx.write_to_clipboard(ClipboardItem::new_string(copied_text(builtin, subject))),
            Builtin::OpenTerminalHere => self.open_terminal_here(first, window, cx),
            Builtin::RevealInFileManager => os_open::start(os_open::reveal(&first.path)),
            Builtin::OpenWithDefaultApp => subject.paths.iter().for_each(|path| os_open::start(vec![os_open::default_app(&path.path)])),
            Builtin::OpenInBrowser => os_open::start(vec![os_open::browser(&first.path)]),
            Builtin::OpenInNewWindow => self.open_in_new_window(&first.path, window, cx),
            Builtin::FindInFiles => {
                let paths = subject.paths.iter().map(|path| path.path.clone()).collect();
                self.open_hub(Some(crate::hub::model::HubState { content: true, paths, ..Default::default() }), window, cx);
            }
            _ => {}
        }
    }

    /// A terminal in the folder, or the folder of a file, listed under the checkout that holds it.
    fn open_terminal_here(&mut self, path: &SubjectPath, window: &mut Window, cx: &mut Context<Self>) {
        let directory = match path.kind {
            PathKind::Folder => path.path.as_path(),
            PathKind::File => path.path.parent().unwrap_or(&path.path),
        };
        let checkout = self.checkout_ref_of(directory);
        self.open_terminal(checkout, directory, Vec::new(), window, cx);
    }

    fn open_in_new_window(&mut self, path: &Path, window: &mut Window, cx: &mut Context<Self>) {
        let opened = self.open_file(path, window, cx).and_then(|tab| self.detach_tab_by_request(tab, window, cx));
        if let Err(error) = opened {
            eprintln!("ive: cannot open {} in a window of its own: {error}", path.display());
        }
    }

    /// What Run does for `subject`, with the configurations of the `.run.toml` files IVE has read.
    pub(super) fn run_file_applies(&self, subject: &Subject) -> Applies {
        run_applies(subject, |folder| {
            let file = self.checkout_ref_of(folder).and_then(|checkout| self.run_file_of(&checkout, folder));
            file.map(|file| file.configs.iter().map(|config| (config.key.clone(), config.name.clone())).collect()).unwrap_or_default()
        })
    }

    /// Runs a shell file of `subject`, or starts configuration `key` of its folder's `.run.toml`.
    pub(super) fn run_from_menu(&mut self, key: Option<&str>, subject: &Subject, window: &mut Window, cx: &mut Context<Self>) {
        let [path] = &subject.paths[..] else {
            return;
        };
        let checkout = self.checkout_ref_of(&path.path);
        let started = match (key, checkout) {
            (Some(key), Some(checkout)) => {
                let request = crate::run::values_tab::RunRequest { checkout, folder: path.path.clone(), key: key.to_owned(), show: crate::run::values_tab::ShowRun::Tab };
                self.request_run(request, window, cx);
                Ok(())
            }
            (None, checkout) => self.run_shell_file(&path.path, checkout, window, cx).map(drop),
            (Some(_), None) => Err(format!("{} is in no open project", path.path.display())),
        };
        if let Err(error) = started {
            eprintln!("ive: cannot run {}: {error}", path.path.display());
        }
    }

    /// The innermost open checkout holding `path`.
    pub(super) fn checkout_ref_of(&self, path: &Path) -> Option<CheckoutRef> {
        let holding = self.projects.iter().filter_map(|open| open.project.checkout_containing(path).map(|checkout| open.project.checkout_ref(checkout)));
        holding.max_by_key(|checkout| checkout.path.components().count())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::menu::resolve::Choice;
    use crate::menu::{Home, SubjectPath};

    fn path(path: &str, within: Within, kind: PathKind) -> SubjectPath {
        SubjectPath { path: PathBuf::from(path), within, kind }
    }

    fn subject(paths: Vec<SubjectPath>) -> Subject {
        Subject { paths, home: Home::Mixed }
    }

    fn file(at: &str) -> SubjectPath {
        path(&format!("/w/{at}"), Within::Checkout(at.to_owned()), PathKind::File)
    }

    fn shown(applies: Option<Applies>) -> bool {
        matches!(applies, Some(Applies::Shown { enabled: true }))
    }

    #[test]
    fn should_copy_relative_paths_one_per_line() {
        // given
        let paths = subject(vec![file("src/a.rs"), path("/tmp/x", Within::Outside, PathKind::File)]);

        // when
        let text = copied_text(Builtin::CopyRelativePath, &paths);

        // then
        assert_eq!(text, "src/a.rs\n/tmp/x");
    }

    #[test]
    fn should_copy_file_names_and_absolute_paths() {
        // given
        let paths = subject(vec![file("src/a.rs"), file("README.md")]);

        // when
        let copied = [copied_text(Builtin::CopyFileName, &paths), copied_text(Builtin::CopyFullPath, &paths)];

        // then
        assert_eq!(copied, ["a.rs\nREADME.md", "/w/src/a.rs\n/w/README.md"]);
    }

    #[test]
    fn should_offer_default_app_only_without_folders() {
        // given
        let files = subject(vec![file("a.png"), file("b.png")]);
        let with_folder = subject(vec![file("a.png"), path("/w/src", Within::Checkout("src".to_owned()), PathKind::Folder)]);

        // when
        let offered = [shown(path_applies(Builtin::OpenWithDefaultApp, &files)), shown(path_applies(Builtin::OpenWithDefaultApp, &with_folder))];

        // then
        assert_eq!(offered, [true, false]);
    }

    #[test]
    fn should_offer_the_browser_for_one_web_file() {
        // when
        let offered = [
            shown(path_applies(Builtin::OpenInBrowser, &subject(vec![file("index.html")]))),
            shown(path_applies(Builtin::OpenInBrowser, &subject(vec![file("logo.SVG")]))),
            shown(path_applies(Builtin::OpenInBrowser, &subject(vec![file("a.rs")]))),
            shown(path_applies(Builtin::OpenInBrowser, &subject(vec![file("a.html"), file("b.html")]))),
        ];

        // then
        assert_eq!(offered, [true, true, false, false]);
    }

    #[test]
    fn should_offer_one_path_actions_for_exactly_one_path() {
        // given
        let (one, two) = (subject(vec![file("a.rs")]), subject(vec![file("a.rs"), file("b.rs")]));

        // when
        let offered = [Builtin::OpenTerminalHere, Builtin::RevealInFileManager, Builtin::OpenInNewWindow].map(|builtin| (shown(path_applies(builtin, &one)), shown(path_applies(builtin, &two))));

        // then
        assert_eq!(offered, [(true, false); 3]);
    }

    #[test]
    fn should_offer_a_new_window_only_for_a_file() {
        // given
        let folder = subject(vec![path("/w/src", Within::Checkout("src".to_owned()), PathKind::Folder)]);

        // when
        let offered = shown(path_applies(Builtin::OpenInNewWindow, &folder));

        // then
        assert!(!offered);
    }

    #[test]
    fn should_hide_path_actions_without_paths() {
        // when
        let offered = [Builtin::CopyRelativePath, Builtin::CopyFileName, Builtin::OpenTerminalHere].map(|builtin| path_applies(builtin, &subject(Vec::new())));

        // then
        assert_eq!(offered, [Some(Applies::Hidden), Some(Applies::Hidden), Some(Applies::Hidden)]);
    }

    fn configs(folder: &std::path::Path) -> Vec<(String, String)> {
        match folder.to_str() {
            Some("/w/app") => vec![("dev".to_owned(), "Dev server".to_owned()), ("test".to_owned(), "Tests".to_owned())],
            _ => Vec::new(),
        }
    }

    #[test]
    fn should_offer_run_only_for_shell_files_and_run_folders() {
        // given
        let app = path("/w/app", Within::Checkout("app".to_owned()), PathKind::Folder);
        let docs = path("/w/docs", Within::Checkout("docs".to_owned()), PathKind::Folder);

        // when
        let offered = [
            run_applies(&subject(vec![file("build.sh")]), configs),
            run_applies(&subject(vec![file("deploy.ZSH")]), configs),
            run_applies(&subject(vec![file("a.rs")]), configs),
            run_applies(&subject(vec![file("a.sh"), file("b.sh")]), configs),
            run_applies(&subject(vec![docs]), configs),
        ];

        // then
        assert_eq!(offered, [Applies::Shown { enabled: true }, Applies::Shown { enabled: true }, Applies::Hidden, Applies::Hidden, Applies::Hidden]);
        let Applies::Choices(choices) = run_applies(&subject(vec![app]), configs) else { panic!() };
        assert_eq!(choices[1], Choice::Item { label: "Tests".to_owned(), choice: "test".to_owned(), enabled: true });
    }

    #[test]
    fn should_leave_other_actions_to_others() {
        // when
        let applies = path_applies(Builtin::GitHistory, &subject(vec![file("a.rs")]));

        // then
        assert_eq!(applies, None);
    }
}
