use crate::{cli::Comparison, svg::DisplaySvg};
use image::{ImageFormat, RgbaImage};
use std::{path::Path, sync::atomic::AtomicBool};

#[derive(Clone, Copy)]
enum CanvasSize {
    Contain,
    GraphvizPoints,
}

struct ExportPolicy {
    comparison: Comparison,
    canvas: CanvasSize,
}

pub fn write(svg: &str, output: &Path, comparison: Comparison) -> Result<(), String> {
    write_with_policy(
        svg,
        output,
        ExportPolicy {
            comparison,
            canvas: CanvasSize::Contain,
        },
    )
}

pub fn write_dot(svg: &str, output: &Path, comparison: Comparison) -> Result<(), String> {
    write_with_policy(
        svg,
        output,
        ExportPolicy {
            comparison,
            canvas: CanvasSize::GraphvizPoints,
        },
    )
}

/// Writes an engine-validated native DOT PNG and applies the normal pixel comparison.
pub fn write_native_dot_png(
    bytes: &[u8],
    output: &Path,
    comparison: Comparison,
) -> Result<(), String> {
    check_comparison_path(output, &comparison)?;
    if bytes.len() > 16 * 1024 * 1024 {
        return Err(format!(
            "native DOT PNG has {} bytes, exceeding the 16 MiB output limit",
            bytes.len()
        ));
    }
    let image = image::load_from_memory_with_format(bytes, ImageFormat::Png)
        .map_err(|error| format!("cannot decode native DOT PNG: {error}"))?
        .to_rgba8();
    std::fs::write(output, bytes)
        .map_err(|error| format!("cannot write PNG {}: {error}", output.display()))?;
    compare(&image, comparison)
}

fn check_comparison_path(output: &Path, comparison: &Comparison) -> Result<(), String> {
    let Comparison::Png(reference) = comparison else {
        return Ok(());
    };
    let actual = output
        .canonicalize()
        .unwrap_or_else(|_| output.to_path_buf());
    let expected = reference
        .canonicalize()
        .unwrap_or_else(|_| reference.to_path_buf());
    if actual == expected {
        return Err(format!(
            "PNG output {} is also its comparison reference",
            output.display()
        ));
    }
    Ok(())
}

fn write_with_policy(svg: &str, output: &Path, policy: ExportPolicy) -> Result<(), String> {
    let ExportPolicy { comparison, canvas } = policy;
    let format = output.extension().and_then(|extension| extension.to_str());
    match format {
        Some(extension) if extension.eq_ignore_ascii_case("svg") => {
            if matches!(comparison, Comparison::Png(_)) {
                return Err(format!(
                    "SVG output {} cannot use PNG comparison",
                    output.display()
                ));
            }
            std::fs::write(output, svg)
                .map_err(|error| format!("cannot write SVG {}: {error}", output.display()))
        }
        Some(extension) if extension.eq_ignore_ascii_case("png") => {
            check_comparison_path(output, &comparison)?;
            let image = rasterize(svg, canvas)?;
            image
                .save_with_format(output, ImageFormat::Png)
                .map_err(|error| format!("cannot write PNG {}: {error}", output.display()))?;
            compare(&image, comparison)
        }
        _ => Err(format!(
            "render output {} must end in .svg or .png",
            output.display()
        )),
    }
}

fn rasterize(svg: &str, canvas: CanvasSize) -> Result<RgbaImage, String> {
    let cancelled = AtomicBool::new(false);
    let display = DisplaySvg::parse(svg, &cancelled)?;
    let dimensions = display.dimensions();
    let size = match canvas {
        CanvasSize::Contain => (dimensions.0.ceil() as u32, dimensions.1.ceil() as u32),
        CanvasSize::GraphvizPoints => (dimensions.0.round() as u32, dimensions.1.round() as u32),
    };
    display.raster_rgba(size, &cancelled)
}

fn compare(actual: &RgbaImage, comparison: Comparison) -> Result<(), String> {
    let Comparison::Png(reference) = comparison else {
        return Ok(());
    };
    let metadata = std::fs::metadata(&reference).map_err(|error| {
        format!(
            "cannot inspect reference PNG {}: {error}",
            reference.display()
        )
    })?;
    if metadata.len() > 16 * 1024 * 1024 {
        return Err(format!(
            "reference PNG {} exceeds 16 MiB",
            reference.display()
        ));
    }
    let bytes = std::fs::read(&reference)
        .map_err(|error| format!("cannot read reference PNG {}: {error}", reference.display()))?;
    if bytes.len() < 24 || &bytes[..8] != b"\x89PNG\r\n\x1a\n" || &bytes[12..16] != b"IHDR" {
        return Err(format!(
            "reference PNG {} has no valid header",
            reference.display()
        ));
    }
    let width = u32::from_be_bytes(bytes[16..20].try_into().unwrap());
    let height = u32::from_be_bytes(bytes[20..24].try_into().unwrap());
    if width == 0
        || height == 0
        || width > 4096
        || height > 4096
        || u64::from(width) * u64::from(height) > 4_194_304
    {
        return Err(format!(
            "reference PNG {} exceeds raster bounds: {width}×{height}",
            reference.display()
        ));
    }
    let expected = image::load_from_memory_with_format(&bytes, ImageFormat::Png)
        .map_err(|error| {
            format!(
                "cannot decode reference PNG {}: {error}",
                reference.display()
            )
        })?
        .to_rgba8();
    if actual.dimensions() != expected.dimensions() {
        return Err(format!(
            "PNG dimensions differ: actual {:?}, reference {:?} in {}",
            actual.dimensions(),
            expected.dimensions(),
            reference.display()
        ));
    }
    let differing = actual
        .pixels()
        .zip(expected.pixels())
        .filter(|(left, right)| left != right)
        .count();
    if differing > 0 {
        return Err(format!(
            "PNG pixels differ: {differing} of {} in {}",
            actual.width() as u64 * actual.height() as u64,
            reference.display()
        ));
    }
    Ok(())
}
