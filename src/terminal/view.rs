use std::cell::Cell;
use std::io;
use std::ops::Range;
use std::path::Path;
use std::process::ExitStatus;
use std::rc::Rc;
use std::time::{Duration, Instant};

use alacritty_terminal::Term;
use alacritty_terminal::event::{Event, EventListener};
use alacritty_terminal::grid::Dimensions;
use alacritty_terminal::index::{Column, Line, Point as GridPoint, Side};
use alacritty_terminal::selection::SelectionType;
use alacritty_terminal::term::TermMode;
use alacritty_terminal::vte::ansi::CursorShape;
use futures::StreamExt;
use futures::channel::mpsc::UnboundedReceiver;
use gpui::{
    App, Bounds, BorderStyle, ClipboardItem, ContentMask, Context, CursorStyle, ExternalPaths, FocusHandle, Font, FontStyle, Hsla,
    KeyDownEvent, Keystroke, Modifiers, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, Point, ScrollWheelEvent,
    SharedString, Size,
    StrikethroughStyle, Task, TextRun, WeakEntity, UnderlineStyle, Window, canvas, div, fill, outline, point, prelude::*,
    rgb, size,
};

use super::colors::{ColorResolver, unpack};
use super::input::{KeyAction, key_action};
use super::launch::Launch;
use super::layout::{CursorMark, Frame, Run, Segment, Underline, Window as GridWindow, columns_shift, layout, layout_window, top_showing};
use super::mouse::{self, Action, Button, Gesture, GestureKind, MouseEvent, ScreenCell};
use super::progress::Progress;
use super::session::{GridSize, Session};
use super::text::{ScreenText, output_point, output_rows, output_text, screen_text};
#[cfg(unix)]
use super::hosted::HostedTerminals;
#[cfg(unix)]
use ptyhost::registry::SessionInfo;
use crate::script::keymap::Place;
use crate::clipboard;
use crate::dragged_files::DraggedFiles;
use crate::pasted;
use crate::script::keymap::{self, Builtin, ClipboardTarget, PasteKind};
use crate::script::keymap::{Keymap, MouseKind, MouseTrigger, Scope, SelectionMode};
use crate::script::state::{bound_action, invoke, keymap};
use crate::process::ProcessId;
use crate::theme::{self, TerminalPalette, Theme};
use crate::theme_config::TerminalSizes;
use crate::workspace::Workspace;

/// A shell that fails within this long after start most likely could not start at all.
const QUICK_EXIT: Duration = Duration::from_secs(2);

/// What the workspace does with the process after its events are handled.
pub enum TerminalState {
    Running,
    Closed,
    Failed(SharedString),
    /// a run's program exited with this code; the run stays, with its output
    Exited(i32),
}

/// Where a terminal's shell comes from.
pub enum Source<'a> {
    /// a PTY of IVE's own; the program ends with IVE
    InProcess(Launch),
    /// a new host, labelled with `labels`; the program outlives IVE
    #[cfg(unix)]
    Hosted { terminals: &'a HostedTerminals, launch: Launch, labels: Vec<(String, String)> },
    /// a host an earlier IVE started
    #[cfg(unix)]
    Adopt { terminals: &'a HostedTerminals, session: &'a SessionInfo },
    #[cfg(not(unix))]
    #[allow(dead_code)]
    Never(std::marker::PhantomData<&'a ()>),
}

/// Whether IVE saw the shell start, which tells whether a quick exit means it could not start.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Start {
    New,
    Adopted,
}

/// A shell session, drawn cell by cell; shown by a process tab.
pub struct TerminalView {
    process: ProcessId,
    session: Rc<Session>,
    focus_handle: FocusHandle,
    title: Option<String>,
    started: Instant,
    start: Start,
    exit_status: Option<ExitStatus>,
    /// a run: its exit keeps the view, rather than closing it
    keeps_after_exit: bool,
    pointer: Rc<Pointer>,
    _events: Task<()>,
}

impl TerminalView {
    /// Starts a shell in `working_directory` for process `process`, or attaches to one, and focuses the terminal.
    pub fn open(process: ProcessId, working_directory: &Path, source: Source, window: &mut Window, cx: &mut Context<Workspace>) -> io::Result<Self> {
        let (start, (session, events)) = match source {
            Source::InProcess(launch) => (Start::New, Session::spawn(working_directory, &launch)?),
            #[cfg(unix)]
            Source::Hosted { terminals, launch, labels } => (Start::New, Session::spawn_hosted(terminals, working_directory, &launch, labels)?),
            #[cfg(unix)]
            Source::Adopt { terminals, session } => (Start::Adopted, Session::attach(terminals, session)?),
            #[cfg(not(unix))]
            Source::Never(_) => unreachable!("no hosted terminals outside Unix"),
        };
        let focus_handle = cx.focus_handle();
        window.focus(&focus_handle, cx);
        Ok(Self {
            process,
            session: Rc::new(session),
            focus_handle,
            title: None,
            started: Instant::now(),
            start,
            exit_status: None,
            keeps_after_exit: false,
            pointer: Rc::new(Pointer { origin: Some((cx.entity().downgrade(), process)), ..Pointer::default() }),
            _events: forward_events(process, events, window, cx),
        })
    }

    /// Makes the program's exit keep the view and its output, as a run's does.
    pub fn keep_after_exit(&mut self) {
        self.keeps_after_exit = true;
    }

    /// The program the terminal started, its process group's leader.
    pub fn program_pid(&self) -> Option<u32> {
        self.session.program_pid()
    }

    /// Ends the shell, hosted or not, as closing its terminal does.
    pub fn hang_up(&self) {
        self.session.hang_up();
    }

    /// True when the shell runs in a host and outlives IVE.
    pub fn is_hosted(&self) -> bool {
        self.session.hosted_id().is_some()
    }

    /// The host's session id, for a shell that outlives IVE.
    pub fn hosted_id(&self) -> Option<&str> {
        self.session.hosted_id()
    }

    /// The title the shell set, as IntelliJ shows it on terminal tabs.
    pub fn title(&self) -> String {
        self.title.clone().unwrap_or_else(|| "Terminal".to_owned())
    }

    pub fn progress(&self) -> Progress {
        self.session.progress()
    }

