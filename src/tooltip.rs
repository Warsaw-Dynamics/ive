use gpui::{AnyView, App, AppContext, Context, IntoElement, Render, SharedString, Window, div, prelude::*, rgb};

use crate::theme::Theme;

/// A plain-text tooltip, for `.tooltip(text_tooltip(…))` on a stateful element.
pub fn text_tooltip(text: impl Into<SharedString>) -> impl Fn(&mut Window, &mut App) -> AnyView + 'static {
    let text = text.into();
    move |_window, cx| cx.new(|_| TextTooltip { text: text.clone() }).into()
}

struct TextTooltip {
    text: SharedString,
}

impl Render for TextTooltip {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        div()
            .px(theme.sizes.controls.tooltip_padding_x.px())
            .py(theme.sizes.controls.tooltip_padding_y.px())
            .rounded(theme.sizes.controls.tooltip_radius.px())
            .border(theme.sizes.controls.tooltip_border.px())
            .border_color(rgb(theme.border))
            .bg(rgb(theme.sidebar))
            .text_size(theme.sizes.controls.tooltip_text_size.px())
            .text_color(rgb(theme.text))
            .child(self.text.clone())
    }
}
