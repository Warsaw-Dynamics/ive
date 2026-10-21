//! Pictures of IVE's own window, drawn by gpui rather than read from the screen.

use std::io::Cursor;

use gpui::{Bounds, Window};
use image::imageops::{self, FilterType};
use image::{ImageFormat, RgbaImage};

/// `window` as gpui last drew it; works while it is covered or minimized.
pub fn capture(window: &Window) -> Result<RgbaImage, String> {
    window.render_to_image().map_err(|error| format!("cannot draw the IVE window to an image: {error}"))
}

pub fn encode_png(image: &RgbaImage) -> Result<Vec<u8>, String> {
    let mut png = Vec::new();
    image
        .write_to(&mut Cursor::new(&mut png), ImageFormat::Png)
        .map_err(|error| format!("cannot encode the image as PNG: {error}"))?;
    Ok(png)
}

/// Part `area` of `image` (in its pixels, cut to fit), shrunk to fit within `max` keeping its proportions; never
/// enlarged.
pub fn thumbnail(image: &RgbaImage, area: Bounds<u32>, max: (u32, u32)) -> RgbaImage {
    let x = area.origin.x.min(image.width());
    let y = area.origin.y.min(image.height());
    let width = area.size.width.min(image.width() - x).max(1);
    let height = area.size.height.min(image.height() - y).max(1);
    let part = imageops::crop_imm(image, x, y, width, height).to_image();
    let scale = (max.0 as f32 / width as f32).min(max.1 as f32 / height as f32).min(1.);
    let size = (((width as f32 * scale).round() as u32).max(1), ((height as f32 * scale).round() as u32).max(1));
    imageops::resize(&part, size.0, size.1, FilterType::Triangle)
}

#[cfg(test)]
mod tests {
    use gpui::{point, size};

    use super::*;

    #[test]
    fn should_crop_then_shrink_keeping_proportions() {
        // given
        let image = RgbaImage::new(1000, 800);
        let area = Bounds::new(point(200, 100), size(800, 400));

        // when
        let thumbnail = thumbnail(&image, area, (200, 200));

        // then
        assert_eq!(thumbnail.dimensions(), (200, 100));
    }

    #[test]
    fn should_cut_an_area_reaching_past_the_image() {
        // given
        let image = RgbaImage::new(300, 200);
        let area = Bounds::new(point(100, 50), size(1000, 1000));

        // when
        let thumbnail = thumbnail(&image, area, (1000, 1000));

        // then
        assert_eq!(thumbnail.dimensions(), (200, 150));
    }
}