    /// What the program last reported as its working directory, for following it between checkouts.
    pub fn directory(&self) -> Option<super::DirectoryReport> {
        self.session.directory()
    }

    /// Which program has the terminal, for Claude Code detection.
    #[cfg(unix)]
    pub fn foreground(&self) -> Option<super::ForegroundProbe> {
        self.session.foreground().cloned()
    }

    /// Tells the program the theme turned `appearance`, if it asked to hear of that.
    pub fn report_theme(&self, appearance: theme::Appearance) {
        self.session.report_theme(appearance);
    }

    pub fn focus_handle(&self) -> FocusHandle {
        self.focus_handle.clone()
    }

    /// Pastes `text`, bracketed when the program asked for that, as a drop does.
    pub fn paste(&self, text: &str) {
        self.session.paste(text);
    }

    /// Types `text` into the shell as it is; a trailing `\r` presses Enter.
    pub fn send_text(&self, text: &str) {
        self.session.type_bytes(text.as_bytes().to_vec());
    }

    /// Presses `keys` in order, each written like a key binding: `enter`, `ctrl-c`, `alt-b`, `up`.
    pub fn send_keys(&self, keys: &[String]) -> Result<(), String> {
        let mode = self.session.mode();
        let mut bytes = Vec::new();
        for key in keys {
            let keystroke = Keystroke::parse(key).map_err(|error| format!("invalid key {key:?}: {error}"))?;
            match key_action(&keystroke, mode) {
                KeyAction::Send(sequence) => bytes.extend(sequence),
                other => return Err(format!("key {key:?} means {other:?} in IVE and does not reach the shell")),
            }
        }
        self.session.type_bytes(bytes);
        Ok(())
    }

    /// Performs `gesture` as the mouse would; returns whether it went to the program (it tracks the mouse)
    /// rather than to IVE (selection, scrollback).
    pub fn send_mouse(&self, gesture: &Gesture, cx: &mut App) -> Result<bool, String> {
        let size = self.session.size();
        let ScreenCell { column, line } = gesture.cell;
        if column >= usize::from(size.columns) || line >= usize::from(size.lines) {
            return Err(format!("cell column {column}, line {line} is outside the {}×{} screen", size.columns, size.lines));
        }
        let at = At { cell: gesture.cell, side: Side::Left, modifiers: gesture.modifiers };
        let (session, pointer) = (&self.session, &self.pointer);
        Ok(match gesture.kind {
            GestureKind::Click { button, clicks } => {
                let reported = pointer.press(session, button, at, clicks, cx);
                pointer.release(session, button, at, cx);
                reported
            }
            GestureKind::Press(button) => pointer.press(session, button, at, 1, cx),
            GestureKind::Release(button) => pointer.release(session, button, at, cx),
            GestureKind::Move => pointer.moved(session, at, cx),
            GestureKind::Scroll(lines) => pointer.wheel(session, lines, at),
        })
    }

    pub fn screen_text(&self) -> ScreenText {
        self.session.with_term(|term| screen_text(term))
    }

    pub fn output_text(&self, last: Option<usize>) -> Vec<String> {
        self.session.with_term(|term| output_text(term, last))
    }

    /// Scrolls the terminal to what `at` names, its match selected; to the bottom when that is gone.
    pub fn reveal(&self, at: &FrameAt) {
        let target = self.session.with_term(|term| {
            let (window, _) = window_at(term, at, term.screen_lines());
            window.mark.map(|mark| (window.top, mark))
        });
        match target {
            Some((top, mark)) => self.session.reveal(top, mark),
            None => self.session.scroll_to_bottom(),
        }
    }

    /// `rows` lines of the terminal as `at` says, laid out without touching what its tab shows.
    pub fn snapshot(&self, at: &FrameAt, rows: usize, palette: &TerminalPalette) -> Snapshot {
        self.session.with_term(|term| {
            let (window, shown) = window_at(term, at, rows);
            Snapshot { frame: Rc::new(layout_window(term, &window, palette)), shown }
        })
    }

    /// Applies what the shell asked for: replies to its queries, clipboard, title, exit.
    pub fn handle_events(&mut self, events: Vec<Event>, cx: &mut App) -> TerminalState {
        for event in events {
            match event {
                Event::Title(title) => self.title = Some(title),
                Event::ResetTitle => self.title = None,
                Event::PtyWrite(text) => self.session.write(text.into_bytes()),
                Event::ClipboardStore(_, text) => cx.write_to_clipboard(ClipboardItem::new_string(text)),
                Event::ClipboardLoad(_, reply) => self.reply_with_clipboard(reply.as_ref(), cx),
                Event::ColorRequest(index, reply) => self.reply_with_color(index, reply.as_ref(), cx),
                Event::TextAreaSizeRequest(reply) => self.session.write(reply(self.session.size().window_size()).into_bytes()),
                Event::ChildExit(status) => self.exit_status = Some(status),
                Event::Exit => return self.exit_state(),
                Event::Wakeup | Event::Bell | Event::MouseCursorDirty | Event::CursorBlinkingChange => {}
            }
        }
        TerminalState::Running
    }

