//! What codeview asks of the program that embeds it: what keys mean and the clipboard.

use std::rc::Rc;

use gpui::{AnyElement, App, ClickEvent, Global, KeyDownEvent, Window};

use crate::diff::Mark;
use crate::text_editor::{Motion, Selecting, Unit};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardTarget {
    Clipboard,
    Primary,
    Both,
}

/// What a key does in a view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Move(Motion, Selecting),
    SelectAll,
    Copy(ClipboardTarget),
    Cut,
    Paste(ClipboardTarget),
    Undo,
    Redo,
    Backspace(Unit),
    Delete(Unit),
    Newline,
    Indent,
    /// saves unsaved edits now instead of after the pause
    Flush,
}

/// What the host made of a key press.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyOutcome {
    /// run it in the view
    Command(Command),
    /// the host took the key (ran a bound function, or bound it to nothing): the view does nothing and the key stops
    Handled,
    /// bound to something only the host can run: the view does nothing and the key goes on to the host
    Foreign,
    /// no binding: the view reads the key as it would by default
    Unbound,
}

/// Which kind of panel asks what a key means.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyScope {
    /// a text editor: the host runs the functions and strings bound to the key itself
    Editor,
    /// a read-only view (diff, file): it only takes Copy and Select All, every other bound key goes on to the host
    Viewer,
}

pub type KeyFn = Rc<dyn Fn(&KeyDownEvent, KeyScope, &App) -> KeyOutcome>;
pub type ClipboardFn = Rc<dyn Fn(ClipboardTarget, String, &mut App)>;
pub type PasteFn = Rc<dyn Fn(ClipboardTarget, &mut App) -> Option<String>>;

/// A click handler that can be shared: by an element and by the host's registry of click targets.
pub type ClickListener = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;
pub type MarkFn = Rc<dyn Fn(Mark, &App) -> AnyElement>;
pub type TargetFn = Rc<dyn Fn(String, ClickListener, &App)>;

/// The host's services, a gpui global. Without one, no key is bound and the clipboard does nothing.
pub struct Host {
    pub keys: KeyFn,
    pub clipboard: ClipboardFn,
    pub paste: PasteFn,
    /// how a diff header shows a file's [`Mark`]
    pub diff_mark: MarkFn,
    /// registers a click handler under a name, so the host can click it by name (IVE's MCP `click`)
    pub click_target: TargetFn,
}

impl Global for Host {}

impl Host {
    pub fn key_outcome(event: &KeyDownEvent, scope: KeyScope, cx: &App) -> KeyOutcome {
        match cx.try_global::<Host>() {
            Some(host) => (host.keys.clone())(event, scope, cx),
            None => KeyOutcome::Unbound,
        }
    }

    pub fn write_clipboard(target: ClipboardTarget, text: String, cx: &mut App) {
        if let Some(write) = cx.try_global::<Host>().map(|host| host.clipboard.clone()) {
            write(target, text, cx);
        }
    }

    pub fn read_clipboard(source: ClipboardTarget, cx: &mut App) -> Option<String> {
        let read = cx.try_global::<Host>().map(|host| host.paste.clone())?;
        read(source, cx)
    }

    /// The host's drawing of `mark`; nothing without a host.
    pub fn diff_mark(mark: Mark, cx: &App) -> Option<AnyElement> {
        cx.try_global::<Host>().map(|host| (host.diff_mark.clone())(mark, cx))
    }

    /// Registers `listener` with the host as click target `name` and returns it, for `.on_click(…)`.
    pub fn click_target(
        name: String,
        listener: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
        cx: &App,
    ) -> impl Fn(&ClickEvent, &mut Window, &mut App) + 'static {
        let listener: ClickListener = Rc::new(listener);
        if let Some(register) = cx.try_global::<Host>().map(|host| host.click_target.clone()) {
            register(name, listener.clone(), cx);
        }
        move |event, window, cx| listener(event, window, cx)
    }
}
