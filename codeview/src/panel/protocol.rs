//! The messages between a panel and its host: casts in, events out, and the data they carry.

use std::borrow::Cow;
use std::ops::Range;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tree_sitter::InputEdit;

use super::state::PanelState;
use crate::changes::{ChangeBase, ChangeDiffJob, ChangeHunks};
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
    /// a diff reloaded: keeps the scroll and the collapsed files; ignored while the new side is edited, whose rows
    /// show only diffs of the edited text's base ([`Cast::SetEditBase`], [`Cast::SetBaseDiff`])
    SetFiles(Vec<FileDiff>),
    /// the answer to [`Event::NeedTexts`] with [`TextPurpose::Colours`], with its `generation`: an answer to an older
    /// ask is dropped
    SetTexts { file: usize, generation: u64, old: Option<Source>, new: Option<Source> },
    /// the answer to [`Event::NeedTexts`] with [`TextPurpose::Editing`], with its `generation`: an answer to an older
    /// ask is dropped
    SetEditBase { generation: u64, base: EditBase },
    /// while the new side is edited: `diff`, the host's diff of `base`, the text the edits apply to (taken with
    /// [`Cast::SetEditBase`] or [`Cast::Saved`]); dropped when they apply to another text by then
    SetBaseDiff { base: Arc<dyn LineSource>, diff: FileDiff },
    /// the answer to [`Event::NeedParse`]; in View mode one for an older revision than the panel's is dropped, in
    /// Edit mode it is shown moved by the edits made since
    Parsed { target: ParseTarget, revision: u64, syntax: Syntax },
    /// the answer to [`Event::SaveRequested`]: `source` is the text as saved, read again (an in-place write may have
    /// moved what the old one's line index points at)
    Saved { revision: u64, source: Arc<dyn LineSource> },
    /// the edits stay; the bar says why, and the next pause or flush tries again
    SaveFailed { revision: u64, reason: SaveFailure },
    /// the host would close the tab or quit but for the unsaved edits: while the last save has failed, the bar offers
    /// closing without saving; ends with a save, an edit or [`Cast::CloseWithdrawn`]
    CloseAsked,
    /// the host no longer waits to close (the user went on working): the bar stops offering closing without saving
    CloseWithdrawn,
    Command(Command),
    /// types the text at the caret as the keyboard would, `\r` a line break; refused over the size limit
    Type(String),
    /// scrolls line `line` (from 1) into view and marks it; a diff scrolls the hunk holding its first file's new-side
    /// line `line` to the top
    RevealLine(usize),
    /// as [`Cast::RevealLine`], and selects `bytes` of the line and scrolls sideways to them; a diff reveals the line
    RevealRange { line: usize, bytes: Range<usize> },
    /// the file at this path went away on disk; its text stays
    MarkDeleted(String),
    SetPolicy(EditPolicy),
    /// the answer to [`Event::NeedChangeBase`], or a new base after HEAD moved: what the change markers compare the
    /// edited text with; the same HEAD again changes nothing
    SetChangeBase { base: ChangeBase },
    /// the answer to [`Event::NeedChangeDiff`]: shown moved by the edits made since; dropped when it is not the diff
    /// in flight, or its text or base was replaced since
    ChangeDiff { revision: u64, hunks: ChangeHunks },
    /// the answer to [`Event::CommitChange`]: the change is committed; the popup closes, and the markers follow the
    /// new HEAD once its base comes
    ChangeCommitted,
    /// the answer to [`Event::CommitChange`]: nothing was committed, for the reason given (for the user); the popup
    /// shows it and keeps the message
    ChangeCommitFailed(String),
    /// the answer to [`Event::CommitChange`] when committing went wrong in a way the user must see whatever HEAD does
    /// next: a bar above the text says `report` until dismissed or a commit goes through; with [`CommitLeft::Commit`]
    /// the popup closes as on [`Cast::ChangeCommitted`], with [`CommitLeft::Nothing`] it stays with the message
    ChangeCommitTrouble { report: String, left: CommitLeft },
    /// a diff's: [`Editable::NewSide`] edits the new side once its text comes, asked for with
    /// [`TextPurpose::Editing`]; [`Editable::No`] makes it read-only, dropping the edited text and its history (the
    /// host sends it only while nothing is unsaved)
    SetEditable(Editable),
}

/// What a troubled commit left in the repository.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommitLeft {
    /// a commit was made and stays
    Commit,
    /// nothing is committed
    Nothing,
}

/// A diff's new side to edit, read for an ask with [`TextPurpose::Editing`].
pub enum EditBase {
    /// `new`, the text to edit, with the old side's text (for colours) and `diff`, the host's diff of the two, which
    /// the rows show
    Text { old: Option<Source>, new: Arc<dyn LineSource>, diff: FileDiff },
    /// the new side cannot be edited: the diff stays read-only
    Unavailable,
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
    /// until a [`Cast::Restore`] brings the tab's own
    pub layout: DiffLayout,
    pub editable: Editable,
}

