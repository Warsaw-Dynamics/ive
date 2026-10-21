//! The clipboard and the primary selection (Linux and FreeBSD; elsewhere there is none), as bindings name them.

use gpui::{App, ClipboardEntry, ClipboardItem, Image};

use crate::script::keymap::ClipboardTarget;

pub fn write(target: ClipboardTarget, text: String, cx: &App) {
    let item = ClipboardItem::new_string(text);
    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    if matches!(target, ClipboardTarget::Primary | ClipboardTarget::Both) {
        cx.write_to_primary(item.clone());
    }
    if matches!(target, ClipboardTarget::Clipboard | ClipboardTarget::Both) {
        cx.write_to_clipboard(item);
    }
}

/// The text in `source`; `Both` reads the clipboard.
pub fn read(source: ClipboardTarget, cx: &App) -> Option<String> {
    read_item(source, cx).and_then(|item| item.text())
}

/// What a clipboard item holds for a terminal paste: its text and its first image, either or both.
pub struct Contents {
    pub text: Option<String>,
    pub image: Option<Image>,
}

impl Contents {
    pub fn of(item: ClipboardItem) -> Self {
        let text = item.text();
        let image = item.into_entries().find_map(|entry| match entry {
            ClipboardEntry::Image(image) => Some(image),
            _ => None,
        });
        Self { text, image }
    }
}

/// The text and image in `source`; `Both` reads the clipboard.
pub fn read_contents(source: ClipboardTarget, cx: &App) -> Contents {
    read_item(source, cx).map_or(Contents { text: None, image: None }, Contents::of)
}

fn read_item(source: ClipboardTarget, cx: &App) -> Option<ClipboardItem> {
    match source {
        ClipboardTarget::Primary => read_primary(cx),
        ClipboardTarget::Clipboard | ClipboardTarget::Both => cx.read_from_clipboard(),
    }
}

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
fn read_primary(cx: &App) -> Option<ClipboardItem> {
    cx.read_from_primary()
}

/// gpui offers the primary selection only where X11 and Wayland have one.
#[cfg(not(any(target_os = "linux", target_os = "freebsd")))]
fn read_primary(_cx: &App) -> Option<ClipboardItem> {
    None
}

#[cfg(test)]
mod tests {
    use gpui::ImageFormat;

    use super::*;

    #[test]
    fn should_read_text_and_image_of_one_item() {
        // given
        let image = Image::from_bytes(ImageFormat::Png, vec![1, 2, 3]);
        let item = ClipboardItem { entries: vec![ClipboardEntry::from("cells".to_owned()), ClipboardEntry::Image(image.clone())] };

        // when
        let contents = Contents::of(item);

        // then
        assert_eq!((contents.text.as_deref(), contents.image.map(|image| image.bytes)), (Some("cells"), Some(vec![1, 2, 3])));
    }

    #[test]
    fn should_read_nothing_from_empty_item() {
        // when
        let contents = Contents::of(ClipboardItem { entries: Vec::new() });

        // then
        assert_eq!((contents.text, contents.image.is_none()), (None, true));
    }
}
