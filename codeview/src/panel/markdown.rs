//! A Markdown text's two views: Text, the body as for any text, and Rendered, the document formatted, shown instead of
//! the body while it is picked. The body stays as it is underneath (its undo history, caret and unsaved edits); a switch
//! keeps the part of the document at the top. See the spec "Markdown: Text | Rendered".

use std::rc::Rc;
use std::sync::Arc;

use gpui::{AnyElement, App, AppContext, Context, Entity, FocusHandle, Focusable, Window};

use super::{Body, Content, Event, MarkdownView, Offer, Panel, Pointer};
use crate::find::FindText;
use crate::markdown::pill::{SwitchListener, pill};
use crate::markdown::{Doc, RenderedView, build};
use crate::position::ScrollPosition;
use crate::syntax::{Cancel, Language, MAX_HIGHLIGHT_BYTES};

/// Rendered's revisions for the find box, apart from any body's.
const RENDERED_REVISIONS: u64 = 1 << 63;

/// The view picked, and the Rendered document while it is shown.
pub(crate) struct Markdown {
    view: MarkdownView,
    offer: Offer,
    rendered: Option<Entity<RenderedView>>,
    /// counts Rendered documents, so the find box tells them from each other
    revision: u64,
    building: gpui::Task<()>,
    cancel: Cancel,
    /// the source line Rendered shows at its top once its first document lands
    reveal: Option<usize>,
    /// the focus a switch left, handed to the view shown at the next frame when it still has the keyboard
    refocus: Option<FocusHandle>,
    /// new content is being opened: a switch it brings (a restore) is built once the content is in place
    opening: bool,
    #[cfg(test)]
    builds: usize,
}

impl Markdown {
    pub(crate) fn new() -> Self {
        Self {
            view: MarkdownView::Text,
            offer: Offer::No,
            rendered: None,
            revision: 0,
            building: gpui::Task::ready(()),
            cancel: Cancel::default(),
            reveal: None,
            refocus: None,
            opening: false,
            #[cfg(test)]
            builds: 0,
        }
    }

    fn wants_rendered(&self) -> bool {
        self.view == MarkdownView::Rendered && self.offer == Offer::Yes
    }

    /// Drops Rendered and stops building it.
    fn drop_rendered(&mut self) -> Option<Entity<RenderedView>> {
        self.cancel.cancel();
        self.reveal = None;
        self.rendered.take()
    }
}

impl Panel {
    /// The view shown; none when Rendered is not offered at all. A text too long for Rendered shows Text.
    pub fn markdown_view(&self) -> Option<MarkdownView> {
        match self.markdown.offer {
            Offer::No => None,
            Offer::TooLarge => Some(MarkdownView::Text),
            Offer::Yes => Some(self.markdown.view),
        }
    }

    pub fn markdown_offered(&self) -> Offer {
        self.markdown.offer
    }

    /// Rendered, while it is shown instead of the body.
    pub(super) fn shown_rendered(&self) -> Option<&Entity<RenderedView>> {
        self.markdown.rendered.as_ref().filter(|_| self.markdown.wants_rendered())
    }

    #[cfg(test)]
    pub(crate) fn rendered_view(&self) -> Option<Entity<RenderedView>> {
        self.shown_rendered().cloned()
    }

    /// How many Rendered documents were asked for.
    #[cfg(test)]
    pub(crate) fn markdown_builds(&self) -> usize {
        self.markdown.builds
    }

    /// The find box's revision of Rendered's text.
    pub(super) fn rendered_revision(&self) -> u64 {
        RENDERED_REVISIONS | self.markdown.revision
    }