/// Whether a diff edits its new side: only a single file's uncommitted changes do, saved as `policy` says.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParseTarget {
    Text,
    /// the old side of diff file `n`
    DiffOld(usize),
    DiffNew(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveFailure {
    /// the file changed on disk since the last save; what the user may do about it
    Conflict(Resolve),
    Io(String),
}

/// What the user may do about a conflict.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resolve {
    /// read the file again, dropping the edits, or write the edits over it
    ReloadOrOverwrite,
    /// only read it again: the text the edits apply to may be gone, so writing them over it would garble it
    Reload,
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
    /// the host runs the job on a background thread and answers [`Cast::Parsed`], also when the text changed
    /// meanwhile; only a set [`ParseJob::cancel`] makes the answer unwanted
    NeedParse(ParseJob),
    /// a diff file came on screen: the host answers [`Cast::SetTexts`]
    NeedTexts(TextsRequest),
    /// the host writes the edits and answers [`Cast::Saved`] or [`Cast::SaveFailed`]; one at a time
    SaveRequested(SaveRequest),
    /// the user chose Reload on a conflict: the host opens the text again, dropping the edits
    ReloadRequested,
    /// the user chose closing without saving after [`Cast::CloseAsked`]: the panel dropped its edits (nothing is
    /// saved any more) and the host closes, or opens the text again where the tab stays
    DiscardRequested,
    OpenAt { path: PathBuf, line: usize },
    /// the user asked for the edited file's uncommitted diff at its line `line` (from 1), from a change's popup; the
    /// host knows the file. Unsaved edits were sent to be saved just before.
    OpenDiff { line: usize },
    /// an edited text opened: the host answers [`Cast::SetChangeBase`] when it knows the file's base, and casts it
    /// again when HEAD moves
    NeedChangeBase,
    /// the host runs the job on a background thread and answers [`Cast::ChangeDiff`]
    NeedChangeDiff(ChangeDiffJob),
    /// the user asked to commit one change of the edited file, and nothing else: `text` is the whole file to commit
    /// (HEAD's text, as the base was, with only base lines `block` replaced by the change's current lines, unsaved
    /// edits included), `head_id` the base's HEAD. The host commits it on that HEAD, or refuses, and answers
    /// [`Cast::ChangeCommitted`] or [`Cast::ChangeCommitFailed`]
    CommitChange { head_id: String, block: Range<usize>, text: Vec<u8>, message: String },
}

/// An ask for the texts of diff file `file`; each ask has a new `generation`, which its answer carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextsRequest {
    pub file: usize,
    pub generation: u64,
    pub purpose: TextPurpose,
}

/// What a diff asks a file's texts for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextPurpose {
    Colours,
    /// also to edit the new side: the host reads the file on disk, diffs that very text, and checks the saves
    /// against what it read, as for a file it opened for editing; it answers [`Cast::SetEditBase`]
    Editing,
}

/// `edits` turn the text of revision `base` into that of `revision`: sorted, non-overlapping, in `base`'s bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveRequest {
    pub base: u64,
    pub revision: u64,
    pub edits: Vec<Edit>,
    pub kind: SaveKind,
}

/// What a save may write over.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveKind {
    /// only the file as last read or saved; anything else is a [`SaveFailure::Conflict`]
    Checked,
    /// whatever is there: the user chose Overwrite on a conflict
    Overwrite,
}

/// One change to a saved text: `bytes` of it, as of the save's base revision, become `text`, stored as given (not
/// necessarily UTF-8: bytes restored by an undo are written back exactly).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edit {
    pub bytes: Range<u64>,
    pub text: Vec<u8>,
}

impl Edit {
    /// How much longer the text gets; negative when it shrinks.
    pub fn growth(&self) -> i64 {
        self.text.len() as i64 - (self.bytes.end - self.bytes.start) as i64
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DiffLayout {
    #[default]
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
    /// Byte offset where line `line` starts; past the last line, `byte_len`.
    fn line_start(&self, line: usize) -> u64;
    /// The line byte offset `byte` (clamped) is on: the number of newlines before it.
    fn line_of(&self, byte: u64) -> usize;
}

/// A parse to run off the UI thread; see [`Event::NeedParse`].
#[derive(Clone)]
pub struct ParseJob {
    pub(crate) source: Source,
    pub(crate) language: Language,
    pub(crate) cancel: Cancel,
    pub(crate) target: ParseTarget,
    pub(crate) revision: u64,
    pub(crate) previous: Previous,
    pub(crate) failed: Failed,
}

/// Set when a parse ended without a result, so a panel waiting for it asks again instead.
#[derive(Clone, Default)]
pub(crate) struct Failed(Arc<AtomicBool>);

impl Failed {
    pub(crate) fn is_set(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }

    pub(crate) fn set(&self) {
        self.0.store(true, Ordering::Relaxed);
    }
}

/// What a parse may reuse.
#[derive(Clone)]
pub(crate) enum Previous {
    Nothing,
    /// the parse of an older text, which `edits` turned into this one
    Edited { syntax: Syntax, edits: Vec<InputEdit> },
}

impl ParseJob {
    pub fn target(&self) -> ParseTarget {
        self.target
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Set once the panel no longer wants the result: it went away or its text was replaced.
    pub fn cancel(&self) -> Cancel {
        self.cancel.clone()
    }

    pub fn run(self) -> Result<Syntax, ParseError> {
        let source = self.source.as_bytes();
        let result = match &self.previous {
            Previous::Nothing => syntax::parse(source, self.language, &self.cancel, PARSE_BUDGET),
            Previous::Edited { syntax, edits } => syntax::reparse(syntax, edits, source, &self.cancel, PARSE_BUDGET),
        };
        if result.is_err() {
            self.failed.set();
        }
        result
    }
}
