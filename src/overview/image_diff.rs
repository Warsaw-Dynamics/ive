//! A diff tab of an image: the picture before and after, side by side, in place of rows of text.

use std::path::Path;
use std::sync::Arc;

use gpui::{AnyElement, App, Image, ImageFormat, ImageSource, Resource, div, prelude::*, rgb};

use super::diff_tab::{DiffRequest, DiffSource};
use super::git_ui::muted;
use crate::git::diff::{self, ImageSide};
use crate::image_tab::picture;
use crate::theme::Theme;

/// Both pictures of an image's diff, each with what it is; a picture is `None` where the file is missing (added,
/// deleted, untracked).
pub struct ImageSides {
    old: Side,
    new: Side,
}

struct Side {
    label: String,
    picture: Option<Picture>,
}

/// A picture as read on the background executor, which an `ImageSource` cannot leave.
enum Picture {
    File(Arc<Path>),
    Bytes(Arc<Image>),
}

impl Picture {
    fn source(&self) -> ImageSource {
        match self {
            Self::File(path) => ImageSource::Resource(Resource::Path(path.clone())),
            Self::Bytes(image) => ImageSource::Image(image.clone()),
        }
    }
}

impl ImageSides {
    /// Reads both sides of `request`'s file. Reads git and the disk: run it on the background executor.
    pub fn read(request: &DiffRequest) -> Result<Self, String> {
        let (old, new) = diff::image_sides(&request.dir, &request.origin().sides, &request.file).map_err(|error| error.to_string())?;
        let (old_label, new_label) = side_labels(&request.source);
        Ok(Self { old: Side { label: old_label, picture: old.map(picture_of) }, new: Side { label: new_label, picture: new.map(picture_of) } })
    }

    /// Drops the decoded pictures from gpui's cache: a file on disk is read again when next shown.
    pub fn forget(&self, cx: &mut App) {
        for picture in [&self.old.picture, &self.new.picture].into_iter().flatten() {
            picture.source().remove_asset(cx);
        }
    }

    /// The two pictures side by side, each under its label, scaled down to fit.
    pub fn render(&self, path: &str, theme: Theme) -> AnyElement {
        let columns = [(&self.old, "old"), (&self.new, "new")].map(|(side, which)| column(side, format!("{path} ({which})"), theme));
        div().flex().flex_1().min_h_0().gap_4().p_4().bg(rgb(theme.background)).children(columns).into_any_element()
    }
}

fn column(side: &Side, name: String, theme: Theme) -> impl IntoElement {
    let body = match &side.picture {
        Some(shown) => picture(shown.source(), name, theme).into_any_element(),
        None => muted("no file", theme).into_any_element(),
    };
    div().flex().flex_col().flex_1().min_w_0().gap_2().child(muted(side.label.clone(), theme)).child(div().flex().flex_1().min_h_0().child(body))
}

/// What the old and the new side are: `HEAD` and `work tree` for an uncommitted diff.
fn side_labels(source: &DiffSource) -> (String, String) {
    match source {
        DiffSource::Uncommitted => ("HEAD".to_owned(), "work tree".to_owned()),
        DiffSource::Since { base, uncommitted: diff::Uncommitted::Included, .. } => (base.name.clone(), "work tree".to_owned()),
        DiffSource::Since { base, uncommitted: diff::Uncommitted::Excluded, .. } => (base.name.clone(), "HEAD".to_owned()),
        DiffSource::Commit { sha, .. } => (format!("before {}", sha.short()), sha.short().to_owned()),
        DiffSource::Files { left, right } => (left.display().to_string(), right.display().to_string()),
    }
}

/// A file on disk by its path, which gpui reads in any format it knows; a blob by its bytes, whose format gpui must
/// be told (one it cannot decode from bytes shows the picture's failure).
fn picture_of(side: ImageSide) -> Picture {
    match side {
        ImageSide::File(path) => Picture::File(Arc::from(path.as_path())),
        ImageSide::Blob(bytes) => {
            let format = format_of(&bytes);
            Picture::Bytes(Arc::new(Image::from_bytes(format, bytes)))
        }
    }
}

/// The format gpui decodes `bytes` as, by their first bytes; PNG for one it cannot decode from bytes, which then fails.
fn format_of(bytes: &[u8]) -> ImageFormat {
    match image::guess_format(bytes) {
        Ok(image::ImageFormat::Jpeg) => ImageFormat::Jpeg,
        Ok(image::ImageFormat::WebP) => ImageFormat::Webp,
        Ok(image::ImageFormat::Gif) => ImageFormat::Gif,
        Ok(image::ImageFormat::Bmp) => ImageFormat::Bmp,
        Ok(image::ImageFormat::Tiff) => ImageFormat::Tiff,
        Ok(image::ImageFormat::Ico) => ImageFormat::Ico,
        Ok(image::ImageFormat::Pnm) => ImageFormat::Pnm,
        _ => ImageFormat::Png,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_tell_formats_by_their_first_bytes() {
        // given
        let starts: [&[u8]; 4] = [b"\x89PNG\r\n\x1a\n", b"\xff\xd8\xff\xe0", b"GIF89a", b"RIFF\0\0\0\0WEBPVP8 "];
        // when
        let formats: Vec<ImageFormat> = starts.iter().map(|bytes| format_of(bytes)).collect();
        // then
        assert_eq!(formats, [ImageFormat::Png, ImageFormat::Jpeg, ImageFormat::Gif, ImageFormat::Webp]);
    }
}
