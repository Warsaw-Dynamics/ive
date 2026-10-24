use std::sync::Arc;

use gpui::{Context, Image, IntoElement, Pixels, Render, SharedString, Size, Window, div, img, prelude::*, rgb};

use crate::theme::{self, Theme};

/// What a dragged tab would become if released here: a small window with its title and a shrunken picture of its
/// content. Shown in a pop-up following the pointer, which the main window cannot draw outside itself.
pub struct TabPreview {
    title: SharedString,
    image: Arc<Image>,
    size: Size<Pixels>,
}

impl TabPreview {
    /// `size` is the picture's, as it is drawn.
    pub fn new(title: SharedString, image: Arc<Image>, size: Size<Pixels>) -> Self {
        Self { title, image, size }
    }
}

impl Render for TabPreview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        crate::theme::apply_to_window(&theme, window);
        let sizes = &theme.sizes.tabs;
        let header = div()
            .flex()
            .flex_none()
            .items_center()
            .h(sizes.preview_header_height.px())
            .px(sizes.preview_padding_x.px())
            .bg(rgb(theme.sidebar))
            .border_b(sizes.preview_border.px())
            .border_color(rgb(theme.border))
            .text_size(sizes.preview_text_size.px())
            .overflow_hidden()
            .text_ellipsis()
            .whitespace_nowrap()
            .child(self.title.clone());
        div()
            .flex()
            .flex_col()
            .size_full()
            .overflow_hidden()
            .rounded(sizes.preview_radius.px())
            .border(sizes.preview_border.px())
            .border_color(rgb(theme.tab_active_border))
            .bg(rgb(theme.background))
            .opacity(sizes.preview_opacity.0)
            .font_family(theme::ui_font(cx))
            .text_color(rgb(theme.text))
            .child(header)
            .child(img(self.image.clone()).w(self.size.width).h(self.size.height))
    }
}