    fn reply_with_clipboard(&self, reply: &dyn Fn(&str) -> String, cx: &App) {
        if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
            self.session.write(reply(&text).into_bytes());
        }
    }

    fn reply_with_color(&self, index: usize, reply: &dyn Fn(alacritty_terminal::vte::ansi::Rgb) -> String, cx: &App) {
        let palette = cx.global::<Theme>().terminal;
        let color = self.session.with_term(|term| ColorResolver { palette: &palette, overrides: term.colors() }.indexed(index));
        self.session.write(reply(unpack(color)).into_bytes());
    }

    /// A shell that failed right after start (say, a broken `$SHELL`) leaves its status on screen; any other
    /// exit closes the tab, as in IntelliJ.
    fn exit_state(&self) -> TerminalState {
        let lost = self.session.lost().then(|| self.session.hosted_id()).flatten();
        if self.keeps_after_exit {
            return run_exit_state_of(self.exit_status, lost);
        }
        exit_state_of(self.exit_status, self.started.elapsed(), self.start, lost)
    }

    pub fn render(&self, theme: Theme, window: &mut Window, cx: &App) -> impl IntoElement {
        let metrics = CellMetrics::measure(window, cx);
        let focused = self.focus_handle.is_focused(window);
        let process = self.process;
        div()
            .id("terminal")
            .flex_1()
            .min_h_0()
            .overflow_hidden()
            .p(metrics.sizes.padding.px())
            .bg(rgb(theme.terminal.background))
            .cursor(CursorStyle::IBeam)
            .track_focus(&self.focus_handle)
            .on_key_down(on_key_down(self.session.clone(), self.pointer.origin.clone()))
            .on_any_mouse_down(on_mouse_down(self.session.clone(), self.pointer.clone(), self.focus_handle.clone()))
            .on_mouse_move(on_mouse_move(self.session.clone(), self.pointer.clone()))
            .on_mouse_up(MouseButton::Left, on_mouse_up(self.session.clone(), self.pointer.clone()))
            .on_mouse_up(MouseButton::Middle, on_mouse_up(self.session.clone(), self.pointer.clone()))
            .on_mouse_up(MouseButton::Right, on_mouse_up(self.session.clone(), self.pointer.clone()))
            // a drag that ends outside the terminal still releases the button
            .on_mouse_up_out(MouseButton::Left, on_mouse_up(self.session.clone(), self.pointer.clone()))
            .on_mouse_up_out(MouseButton::Middle, on_mouse_up(self.session.clone(), self.pointer.clone()))
            .on_mouse_up_out(MouseButton::Right, on_mouse_up(self.session.clone(), self.pointer.clone()))
            .on_scroll_wheel(on_scroll_wheel(self.session.clone(), self.pointer.clone()))
            .on_drop(on_drop_paths(self.session.clone(), self.focus_handle.clone(), |dropped: &ExternalPaths, _cx| pasted::shell_words(dropped.paths())))
            .on_drop(on_drop_paths(self.session.clone(), self.focus_handle.clone(), move |dragged: &DraggedFiles, cx| dragged.typed_into(process, cx)))
            .child(grid(self.session.clone(), self.pointer.clone(), metrics, theme.terminal, focused))
    }
}

/// Hands terminal events to the workspace; a burst of output arriving together repaints once.
fn forward_events(process: ProcessId, mut events: UnboundedReceiver<Event>, window: &mut Window, cx: &mut Context<Workspace>) -> Task<()> {
    cx.spawn_in(window, async move |workspace, cx| {
        while let Some(event) = events.next().await {
            let mut batch = vec![event];
            while let Ok(event) = events.try_recv() {
                batch.push(event);
            }
            let delivered = workspace.update_in(cx, |workspace, window, cx| workspace.handle_process_events(process, batch, window, cx));
            if delivered.is_err() {
                return;
            }
        }
    })
}

/// Mouse state shared between the grid, which records where it was painted, and the mouse handlers.
#[derive(Default)]
struct Pointer {
    /// top-left of the grid and the cell size, from the last paint
    geometry: Cell<Option<(Point<Pixels>, Size<Pixels>)>>,
    /// the button IVE took the press of, and its click count, until it is released
    held: Cell<Option<(Button, usize)>>,
    /// where a bound function's context comes from: the terminal's tab in the workspace
    origin: Option<(WeakEntity<Workspace>, ProcessId)>,
    /// the button whose press went to the program, so its release and drags do too
    reported: Cell<Option<Button>>,
    /// where the last report was; motion within one cell is not reported again
    reported_cell: Cell<Option<ScreenCell>>,
    /// wheel movement smaller than a line, kept for the next event
    scroll_remainder: Cell<f32>,
}

/// Where a mouse event happened: the screen cell, which half of it, and the modifiers held.
#[derive(Clone, Copy)]
struct At {
    cell: ScreenCell,
    side: Side,
    modifiers: Modifiers,
}

/// What a terminal mouse gesture does: a binding's action, a report to the program tracking the mouse, or nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Lookup {
    Run(keymap::Action),
    Report,
    Nothing,
}

/// The terminal's binding for `trigger`. While the program tracks the mouse (`tracking`) and shift is not held,
/// only `mouse_reporting` bindings apply and the rest goes to the program; with shift held IVE takes the gesture
/// and shift is not part of the match, as in WezTerm.
fn lookup_terminal(keymap: &Keymap, trigger: MouseTrigger, tracking: bool) -> Lookup {
    if tracking && !trigger.mods.shift {
        let reporting = MouseTrigger { mouse_reporting: true, ..trigger };
        return keymap.find_mouse(Scope::Terminal, &reporting).cloned().map_or(Lookup::Report, Lookup::Run);
    }
    let mods = if tracking { Modifiers { shift: false, ..trigger.mods } } else { trigger.mods };
    let local = MouseTrigger { mods, mouse_reporting: false, ..trigger };
    keymap.find_mouse(Scope::Terminal, &local).cloned().map_or(Lookup::Nothing, Lookup::Run)
}

/// Who took the press a release or drag belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Press {
    /// IVE: its own bindings decide, whatever the program tracks or shift says by now
    Ive,
    /// the program tracking the mouse: its reporting bindings decide, the rest is reported
    Program,
    /// neither: the press began outside the terminal
    Elsewhere,
}

/// The binding for a release or drag, following where its press went.
fn follow_press(keymap: &Keymap, trigger: MouseTrigger, press: Press) -> Lookup {
    match press {
        Press::Ive => lookup_terminal(keymap, trigger, false),
        Press::Program => lookup_terminal(keymap, trigger, true),
        Press::Elsewhere => Lookup::Nothing,
    }
}

fn bound_button(button: Button) -> Option<keymap::MouseButton> {
    match button {
        Button::Left => Some(keymap::MouseButton::Left),
        Button::Middle => Some(keymap::MouseButton::Middle),
        Button::Right => Some(keymap::MouseButton::Right),
        Button::WheelUp | Button::WheelDown => None,
    }
}

fn selection_type(mode: SelectionMode) -> SelectionType {
    match mode {
        SelectionMode::Cell => SelectionType::Simple,
        SelectionMode::Word => SelectionType::Semantic,
        SelectionMode::Line => SelectionType::Lines,
    }
}

