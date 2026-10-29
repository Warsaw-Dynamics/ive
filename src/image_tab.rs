//! An image file's tab: the picture, scaled down to fit the tab, read again when the file changes on disk.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use gpui::{AnyElement, App, ImageSource, ObjectFit, Resource, StyledImage, div, img, prelude::*, rgb};

use crate::theme::Theme;

/// The extensions shown as images: what gpui decodes, less SVG, which opens as text to edit.
const IMAGE_EXTENSIONS: [&str; 19] = [
    "png", "jpg", "jpeg", "gif", "webp", "bmp", "ico", "tif", "tiff", "tga", "dds", "hdr", "exr", "qoi", "pbm", "pgm", "ppm", "pam", "ff",
];

/// Whether the file at `path` opens in an image tab, by its extension in any case.
pub fn is_image(path: &Path) -> bool {
    let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
        return false;
    };
    IMAGE_EXTENSIONS.iter().any(|image| image.eq_ignore_ascii_case(extension))
}

/// An image file's tab; the path stays as the tab opened.
pub struct ImageTab {
    path: PathBuf,
}

impl ImageTab {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn file_name(&self) -> String {
        self.path.file_name().map_or_else(|| self.path.display().to_string(), |name| name.to_string_lossy().into_owned())
    }

    /// The file changed or went: gpui keeps decoded images by path, so the old picture is dropped to read it again.
    pub fn reload(&self, cx: &mut App) {
        self.source().remove_asset(cx);
    }

    /// The picture centred, at its own size or scaled down to fit; why it cannot be shown in its place.
    pub fn render(&self, theme: Theme) -> AnyElement {
        let path = self.path.display().to_string();
        div().flex().flex_1().min_h_0().size_full().p_4().bg(rgb(theme.background)).child(picture(self.source(), path, theme)).into_any_element()
    }

    fn source(&self) -> ImageSource {
        ImageSource::Resource(Resource::Path(Arc::from(self.path.as_path())))
    }
}

/// `source` filling its parent, at its own size or scaled down to fit; why it cannot be shown in its place. `name`
/// identifies it across frames (an animated GIF or WebP keeps its frame) and names it in the failure.
pub fn picture(source: ImageSource, name: String, theme: Theme) -> impl IntoElement {
    let failure = format!("Cannot show {name}: it is not an image IVE can read");
    let error = theme.error;
    img(source)
        .object_fit(ObjectFit::ScaleDown)
        .with_fallback(move || div().text_color(rgb(error)).child(failure.clone()).into_any_element())
        .size_full()
        .id(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_show_raster_images_in_any_case() {
        // given
        let paths = ["logo.png", "photo.JPG", "anim.gif", "icon.ico", "scan.TiFf", "a/b/c.webp"];
        // when
        let shown: Vec<bool> = paths.iter().map(|path| is_image(Path::new(path))).collect();
        // then
        assert!(shown.iter().all(|shown| *shown), "{paths:?} -> {shown:?}");
    }

    #[test]
    fn should_leave_svg_text_and_extensionless_files_to_the_editor() {
        // given
        let paths = ["drawing.svg", "main.rs", "README", ".png", "png"];
        // when
        let shown: Vec<bool> = paths.iter().map(|path| is_image(Path::new(path))).collect();
        // then
        assert!(shown.iter().all(|shown| !shown), "{paths:?} -> {shown:?}");
    }
}
