//! A project's or module's favicon: alone in a row's icon slot, or small on another icon's lower right.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use gpui::{Div, ImageSource, Img, IntoElement, ParentElement, Pixels, Resource, Styled, div, img, px, rgb};

use crate::theme::Theme;

/// The favicon file `path` drawn at `size`; a file that does not decode draws nothing.
pub fn favicon(path: &Path, size: Pixels) -> Img {
    img(ImageSource::Resource(Resource::Path(Arc::from(path)))).flex_none().size(size)
}

/// A favicon on an icon, and the colour of the ring that cuts it out: the background the icon is drawn on.
pub struct Badge {
    pub path: PathBuf,
    pub ring: u32,
}

/// `icon` in its 16 px slot, with `badge` on its lower right, overhanging it.
pub fn badged(icon: impl IntoElement, badge: Option<Badge>, theme: Theme) -> Div {
    let sizes = &theme.sizes.tabs;
    let slot = div().relative().flex_none().size(theme.sizes.icon.px()).child(icon);
    let Some(badge) = badge else {
        return slot;
    };
    let offset = -sizes.badge_offset.0;
    // the ring is padding in the background's colour around the image, so a favicon that fails to decode leaves
    // only a small patch of that background, never a coloured square
    let ring = div()
        .absolute()
        .right(px(offset))
        .bottom(px(offset))
        .p(sizes.badge_ring.px())
        .rounded(px(3.))
        .bg(rgb(badge.ring))
        .child(favicon(&badge.path, sizes.badge_size.px()).rounded(px(2.)));
    slot.child(ring)
}