    /// New content: whether it offers Rendered. Lines in Markdown do, in a panel that takes the mouse, up to the size
    /// the editor colours.
    pub(super) fn offer_markdown(&mut self, content: &Content, cx: &App) {
        let offer = match content {
            Content::Lines { source, language, .. } if self.selects() && is_markdown(*language) => {
                match source.byte_len() > MAX_HIGHLIGHT_BYTES as u64 {
                    true => Offer::TooLarge,
                    false => Offer::Yes,
                }
            }
            _ => Offer::No,
        };
        self.markdown.offer = offer;
        self.markdown.opening = true;
        if offer == Offer::Yes {
            return;
        }
        // Text takes the keyboard back when it was Rendered's
        if let Some(dropped) = self.markdown.drop_rendered() {
            self.markdown.refocus = Some(dropped.focus_handle(cx));
        }
    }

    /// The content is in place: Rendered, when picked, is built of it.
    pub(super) fn markdown_opened(&mut self, cx: &mut Context<Self>) {
        self.markdown.opening = false;
        if !self.markdown.wants_rendered() {
            return;
        }
        if self.markdown.rendered.is_none() && self.markdown.reveal.is_none() {
            self.markdown.reveal = Some(self.body_top_line(cx));
        }
        self.build_rendered(cx);
    }

    /// Shows `view` (a restore, the host's default); before content, the view the content opens in.
    pub(super) fn set_markdown_view(&mut self, view: MarkdownView, cx: &mut Context<Self>) {
        let settled = self.markdown.view == view && (view == MarkdownView::Text || self.markdown.rendered.is_some());
        if settled {
            return;
        }
        self.switch_markdown(view, cx);
    }

    /// The key or the pill: the other view, and the host hears it.
    pub(super) fn toggle_markdown_view(&mut self, cx: &mut Context<Self>) {
        if self.markdown.offer != Offer::Yes {
            return;
        }
        let next = match self.markdown.view {
            MarkdownView::Text => MarkdownView::Rendered,
            MarkdownView::Rendered => MarkdownView::Text,
        };
        self.switch_markdown(next, cx);
        cx.emit(Event::MarkdownViewToggled(next));
    }

    fn switch_markdown(&mut self, view: MarkdownView, cx: &mut Context<Self>) {
        self.markdown.view = view;
        if self.markdown.offer != Offer::Yes {
            return;
        }
        match view {
            MarkdownView::Rendered => {
                self.markdown.reveal = Some(self.body_top_line(cx));
                if !self.markdown.opening {
                    self.build_rendered(cx);
                }
            }
            MarkdownView::Text => self.back_to_text(cx),
        }
    }

    /// Text again: the body scrolled to the line Rendered had at its top; the find box searches the body.
    fn back_to_text(&mut self, cx: &mut Context<Self>) {
        let Some(rendered) = self.markdown.drop_rendered() else {
            return;
        };
        let line = rendered.read(cx).top_line();
        self.markdown.refocus = Some(rendered.focus_handle(cx));
        self.land_at(line, cx);
        // the current match's bytes are Rendered's: the body searches from its own place
        self.refind_from_anchor(cx);
        cx.notify();
    }

    /// Scrolls the body to source line `line` (from 0); the editor's caret follows when it is out of sight.
    fn land_at(&self, line: usize, cx: &mut Context<Self>) {
        match &self.body {
            Body::Edit(editor) => editor.update(cx, |editor, cx| editor.land_at(line, cx)),
            Body::View(view) => view.update(cx, |view, cx| {
                let left = view.scroll_position().left;
                view.restore_scroll(ScrollPosition { top_line: line as f64, left }, cx);
            }),
            Body::Diff(_) | Body::Empty => {}
        }
    }

    /// The body's text changed while Rendered is picked but not shown yet: the document being built is of the old
    /// text, so it is built again.
    pub(super) fn markdown_edited(&mut self, cx: &mut Context<Self>) {
        if self.markdown.wants_rendered() && self.markdown.rendered.is_none() {
            self.build_rendered(cx);
        }
    }

