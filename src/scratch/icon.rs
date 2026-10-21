use gpui::{Div, div, img, prelude::*};

use crate::icons;
use crate::theme::Theme;

/// A scratch's file-type icon with the scratch badge; `scratch.svg` draws its clock in the top-right corner of
/// the 16 px box, so it is laid over the whole icon unmoved.
pub fn scratch_icon(theme: Theme, name: &str) -> Div {
    div()
        .relative()
        .flex_none()
        .size(theme.sizes.icon.px())
        .child(img(icons::embedded(theme.appearance, icons::file_icon(name))).size_full())
        .child(img(icons::embedded(theme.appearance, icons::SCRATCH_BADGE)).absolute().top_0().left_0().size_full())
}
