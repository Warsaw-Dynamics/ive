//! The messages between a panel and its host: casts in, events out, and the data they carry.

use std::borrow::Cow;
use std::ops::Range;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::state::PanelState;
use crate::diff::{FileDiff, Headers};
use crate::document::{DisplayLine, Source};
use crate::host::Command;
use crate::syntax::{self, Cancel, Language, PARSE_BUDGET, ParseError, Syntax};
use crate::text_editor::SizeLimit;

/// What the host tells a panel; the panel answers with events, if at all.
pub enum Cast {
    Open(Content),
    /// held until the content is known, then clamped to it and applied
    Restore(PanelState),
    /// a diff reloaded: keeps the scroll and the collapsed files
    SetFiles(Vec<FileDiff>),
    /// the answer to [`Event::NeedTexts`]
    SetTexts { file: usize, old: Option<Source>, new: Option<Source> },
    /// the answer to [`Event::NeedParse`]; one for an older revision than the panel's is dropped
    Parsed { target: ParseTarget, revision: u64, syntax: Syntax },
    /// the answer to [`Event::SaveRequested`]; `source` is the rewritten file, when it was rewritten
    Saved { revision: u64, source: Option<Arc<dyn LineSource>> },
    SaveFailed { revision: u64, reason: SaveFailure },
    Command(Command),
    /// scrolls line `line` (from 1) into view and marks it
    RevealLine(usize),
    /// the file went away on disk; its text stays
    MarkDeleted,
    SetPolicy(EditPolicy),
}

/// What a panel shows.
pub enum Content {
    /// a file or a scratch, viewed or with `edit`, edited
    Lines { source: Arc<dyn LineSource>, language: Option<Language>, edit: Option<EditPolicy> },
    Diff(DiffInput),
}

pub struct DiffInput {
    pub files: Vec<FileDiff>,
    pub headers: Headers,
    pub editable: Editable,
}

pub enum Editable {
    No,
    NewSide { policy: EditPolicy },
}

/// How the host wants edits saved; it knows what a write costs. The panel runs the timers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EditPolicy {
    /// the pause after the last edit before [`Event::SaveRequested`]
    pub save_delay: Duration,
    /// scratches keep their size limit; files have none
    pub limit: Option<SizeLimit>,
}

/// Which text a parse is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseTarget {
    Text,
    /// the old side of diff file `n`
    DiffOld(usize),
    DiffNew(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveFailure {
    /// the file changed on disk since the last save
    Conflict,
    Io(String),
}

/// What a panel tells its host.
pub enum Event {
    /// the text changed; it is unsaved until [`Cast::Saved`] for `revision`
    Edited { revision: u64 },
    /// an edit was refused (over the size limit); the reason is for the user
    Refused(String),
    /// the scroll, caret, layout or collapsed files changed: [`super::Panel::state`] has the new state
    StateChanged,
    LayoutToggled(DiffLayout),
    /// the host runs the job on a background thread and answers [`Cast::Parsed`]
    NeedParse(ParseJob),
    /// a diff file came on screen: the host answers [`Cast::SetTexts`]
    NeedTexts { file: usize },
    /// `edits` turn the text of revision `base` into that of `revision`
    SaveRequested { base: u64, revision: u64, edits: Vec<Edit> },
    OpenAt { path: PathBuf, line: usize },
}

/// One change to a saved text: `bytes` of it, as of the save's base revision, become `text`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edit {
    pub bytes: Range<u64>,
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DiffLayout {
    Inline,
    SideBySide,
}

/// Where a panel reads its lines; the host owns the text. Ranges only: a file may have 100M lines.
pub trait LineSource: Send + Sync {
    fn line_count(&self) -> usize;
    /// Lines `range` (clamped), decoded for display: tabs expanded, CRLF dropped, cut at `MAX_LINE_CHARS`.
    fn lines(&self, range: Range<usize>) -> Vec<DisplayLine>;
    /// The whole text when it is small enough to highlight (`MAX_HIGHLIGHT_BYTES`).
    fn source(&self) -> Option<Source>;
    /// Bytes `range` (clamped) as they are stored.
    fn bytes(&self, range: Range<u64>) -> Cow<'_, [u8]>;
    fn byte_len(&self) -> u64;
}

/// A parse to run off the UI thread; see [`Event::NeedParse`].
#[derive(Clone)]
pub struct ParseJob {
    pub(crate) source: Source,
    pub(crate) language: Language,
    pub(crate) cancel: Cancel,
    pub(crate) target: ParseTarget,
    pub(crate) revision: u64,
}

impl ParseJob {
    pub fn target(&self) -> ParseTarget {
        self.target
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Set once the panel no longer wants the result: it went away or its text changed.
    pub fn cancel(&self) -> Cancel {
        self.cancel.clone()
    }

    pub fn run(self) -> Result<Syntax, ParseError> {
        syntax::parse(self.source.as_bytes(), self.language, &self.cancel, PARSE_BUDGET)
    }
}