/// What a mouse gesture does, for the real mouse and for MCP alike: what the terminal's mouse bindings say, or a
/// report to a program tracking the mouse.
impl Pointer {
    /// Where `position` falls on the grid, clamped to it; `None` before the first paint.
    fn at(&self, position: Point<Pixels>, modifiers: Modifiers, session: &Session) -> Option<At> {
        let (origin, cell) = self.geometry.get()?;
        let size = session.size();
        let x = (f32::from(position.x - origin.x) / f32::from(cell.width)).max(0.);
        let y = (f32::from(position.y - origin.y) / f32::from(cell.height)).max(0.);
        let column = (x as usize).min(usize::from(size.columns) - 1);
        let line = (y as usize).min(usize::from(size.lines) - 1);
        let side = if x.fract() < 0.5 { Side::Left } else { Side::Right };
        Some(At { cell: ScreenCell { column, line }, side, modifiers })
    }

    /// Returns whether the press went to the program.
    fn press(&self, session: &Session, button: Button, at: At, clicks: usize, cx: &mut App) -> bool {
        match self.lookup(MouseKind::Down, button, clicks, at, session, cx) {
            Lookup::Report => {
                if self.report(session, Action::Press, Some(button), at) {
                    self.reported.set(Some(button));
                }
                return true;
            }
            Lookup::Run(action) => self.run(action, at, session, cx),
            Lookup::Nothing => {}
        }
        self.held.set(Some((button, clicks)));
        false
    }

    /// Returns whether the motion went to the program.
    fn moved(&self, session: &Session, at: At, cx: &mut App) -> bool {
        if let Some((button, clicks)) = self.held.get() {
            if let Lookup::Run(action) = self.follow(MouseKind::Drag, button, clicks, at, Press::Ive, cx) {
                self.run(action, at, session, cx);
            }
            return false;
        }
        if let Some(button) = self.reported.get()
            && let Lookup::Run(action) = self.follow(MouseKind::Drag, button, 1, at, Press::Program, cx)
        {
            self.run(action, at, session, cx);
            return true;
        }
        if mouse::is_reported(session.mode(), &at.modifiers) || self.reported.get().is_some() {
            if self.reported_cell.get() != Some(at.cell) {
                self.report(session, Action::Motion, self.reported.get(), at);
            }
            return true;
        }
        false
    }

    /// Returns whether the release went to the program. It follows its press: a press IVE took ends with IVE's
    /// bindings, one the program got with its reporting bindings and a report, one from outside with nothing.
    fn release(&self, session: &Session, button: Button, at: At, cx: &mut App) -> bool {
        let held = self.held.take().filter(|(pressed, _)| *pressed == button);
        let reported = self.reported.get() == Some(button);
        let press = match (held, reported) {
            (Some(_), _) => Press::Ive,
            (None, true) => Press::Program,
            (None, false) => Press::Elsewhere,
        };
        let clicks = held.map_or(1, |(_, clicks)| clicks);
        if let Lookup::Run(action) = self.follow(MouseKind::Up, button, clicks, at, press, cx) {
            self.run(action, at, session, cx);
        }
        if reported {
            self.reported.set(None);
            // the program sees a release for every press it got, whatever a binding did
            return self.report(session, Action::Release, Some(button), at);
        }
        false
    }

    fn lookup(&self, kind: MouseKind, button: Button, clicks: usize, at: At, session: &Session, cx: &App) -> Lookup {
        let tracking = session.mode().intersects(TermMode::MOUSE_MODE);
        self.find(kind, button, clicks, at, cx, |keymap, trigger| lookup_terminal(keymap, trigger, tracking))
    }

    fn follow(&self, kind: MouseKind, button: Button, clicks: usize, at: At, press: Press, cx: &App) -> Lookup {
        self.find(kind, button, clicks, at, cx, |keymap, trigger| follow_press(keymap, trigger, press))
    }

    fn find(&self, kind: MouseKind, button: Button, clicks: usize, at: At, cx: &App, look: impl Fn(&Keymap, MouseTrigger) -> Lookup) -> Lookup {
        let Some(button) = bound_button(button) else {
            return Lookup::Nothing;
        };
        let trigger = MouseTrigger { kind, button, streak: clicks.clamp(1, 3) as u8, mods: at.modifiers, mouse_reporting: false };
        match keymap(cx) {
            Some(keymap) => look(keymap, trigger),
            None => look(&Keymap::empty(), trigger),
        }
    }

    fn run(&self, action: keymap::Action, at: At, session: &Session, cx: &mut App) {
        match action {
            keymap::Action::Builtin(Builtin::SelectTextAtMouseCursor(mode)) => {
                session.start_selection(selection_type(mode), grid_point(session, at.cell), at.side);
            }
            keymap::Action::Builtin(Builtin::ExtendSelectionToMouseCursor(_)) => session.extend_selection(grid_point(session, at.cell), at.side),
            keymap::Action::Builtin(Builtin::CompleteSelection(target) | Builtin::CopyTo(target)) => {
                if let Some(text) = session.selection_text().filter(|text| !text.is_empty()) {
                    clipboard::write(target, text, cx);
                }
            }
            keymap::Action::Builtin(Builtin::PasteFrom(source)) => paste(session, PasteKind::TextElseImage, source, cx),
            keymap::Action::Builtin(Builtin::Paste(kind, source)) => paste(session, kind, source, cx),
            keymap::Action::Send(text) => session.type_bytes(text.into_bytes()),
            keymap::Action::Invoke { generation, function } => self.invoke_at(generation, function, at.cell, cx),
            // the keymap allows no other action in the terminal scope
            _ => {}
        }
    }

    /// Runs a bound function with the terminal's tab and the cell as its context. Deferred: MCP's `send_mouse`
    /// comes while the workspace is being updated, and the context is read from it.
    fn invoke_at(&self, generation: u64, function: usize, cell: ScreenCell, cx: &mut App) {
        let origin = self.origin.clone();
        cx.defer(move |cx| {
            let context = origin.and_then(|(workspace, process)| Some(workspace.upgrade()?.read(cx).terminal_context(process, cell)));
            invoke(generation, function, context, cx);
        });
    }

