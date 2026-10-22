//! What codeview asks of the program that embeds it: what keys mean and the clipboard.

use std::rc::Rc;

use gpui::{App, Global, KeyDownEvent};

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

pub type KeyFn = Rc<dyn Fn(&KeyDownEvent, &App) -> KeyOutcome>;
pub type ClipboardFn = Rc<dyn Fn(ClipboardTarget, String, &mut App)>;
pub type PasteFn = Rc<dyn Fn(ClipboardTarget, &mut App) -> Option<String>>;

/// The host's services, a gpui global. Without one, no key is bound and the clipboard does nothing.
pub struct Host {
    pub keys: KeyFn,
    pub clipboard: ClipboardFn,
    pub paste: PasteFn,
}

impl Global for Host {}

impl Host {
    pub fn key_outcome(event: &KeyDownEvent, cx: &App) -> KeyOutcome {
        match cx.try_global::<Host>() {
            Some(host) => (host.keys.clone())(event, cx),
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
}
