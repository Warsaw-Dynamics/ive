//! The mark of a process that called for the user: a dot over the top right of an icon, in the theme's attention
//! colour, ringed with the colour behind it so it stands off the icon.

use gpui::{Div, IntoElement, div, prelude::*, px, rgb};

use crate::theme::Theme;

/// The dot's diameter and its ring's width, in pixels.
const DOT: f32 = 6.0;
const RING: f32 = 1.5;

/// `icon` with the dot over its top right corner; `behind` is the colour around the icon.
pub fn with_dot(icon: impl IntoElement, theme: Theme, behind: u32) -> Div {
    let dot = div()
        .absolute()
        .top(px(-RING - 1.0))
        .right(px(-RING - 2.0))
        .size(px(DOT + 2.0 * RING))
        .rounded_full()
        .border(px(RING))
        .border_color(rgb(behind))
        .bg(rgb(theme.attention));
    div().relative().flex_none().flex().child(icon).child(dot)
}