    /// Builds Rendered's document of the body's text, unsaved edits included, on the background executor.
    fn build_rendered(&mut self, cx: &mut Context<Self>) {
        self.markdown.cancel.cancel();
        let cancel = Cancel::default();
        self.markdown.cancel = cancel.clone();
        let text = whole_text(self.text_find_text(cx).1);
        #[cfg(test)]
        {
            self.markdown.builds += 1;
        }
        self.markdown.building = cx.spawn(async move |panel, cx| {
            let doc = cx.background_executor().spawn(async move { build(&text, &cancel) }).await;
            panel.update(cx, |panel, cx| panel.rendered_built(doc, cx)).ok();
        });
    }

    /// A document landed: Rendered shows it (the first one at the line the body had at its top).
    fn rendered_built(&mut self, doc: Doc, cx: &mut Context<Self>) {
        if !self.markdown.wants_rendered() {
            return;
        }
        let doc = Arc::new(doc);
        let switched = self.markdown.rendered.is_none();
        let rendered = match self.markdown.rendered.clone() {
            Some(rendered) => {
                rendered.update(cx, |view, cx| view.set_doc(doc, cx));
                rendered
            }
            None => self.show_rendered(doc, cx),
        };
        if let Some(line) = self.markdown.reveal.take() {
            rendered.update(cx, |view, cx| view.reveal_source_line(line, cx));
        }
        self.markdown.revision += 1;
        // just switched, the current match's bytes are the body's: Rendered searches from its top block
        if switched {
            self.refind_from_anchor(cx);
        } else {
            self.refind(cx);
        }
        self.notice_state(cx);
        cx.notify();
    }

    fn show_rendered(&mut self, doc: Arc<Doc>, cx: &mut Context<Self>) -> Entity<RenderedView> {
        let rendered = cx.new(|cx| RenderedView::new(doc, cx));
        self.markdown.refocus = Some(self.body_focus_handle(cx));
        self.markdown.rendered = Some(rendered.clone());
        rendered
    }

    /// The Text | Rendered pill, on a Markdown text while the find box is closed.
    pub(super) fn markdown_pill(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let shown = self.markdown_view().filter(|_| !self.find_is_open())?;
        let panel = cx.weak_entity();
        let switch: SwitchListener = Rc::new(move |view, _window, cx| drop(panel.update(cx, |panel, cx| panel.pick_markdown_view(view, cx))));
        Some(pill(shown, self.markdown.offer, switch, cx))
    }

    /// A click on the pill: the view clicked, as the key would switch to it.
    fn pick_markdown_view(&mut self, view: MarkdownView, cx: &mut Context<Self>) {
        if self.markdown_view() == Some(view) {
            return;
        }
        self.toggle_markdown_view(cx);
        self.notice_state(cx);
        cx.notify();
    }

    /// After a switch, the view shown takes the keyboard when the one it replaced had it.
    pub(super) fn settle_markdown_focus(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(left) = self.markdown.refocus.take() else {
            return;
        };
        if left.contains_focused(window, cx) {
            window.focus(&self.focus_handle(cx), cx);
        }
    }

    /// The first source line (from 0) the body shows.
    fn body_top_line(&self, cx: &App) -> usize {
        let top = match &self.body {
            Body::Edit(editor) => editor.read(cx).scroll_position().top_line,
            Body::View(view) => view.read(cx).scroll_position().top_line,
            Body::Diff(_) | Body::Empty => 0.,
        };
        top.max(0.).floor() as usize
    }

    fn selects(&self) -> bool {
        matches!(self.pointer, Pointer::Selects)
    }
}

fn is_markdown(language: Option<Language>) -> bool {
    language.is_some() && language == Language::for_alias("markdown")
}

/// All of a body's text, to parse.
fn whole_text(text: FindText) -> Vec<u8> {
    match text {
        FindText::Shared(source) => source.bytes(0..source.byte_len()).into_owned(),
        FindText::Owned(bytes) | FindText::Parts { bytes, .. } => bytes.as_ref().clone(),
    }
}