    /// Scrolls `lines` (positive is up); returns whether the wheel went to the program. Full-screen programs
    /// such as `less` that do not track the mouse get arrow keys instead, as in xterm.
    fn wheel(&self, session: &Session, lines: i32, at: At) -> bool {
        let mode = session.mode();
        if mouse::is_reported(mode, &at.modifiers) {
            let button = if lines > 0 { Button::WheelUp } else { Button::WheelDown };
            for _ in 0..lines.unsigned_abs() {
                self.report(session, Action::Press, Some(button), at);
            }
            return true;
        }
        if mode.contains(TermMode::ALT_SCREEN | TermMode::ALTERNATE_SCROLL) && !at.modifiers.shift {
            let arrow: &[u8] = if lines > 0 { b"\x1bOA" } else { b"\x1bOB" };
            session.write(arrow.repeat(lines.unsigned_abs() as usize));
            return false;
        }
        session.scroll(lines);
        false
    }

    /// Sends `action` to the program; false when its tracking mode does not want it.
    fn report(&self, session: &Session, action: Action, button: Option<Button>, at: At) -> bool {
        let Some(bytes) = mouse::report(&MouseEvent { action, button, cell: at.cell, modifiers: at.modifiers }, session.mode()) else {
            return false;
        };
        self.reported_cell.set(Some(at.cell));
        session.write(bytes);
        true
    }
}

/// The grid cell of a screen cell, counting the scrolled-back history.
fn grid_point(session: &Session, cell: ScreenCell) -> GridPoint {
    let display_offset = session.with_term(|term| term.grid().display_offset()) as i32;
    GridPoint::new(Line(cell.line as i32 - display_offset), Column(cell.column))
}

fn button_of(button: MouseButton) -> Option<Button> {
    match button {
        MouseButton::Left => Some(Button::Left),
        MouseButton::Middle => Some(Button::Middle),
        MouseButton::Right => Some(Button::Right),
        MouseButton::Navigate(_) => None,
    }
}

/// `origin` is told of Enter, which submits input: the hub counts the process's worktree as used.
fn on_key_down(session: Rc<Session>, origin: Option<(WeakEntity<Workspace>, ProcessId)>) -> impl Fn(&KeyDownEvent, &mut Window, &mut App) {
    move |event, window, cx| {
        if let Some(action) = bound_action(event, Place::Terminal, cx) {
            // a window action (Settings, a new terminal) is the workspace's to run
            if run_bound_key(&session, action, cx) {
                cx.stop_propagation();
                window.refresh();
            }
            return;
        }
        match key_action(&event.keystroke, session.mode()) {
            KeyAction::Send(bytes) => {
                session.type_bytes(bytes);
                if is_submit(&event.keystroke)
                    && let Some((workspace, process)) = &origin
                {
                    workspace.update(cx, |workspace, cx| workspace.process_input(*process, cx)).ok();
                }
            }
            KeyAction::Unhandled => return,
        }
        cx.stop_propagation();
        window.refresh();
    }
}

fn is_submit(keystroke: &Keystroke) -> bool {
    keystroke.key == "enter" && keystroke.modifiers == Modifiers::default()
}

/// Runs a key's bound action in the terminal; false for an action only the workspace can run.
fn run_bound_key(session: &Session, action: keymap::Action, cx: &App) -> bool {
    match action {
        keymap::Action::Send(text) => session.type_bytes(text.into_bytes()),
        keymap::Action::Invoke { generation, function } => invoke(generation, function, None, cx),
        keymap::Action::Builtin(Builtin::CopyTo(target)) => {
            if let Some(text) = session.selection_text() {
                clipboard::write(target, text, cx);
            }
        }
        keymap::Action::Builtin(Builtin::PasteFrom(source)) => paste(session, PasteKind::TextElseImage, source, cx),
        keymap::Action::Builtin(Builtin::Paste(kind, source)) => paste(session, kind, source, cx),
        keymap::Action::Builtin(Builtin::Nop) => {}
        keymap::Action::Builtin(_) | keymap::Action::ShowTabMenu(_) | keymap::Action::ShowTreeMenu(_) | keymap::Action::Disable => return false,
    }
    true
}

/// Pastes what `kind` takes from `source`: text, and an image as the path of a file it is saved to, which Claude
/// Code attaches.
fn paste(session: &Session, kind: PasteKind, source: ClipboardTarget, cx: &App) {
    match pasted::paste_text(kind, clipboard::read_contents(source, cx), pasted::save_image) {
        Ok(Some(text)) => session.paste(&text),
        Ok(None) => {}
        Err(error) => eprintln!("ive: cannot save the pasted image: {error}"),
    }
}

/// Types the paths of what was dropped, from a file manager or the project tree, as `typed` writes them, and takes
/// the keyboard.
fn on_drop_paths<T: 'static>(session: Rc<Session>, focus: FocusHandle, typed: impl Fn(&T, &App) -> String + 'static) -> impl Fn(&T, &mut Window, &mut App) {
    move |dropped, window, cx| {
        session.paste(&typed(dropped, cx));
        window.focus(&focus, cx);
    }
}

fn on_mouse_down(session: Rc<Session>, pointer: Rc<Pointer>, focus: FocusHandle) -> impl Fn(&MouseDownEvent, &mut Window, &mut App) {
    move |event, window, cx| {
        window.focus(&focus, cx);
        let (Some(button), Some(at)) = (button_of(event.button), pointer.at(event.position, event.modifiers, &session)) else {
            return;
        };
        pointer.press(&session, button, at, event.click_count, cx);
        window.refresh();
    }
}

fn on_mouse_move(session: Rc<Session>, pointer: Rc<Pointer>) -> impl Fn(&MouseMoveEvent, &mut Window, &mut App) {
    move |event, window, cx| {
        // a drag follows only the button held since its press; moves without it (say, after it was released
        // outside) do not
        if pointer.held.get().is_some_and(|(button, _)| Some(button) != event.pressed_button.and_then(button_of)) {
            pointer.held.set(None);
            return;
        }
        if let Some(at) = pointer.at(event.position, event.modifiers, &session) {
            pointer.moved(&session, at, cx);
            window.refresh();
        }
    }
}

fn on_mouse_up(session: Rc<Session>, pointer: Rc<Pointer>) -> impl Fn(&MouseUpEvent, &mut Window, &mut App) {
    move |event, _window, cx| {
        if let (Some(button), Some(at)) = (button_of(event.button), pointer.at(event.position, event.modifiers, &session)) {
            pointer.release(&session, button, at, cx);
        }
    }
}

