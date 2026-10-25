//! A project's or module's favicon: alone in a row's icon slot, or small on another icon's lower right.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use gpui::{AnyElement, Div, ImageSource, Img, IntoElement, ParentElement, Pixels, Resource, Styled, StyledImage, div, img, px, rgb};

use crate::theme::Theme;

/// The favicon file `path` drawn at `size`; a file that does not decode draws nothing.
pub fn favicon(path: &Path, size: Pixels) -> Img {
    img(ImageSource::Resource(Resource::Path(Arc::from(path)))).flex_none().size(size)
}

/// The favicon file `path` at `size`, or what `fallback` draws when the file does not decode: what the favicon stands
/// in for (a project's initials, a folder's icon), so the slot is never empty.
pub fn favicon_or(path: &Path, size: Pixels, fallback: impl Fn() -> AnyElement + 'static) -> Img {
    favicon(path, size).with_fallback(fallback)
}

/// What a badge shows: a favicon file, or one of IVE's own icons (the main checkout's star).
pub enum BadgeImage {
    Favicon(PathBuf),
    Embedded(&'static str),
}

/// A badge on an icon, and the colour of the ring that cuts it out: the background the icon is drawn on.
pub struct Badge {
    pub image: BadgeImage,
    pub ring: u32,
}

impl Badge {
    pub fn favicon(path: PathBuf, ring: u32) -> Self {
        Self { image: BadgeImage::Favicon(path), ring }
    }
}

fn badge_image(image: &BadgeImage, theme: Theme) -> Img {
    match image {
        BadgeImage::Favicon(path) => favicon(path, theme.sizes.tabs.badge_size.px()),
        BadgeImage::Embedded(name) => img(crate::icons::embedded(theme.appearance, name)).flex_none(),
    }
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
        .child(badge_image(&badge.image, theme).size(sizes.badge_size.px()).rounded(px(2.)));
    slot.child(ring)
}
