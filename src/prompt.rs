//! Prompts drawn by IVE where the OS has none (Linux), in place of gpui's fallback: in the theme's colours,
//! answered by Enter (the first button that is not Cancel) and Escape (Cancel), and with each button a click
//! target `prompt:<label>`, so agents can answer them over MCP.

use gpui::{
    App, Context, Div, EventEmitter, FocusHandle, Focusable, KeyDownEvent, PromptButton, PromptHandle, PromptLevel,
    PromptResponse, Render, RenderablePromptHandle, SharedString, Stateful, Window, div, prelude::*, rgb, rgba,
};

use crate::click_targets::target;
use crate::theme::Theme;


/// For `App::set_prompt_builder`.
pub fn build(
    _level: PromptLevel,
    message: &str,
    detail: Option<&str>,
    actions: &[PromptButton],
    handle: PromptHandle,
    window: &mut Window,
    cx: &mut App,
) -> RenderablePromptHandle {
    let prompt = cx.new(|cx| IvePrompt {
        message: message.to_owned(),
        detail: detail.map(str::to_owned),
        actions: actions.to_vec(),
        focus: cx.focus_handle(),
    });
    handle.with_view(prompt, window, cx)
}

struct IvePrompt {
    message: String,
    detail: Option<String>,
    actions: Vec<PromptButton>,
    focus: FocusHandle,
}

impl IvePrompt {
    fn answer_key(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) {
        if let Some(index) = answer_for_key(&event.keystroke.key, &self.actions) {
            cx.emit(PromptResponse(index));
        }
    }
}

/// Which button a key presses: Enter the first that is not Cancel, Escape the Cancel one.
fn answer_for_key(key: &str, actions: &[PromptButton]) -> Option<usize> {
    let is_cancel = |action: &PromptButton| matches!(action, PromptButton::Cancel(_));
    match key {
        "enter" => actions.iter().position(|action| !is_cancel(action)),
        "escape" => actions.iter().position(is_cancel),
        _ => None,
    }
}

impl Render for IvePrompt {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let buttons = self.actions.iter().enumerate().map(|(index, action)| {
            let answer = cx.listener(move |_, _, _, cx| cx.emit(PromptResponse(index)));
            button(index, action.label().clone(), theme).on_click(target(format!("prompt:{}", action.label()), answer, cx))
        });
        let panel = dialog(theme, self.message.clone(), self.detail.clone(), buttons.collect())
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|prompt, event: &KeyDownEvent, _window, cx| prompt.answer_key(event, cx)));
        // a prompt is drawn outside the workspace, so it does not inherit the workspace's font
        scrim(panel, theme).font_family(crate::theme::ui_font(cx))
    }
}

/// A prompt button showing `label`; the first (index 0) is the primary one. Callers make it click target
/// `prompt:<name>`, so agents can answer over MCP even while the label changes.
pub fn button(index: usize, label: SharedString, theme: Theme) -> Stateful<Div> {
    let sizes = &theme.sizes.controls;
    let button = div()
        .id(("prompt-button", index))
        .px(sizes.prompt_button_padding_x.px())
        .py(sizes.prompt_button_padding_y.px())
        .rounded(sizes.prompt_button_radius.px())
        .text_size(sizes.prompt_button_text_size.px())
        .cursor_pointer()
        .child(label);
    match index {
        0 => button.bg(rgb(theme.accent)).text_color(rgb(theme.on_accent)),
        _ => button.border(sizes.prompt_button_border.px()).border_color(rgb(theme.border)),
    }
}

/// The prompt's panel: message, optional detail, buttons in a row on the right.
pub fn dialog(theme: Theme, message: String, detail: Option<String>, buttons: Vec<Stateful<Div>>) -> Div {
    let sizes = &theme.sizes.controls;
    div()
        .flex()
        .flex_col()
        .gap(sizes.prompt_gap.px())
        .w(sizes.prompt_width.px())
        .p(sizes.prompt_padding.px())
        .rounded(sizes.prompt_radius.px())
        .border(sizes.prompt_border.px())
        .border_color(rgb(theme.border))
        .bg(rgb(theme.sidebar))
        .text_color(rgb(theme.text))
        .child(div().font_weight(sizes.prompt_title_weight.font_weight()).child(message))
        .children(detail.map(|detail| div().text_size(sizes.prompt_detail_text_size.px()).text_color(rgb(theme.muted)).child(detail)))
        .child(div().flex().justify_end().gap(sizes.prompt_buttons_gap.px()).mt(sizes.prompt_buttons_margin_top.px()).children(buttons))
}

/// Darkens the window around `panel` and takes the mouse, so the window behind cannot be clicked while it waits.
pub fn scrim(panel: impl IntoElement, theme: Theme) -> Stateful<Div> {
    div().id("prompt-scrim").size_full().flex().items_center().justify_center().bg(rgba(theme.scrim)).occlude().child(panel)
}

impl EventEmitter<PromptResponse> for IvePrompt {}

impl Focusable for IvePrompt {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_answer_with_the_first_button_on_enter_and_cancel_on_escape() {
        // given
        let actions: Vec<PromptButton> = ["Close project", "Cancel"].map(PromptButton::from).to_vec();

        // when
        let answers = ["enter", "escape", "a"].map(|key| answer_for_key(key, &actions));

        // then
        assert_eq!(answers, [Some(0), Some(1), None]);
    }
}