fn on_scroll_wheel(session: Rc<Session>, pointer: Rc<Pointer>) -> impl Fn(&ScrollWheelEvent, &mut Window, &mut App) {
    move |event, window, _cx| {
        let (Some((_, cell)), Some(at)) = (pointer.geometry.get(), pointer.at(event.position, event.modifiers, &session)) else {
            return;
        };
        let lines = pointer.scroll_remainder.get() + f32::from(event.delta.pixel_delta(cell.height).y) / f32::from(cell.height);
        pointer.scroll_remainder.set(lines.fract());
        let whole = lines.trunc() as i32;
        if whole != 0 && !pointer.wheel(&session, whole, at) {
            window.refresh();
        }
    }
}

/// Fits the terminal to its bounds before painting, then paints the screen.
fn grid(session: Rc<Session>, pointer: Rc<Pointer>, metrics: CellMetrics, palette: TerminalPalette, focused: bool) -> impl IntoElement {
    let painted = session.clone();
    canvas(
        move |bounds, _window, _cx| {
            pointer.geometry.set(Some((bounds.origin, size(metrics.cell_width, metrics.line_height))));
            session.resize(metrics.grid_size(bounds.size));
        },
        move |bounds, (), window, cx| {
            // laid out under the lock, painted after it, so the PTY thread is not held up by drawing
            let frame = painted.with_term(|term| layout(term.renderable_content(), &palette, focused));
            paint_frame(&frame, bounds.origin, metrics, palette.cursor, window, cx);
        },
    )
    .size_full()
}

/// Where a snapshot of a terminal looks: its last lines, or around line `index` of its output, with `bytes` of
/// that line marked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrameAt {
    Bottom,
    Line { index: usize, bytes: Range<usize> },
}

/// Lines of a terminal laid out, and what of them to keep in view.
#[derive(Debug, Clone)]
pub struct Snapshot {
    pub frame: Rc<Frame>,
    pub shown: Shown,
}

/// What a snapshot keeps in view when it is painted in less room than it has lines and columns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Shown {
    /// its last line, at the bottom
    Bottom,
    /// the match on frame line `line`, a third of the way down, and its `columns`
    Match { line: usize, columns: Range<usize> },
}

/// The grid lines `at` shows, and what of them to keep in view; a line no longer in the output shows the bottom.
fn window_at<T: EventListener>(term: &Term<T>, at: &FrameAt, rows: usize) -> (GridWindow, Shown) {
    let (history, screen) = (term.grid().history_size(), term.screen_lines());
    let output = output_rows(term);
    let found = match at {
        FrameAt::Line { index, bytes } => output.get(*index).map(|row| (row, bytes)),
        FrameAt::Bottom => None,
    };
    let Some((row, bytes)) = found else {
        let last = output.last().map_or(0, |row| row.first);
        let top = (last + 1 - rows as i32).min(screen as i32 - rows as i32).max(-(history as i32));
        return (GridWindow { top, rows, mark: None }, Shown::Bottom);
    };
    let start = output_point(term, row, bytes.start);
    let last_char = row.text.get(..bytes.end).and_then(|text| text.char_indices().last()).map_or(bytes.start, |(at, _)| at.max(bytes.start));
    let end = output_point(term, row, last_char);
    let top = top_showing(start.line.0, rows, history, screen);
    let shown = Shown::Match { line: (start.line.0 - top) as usize, columns: start.column.0..end.column.0.max(start.column.0) + 1 };
    (GridWindow { top, rows, mark: Some((start, end)) }, shown)
}

/// Paints `snapshot` as the terminal draws, taking no input, scrolled so what it keeps in view is in view.
pub fn snapshot_element(snapshot: Snapshot, palette: TerminalPalette) -> impl IntoElement {
    let Snapshot { frame, shown } = snapshot;
    let painted = canvas(
        |_, _, _| {},
        move |bounds, (), window, cx| {
            let metrics = CellMetrics::measure(window, cx);
            let columns = (f32::from(bounds.size.width) / f32::from(metrics.cell_width)).floor() as usize;
            let rows = (f32::from(bounds.size.height) / f32::from(metrics.line_height)).floor() as usize;
            let (right, down) = match &shown {
                Shown::Bottom => (0, frame.lines.len().saturating_sub(rows)),
                Shown::Match { line, columns: matched } => (columns_shift(matched, columns), line.saturating_sub(rows / 3)),
            };
            let origin = point(bounds.origin.x - metrics.cell_width * right as f32, bounds.origin.y - metrics.line_height * down as f32);
            window.with_content_mask(Some(ContentMask { bounds }), |window| paint_frame(&frame, origin, metrics, palette.cursor, window, cx));
        },
    )
    .size_full();
    div().flex_1().min_h_0().overflow_hidden().bg(rgb(palette.background)).child(painted)
}

/// Cell size of the monospace font at the terminal's text size.
#[derive(Clone, Copy)]
struct CellMetrics {
    font_size: Pixels,
    cell_width: Pixels,
    line_height: Pixels,
    sizes: TerminalSizes,
}

impl CellMetrics {
    fn measure(window: &Window, cx: &App) -> Self {
        let text_system = window.text_system();
        let font_id = text_system.resolve_font(&gpui::font(theme::mono_font(cx)));
        let sizes = cx.global::<Theme>().sizes.terminal;
        let font_size = sizes.text_size.px();
        let cell_width = text_system.advance(font_id, font_size, 'm').map_or(font_size * 0.6, |advance| advance.width);
        Self { font_size, cell_width, line_height: (font_size * sizes.line_height_factor.0).round(), sizes }
    }

    fn grid_size(&self, bounds: Size<Pixels>) -> GridSize {
        let cells = |length: Pixels, cell: Pixels| (f32::from(length) / f32::from(cell)).floor().clamp(1., f32::from(u16::MAX)) as u16;
        GridSize {
            columns: cells(bounds.width, self.cell_width),
            lines: cells(bounds.height, self.line_height),
            cell_width: f32::from(self.cell_width).round() as u16,
            cell_height: f32::from(self.line_height) as u16,
        }
    }

    fn cell_bounds(&self, origin: Point<Pixels>, line: usize, column: usize, cells: usize) -> Bounds<Pixels> {
        let top_left = point(origin.x + self.cell_width * column as f32, origin.y + self.line_height * line as f32);
        Bounds::new(top_left, size(self.cell_width * cells as f32, self.line_height))
    }
}

fn paint_frame(frame: &Frame, origin: Point<Pixels>, metrics: CellMetrics, cursor_color: u32, window: &mut Window, cx: &mut App) {
    for (line_index, line) in frame.lines.iter().enumerate() {
        for background in &line.fills {
            window.paint_quad(fill(metrics.cell_bounds(origin, line_index, background.column, background.width), rgb(background.color)));
        }
        for segment in &line.segments {
            paint_segment(segment, metrics.cell_bounds(origin, line_index, segment.column, segment.cells).origin, metrics, window, cx);
        }
    }
    if let Some(cursor) = frame.cursor {
        let cell = metrics.cell_bounds(origin, cursor.line, cursor.column, cursor.cells);
        paint_cursor(cursor, cell, cursor_color, metrics.sizes.cursor_thickness.px(), window);
    }
}

/// Glyphs are pinned to the cell grid, so fallback fonts with other advances cannot shift later columns.
fn paint_segment(segment: &Segment, origin: Point<Pixels>, metrics: CellMetrics, window: &mut Window, cx: &mut App) {
    let font = gpui::font(theme::mono_font(cx));
    let runs: Vec<TextRun> = segment.runs.iter().map(|run| text_run(run, &font, metrics.sizes)).collect();
    let force_width = (!segment.wide).then_some(metrics.cell_width);
    let shaped = window.text_system().shape_line(segment.text.clone().into(), metrics.font_size, &runs, force_width);
    // fails only when a glyph cannot be rasterized; the rest of the screen is still worth drawing
    let _ = shaped.paint(origin, metrics.line_height, gpui::TextAlign::Left, None, window, cx);
}

fn text_run(run: &Run, font: &Font, sizes: TerminalSizes) -> TextRun {
    let style = run.style;
    let color: Hsla = rgb(style.color).into();
    let thickness = sizes.line_thickness.px();
    let weight = if style.bold { sizes.bold_weight } else { sizes.normal_weight };
    let font = Font {
        weight: weight.font_weight(),
        style: if style.italic { FontStyle::Italic } else { FontStyle::Normal },
        ..font.clone()
    };
    let underline = match style.underline {
        Underline::None => None,
        Underline::Straight => Some(UnderlineStyle { thickness, color: Some(color), wavy: false }),
        Underline::Curly => Some(UnderlineStyle { thickness, color: Some(color), wavy: true }),
    };
    let strikethrough = style.strikeout.then_some(StrikethroughStyle { thickness, color: Some(color) });
    TextRun { len: run.len, font, color, background_color: None, underline, strikethrough }
}

fn paint_cursor(cursor: CursorMark, cell: Bounds<Pixels>, color: u32, thickness: Pixels, window: &mut Window) {
    let quad = match cursor.shape {
        CursorShape::Beam => fill(Bounds::new(cell.origin, size(thickness, cell.size.height)), rgb(color)),
        CursorShape::Underline => {
            fill(Bounds::new(point(cell.origin.x, cell.bottom() - thickness), size(cell.size.width, thickness)), rgb(color))
        }
        _ => outline(cell, rgb(color), BorderStyle::Solid),
    };
    window.paint_quad(quad);
}

/// How a terminal ends: lost hosted sessions say where they still run; adopted ones never count as failed to start,
/// since IVE did not see them start.
/// A run that exited stays with its code, however quickly it failed; one whose host was lost may still run.
fn run_exit_state_of(status: Option<ExitStatus>, lost: Option<&str>) -> TerminalState {
    match lost {
        Some(id) => TerminalState::Failed(format!("Lost the run; it still runs: ive --pty-host attach {id}").into()),
        None => TerminalState::Exited(exit_code_of(status)),
    }
}

/// The exit code, or 128 + the signal that killed the program, as shells report it; -1 when unknown.
#[cfg(unix)]
fn exit_code_of(status: Option<ExitStatus>) -> i32 {
    use std::os::unix::process::ExitStatusExt;
    status.and_then(|status| status.code().or_else(|| status.signal().map(|signal| 128 + signal))).unwrap_or(-1)
}

#[cfg(not(unix))]
fn exit_code_of(status: Option<ExitStatus>) -> i32 {
    status.and_then(|status| status.code()).unwrap_or(-1)
}

fn exit_state_of(status: Option<ExitStatus>, elapsed: Duration, start: Start, lost: Option<&str>) -> TerminalState {
    if let Some(id) = lost {
        return TerminalState::Failed(format!("Lost the terminal; it still runs: ive --pty-host attach {id}").into());
    }
    match status {
        Some(status) if !status.success() && start == Start::New && elapsed < QUICK_EXIT => {
            TerminalState::Failed(format!("The shell exited right after start: {status}").into())
        }
        _ => TerminalState::Closed,
    }
}

#[cfg(test)]
mod tests {
    use std::os::unix::process::ExitStatusExt;

    use super::*;
    use crate::script::keymap::{Binding, ClipboardTarget, Keymap, MouseButton as BoundButton, MouseKind, MouseTrigger, Scope, SelectionMode, Trigger};

    fn gesture(kind: MouseKind, button: BoundButton, modifiers: Modifiers) -> MouseTrigger {
        MouseTrigger { kind, button, streak: 1, mods: modifiers, mouse_reporting: false }
    }

    fn terminal_binding(trigger: MouseTrigger, builtin: Builtin) -> Binding {
        Binding { trigger: Trigger::Mouse(trigger), scope: Scope::Terminal, action: keymap::Action::Builtin(builtin) }
    }

    fn defaults() -> Keymap {
        Keymap::new(vec![
            terminal_binding(gesture(MouseKind::Down, BoundButton::Left, Modifiers::default()), Builtin::SelectTextAtMouseCursor(SelectionMode::Cell)),
            terminal_binding(gesture(MouseKind::Up, BoundButton::Middle, Modifiers::default()), Builtin::PasteFrom(ClipboardTarget::Primary)),
        ])
        .unwrap()
    }

    fn shift() -> Modifiers {
        Modifiers { shift: true, ..Modifiers::default() }
    }

    fn complete_on_up() -> Keymap {
        let up = gesture(MouseKind::Up, BoundButton::Left, Modifiers::default());
        Keymap::new(vec![terminal_binding(up, Builtin::CompleteSelection(ClipboardTarget::Primary))]).unwrap()
    }

    #[test]
    fn should_complete_selection_after_shift_is_released_mid_drag() {
        // given
        let up = gesture(MouseKind::Up, BoundButton::Left, Modifiers::default());

        // when
        let found = follow_press(&complete_on_up(), up, Press::Ive);

        // then
        assert_eq!(found, Lookup::Run(keymap::Action::Builtin(Builtin::CompleteSelection(ClipboardTarget::Primary))));
    }

    #[test]
    fn should_ignore_release_of_press_begun_outside_the_terminal() {
        // when
        let found = follow_press(&defaults(), gesture(MouseKind::Up, BoundButton::Middle, Modifiers::default()), Press::Elsewhere);

        // then
        assert_eq!(found, Lookup::Nothing);
    }

    #[test]
    fn should_run_reporting_up_binding_after_reported_press() {
        // given
        let reporting = MouseTrigger { mouse_reporting: true, ..gesture(MouseKind::Up, BoundButton::Middle, Modifiers::default()) };
        let keymap = Keymap::new(vec![terminal_binding(reporting, Builtin::Nop)]).unwrap();

        // when
        let found = follow_press(&keymap, gesture(MouseKind::Up, BoundButton::Middle, Modifiers::default()), Press::Program);

        // then
        assert_eq!(found, Lookup::Run(keymap::Action::Builtin(Builtin::Nop)));
    }

    #[test]
    fn should_report_release_of_reported_press_without_binding() {
        // when
        let found = follow_press(&defaults(), gesture(MouseKind::Up, BoundButton::Middle, Modifiers::default()), Press::Program);

        // then
        assert_eq!(found, Lookup::Report);
    }

    #[test]
    fn should_report_to_program_without_reporting_binding() {
        // when
        let found = lookup_terminal(&defaults(), gesture(MouseKind::Down, BoundButton::Left, Modifiers::default()), true);

        // then
        assert_eq!(found, Lookup::Report);
    }

    #[test]
    fn should_select_with_shift_while_program_tracks() {
        // when
        let found = lookup_terminal(&defaults(), gesture(MouseKind::Down, BoundButton::Left, shift()), true);

        // then
        assert_eq!(found, Lookup::Run(keymap::Action::Builtin(Builtin::SelectTextAtMouseCursor(SelectionMode::Cell))));
    }

    #[test]
    fn should_run_reporting_binding_while_program_tracks() {
        // given
        let reporting = MouseTrigger { mouse_reporting: true, ..gesture(MouseKind::Up, BoundButton::Middle, Modifiers::default()) };
        let keymap = Keymap::merged(defaults(), Keymap::new(vec![terminal_binding(reporting, Builtin::Nop)]).unwrap());

        // when
        let found = lookup_terminal(&keymap, gesture(MouseKind::Up, BoundButton::Middle, Modifiers::default()), true);

        // then
        assert_eq!(found, Lookup::Run(keymap::Action::Builtin(Builtin::Nop)));
    }

    #[test]
    fn should_paste_primary_on_middle_up() {
        // when
        let found = lookup_terminal(&defaults(), gesture(MouseKind::Up, BoundButton::Middle, Modifiers::default()), false);

        // then
        assert_eq!(found, Lookup::Run(keymap::Action::Builtin(Builtin::PasteFrom(ClipboardTarget::Primary))));
    }

    #[test]
    fn should_do_nothing_for_disabled_default() {
        // given
        let disabled = Binding { action: keymap::Action::Disable, ..terminal_binding(gesture(MouseKind::Down, BoundButton::Left, Modifiers::default()), Builtin::Nop) };
        let keymap = Keymap::merged(defaults(), Keymap::new(vec![disabled]).unwrap());

        // when
        let found = lookup_terminal(&keymap, gesture(MouseKind::Down, BoundButton::Left, Modifiers::default()), false);

        // then
        assert_eq!(found, Lookup::Nothing);
    }

    #[test]
    fn should_keep_shift_as_a_modifier_when_nothing_tracks() {
        // given
        let shift_click = gesture(MouseKind::Down, BoundButton::Left, shift());
        let keymap = Keymap::merged(defaults(), Keymap::new(vec![terminal_binding(shift_click, Builtin::Nop)]).unwrap());

        // when
        let found = lookup_terminal(&keymap, shift_click, false);

        // then
        assert_eq!(found, Lookup::Run(keymap::Action::Builtin(Builtin::Nop)));
    }

    fn failed(status: Option<ExitStatus>, elapsed: Duration, start: Start, lost: Option<&str>) -> Option<String> {
        match exit_state_of(status, elapsed, start, lost) {
            TerminalState::Failed(message) => Some(message.to_string()),
            TerminalState::Closed | TerminalState::Running | TerminalState::Exited(_) => None,
        }
    }

    #[test]
    fn should_give_the_exit_code_or_128_plus_the_signal() {
        // given
        let statuses = [Some(ExitStatus::from_raw(1 << 8)), Some(ExitStatus::from_raw(libc::SIGTERM)), None];

        // when
        let codes = statuses.map(exit_code_of);

        // then
        assert_eq!(codes, [1, 143, -1]);
    }

    #[test]
    fn should_keep_a_run_that_fails_right_after_start() {
        // given
        let status = Some(ExitStatus::from_raw(127 << 8));

        // when
        let state = run_exit_state_of(status, None);

        // then
        assert!(matches!(state, TerminalState::Exited(127)));
    }

    #[test]
    fn should_report_shell_that_failed_right_after_start() {
        // given
        let status = Some(ExitStatus::from_raw(1 << 8));

        // when
        let message = failed(status, Duration::from_millis(100), Start::New, None);

        // then
        assert!(message.is_some_and(|message| message.contains("right after start")));
    }

    #[test]
    fn should_close_adopted_session_that_exits_quickly() {
        // given
        let status = Some(ExitStatus::from_raw(1 << 8));

        // when
        let message = failed(status, Duration::from_millis(100), Start::Adopted, None);

        // then
        assert_eq!(message, None);
    }

    #[test]
    fn should_tell_how_to_reach_a_lost_session() {
        // given
        let lost = Some("t1a2b3");

        // when
        let message = failed(None, Duration::from_secs(60), Start::Adopted, lost);

        // then
        assert_eq!(message.as_deref(), Some("Lost the terminal; it still runs: ive --pty-host attach t1a2b3"));
    }
}
