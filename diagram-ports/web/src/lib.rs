#[cfg(target_arch = "wasm32")]
use diagram_engine::{DotRunner, GraphLayout};
use diagram_engine::{Language, RenderOptions};
use std::alloc::{Layout, alloc_zeroed, dealloc};
use std::sync::{Arc, OnceLock};

const MAX_SOURCE_BYTES: usize = 256 * 1024;
const MAX_OUTPUT_BYTES: usize = 16 * 1024 * 1024;
const MAX_PIXELS: u64 = 4_194_304;

enum Format {
    Svg,
    Png,
}

struct Output {
    bytes: Vec<u8>,
    error: bool,
}

fn render_source(source: &str) -> Result<String, String> {
    let options = RenderOptions {
        language: Language::PlantUml,
        #[cfg(target_arch = "wasm32")]
        graph_layout: GraphLayout::Dot(Arc::new(BrowserDot)),
        ..RenderOptions::default()
    };
    diagram_engine::render(source, &options)
        .map(|diagram| diagram.svg)
        .map_err(|error| error.to_string())
}

fn render_png(source: &str) -> Result<Vec<u8>, String> {
    let svg = render_source(source)?;
    let tree = usvg::Tree::from_str(&svg, &svg_options())
        .map_err(|error| format!("cannot parse rendered SVG for PNG: {error}"))?;
    let width = tree.size().width().ceil() as u32;
    let height = tree.size().height().ceil() as u32;
    if width == 0
        || height == 0
        || width > 4096
        || height > 4096
        || u64::from(width) * u64::from(height) > MAX_PIXELS
    {
        return Err(format!(
            "diagram raster size {width}×{height} exceeds 4096-axis/4194304-pixel bounds"
        ));
    }
    let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height)
        .ok_or_else(|| format!("cannot allocate diagram raster {width}×{height}"))?;
    let transform = resvg::tiny_skia::Transform::from_scale(
        width as f32 / tree.size().width(),
        height as f32 / tree.size().height(),
    );
    resvg::render(&tree, transform, &mut pixmap.as_mut());
    encode_png(pixmap.take(), width, height)
}

fn svg_options() -> usvg::Options<'static> {
    static FONTS: OnceLock<Arc<usvg::fontdb::Database>> = OnceLock::new();
    let fontdb = FONTS.get_or_init(|| {
        let mut fonts = usvg::fontdb::Database::new();
        for data in [
            include_bytes!("../fonts/LiberationSans-Regular.ttf").as_slice(),
            include_bytes!("../fonts/LiberationSans-Bold.ttf").as_slice(),
            include_bytes!("../fonts/LiberationSans-Italic.ttf").as_slice(),
            include_bytes!("../fonts/LiberationSans-BoldItalic.ttf").as_slice(),
            include_bytes!("../fonts/LiberationSerif-Regular.ttf").as_slice(),
            include_bytes!("../fonts/LiberationSerif-Bold.ttf").as_slice(),
            include_bytes!("../fonts/LiberationSerif-Italic.ttf").as_slice(),
            include_bytes!("../fonts/LiberationSerif-BoldItalic.ttf").as_slice(),
            include_bytes!("../fonts/LiberationMono-Regular.ttf").as_slice(),
            include_bytes!("../fonts/LiberationMono-Bold.ttf").as_slice(),
            include_bytes!("../fonts/LiberationMono-Italic.ttf").as_slice(),
            include_bytes!("../fonts/LiberationMono-BoldItalic.ttf").as_slice(),
        ] {
            fonts.load_font_data(data.to_vec());
        }
        fonts.set_sans_serif_family("Liberation Sans");
        fonts.set_serif_family("Liberation Serif");
        fonts.set_monospace_family("Liberation Mono");
        Arc::new(fonts)
    });
    usvg::Options {
        fontdb: Arc::clone(fontdb),
        ..usvg::Options::default()
    }
}

fn encode_png(mut pixels: Vec<u8>, width: u32, height: u32) -> Result<Vec<u8>, String> {
    for pixel in pixels.as_chunks_mut::<4>().0 {
        let alpha = u16::from(pixel[3]);
        if alpha > 0 && alpha < 255 {
            for channel in &mut pixel[..3] {
                *channel = ((u16::from(*channel) * 255 + alpha / 2) / alpha).min(255) as u8;
            }
        }
    }
    let mut output = Vec::new();
    let mut encoder = png::Encoder::new(&mut output, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder
        .write_header()
        .map_err(|error| format!("cannot start PNG encoding: {error}"))?;
    writer
        .write_image_data(&pixels)
        .map_err(|error| format!("cannot encode PNG: {error}"))?;
    drop(writer);
    Ok(output)
}

#[cfg(target_arch = "wasm32")]
struct BrowserDot;

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "env")]
unsafe extern "C" {
    fn diagram_dot_layout(pointer: *const u8, len: usize) -> *mut u8;
}

#[cfg(target_arch = "wasm32")]
impl DotRunner for BrowserDot {
    fn run(&self, source: &str) -> Result<String, String> {
        // SAFETY: the synchronous JS import reads source only during this call.
        let handle = unsafe { diagram_dot_layout(source.as_ptr(), source.len()) };
        if handle.is_null() {
            return Err("browser DOT returned no result".into());
        }
        // SAFETY: the JS import returns a live Box<Output> from diagram_output_alloc.
        let output = unsafe { Box::from_raw(handle.cast::<Output>()) };
        let text = String::from_utf8(output.bytes)
            .map_err(|error| format!("browser DOT output is not UTF-8: {error}"))?;
        if output.error {
            return Err(text);
        }
        Ok(text)
    }
}

/// Allocates `len` bytes of WASM memory for a UTF-8 diagram source.
#[unsafe(no_mangle)]
pub extern "C" fn diagram_alloc(len: usize) -> *mut u8 {
    if len == 0 || len > MAX_SOURCE_BYTES {
        return std::ptr::null_mut();
    }
    let layout = Layout::array::<u8>(len).expect("bounded source layout");
    // SAFETY: the layout has nonzero size and fits the bounded source limit.
    unsafe { alloc_zeroed(layout) }
}

/// Releases a source buffer allocated by `diagram_alloc`.
///
/// # Safety
/// `pointer` and `len` must match a live `diagram_alloc` result and be freed once.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn diagram_free(pointer: *mut u8, len: usize) {
    if pointer.is_null() || len == 0 || len > MAX_SOURCE_BYTES {
        return;
    }
    let layout = Layout::array::<u8>(len).expect("bounded source layout");
    // SAFETY: the caller returns the pointer and original length from diagram_alloc.
    unsafe { dealloc(pointer, layout) };
}

/// Renders UTF-8 PlantUML source to SVG; returns an owned result handle.
///
/// # Safety
/// `pointer` must address at least `len` live bytes from `diagram_alloc`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn diagram_render_svg(pointer: *const u8, len: usize) -> *mut u8 {
    // SAFETY: the caller supplies a live diagram_alloc buffer of len bytes.
    unsafe { render_input(pointer, len, Format::Svg) }
}

/// Renders UTF-8 PlantUML source to PNG; returns an owned result handle.
///
/// # Safety
/// `pointer` must address at least `len` live bytes from `diagram_alloc`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn diagram_render_png(pointer: *const u8, len: usize) -> *mut u8 {
    // SAFETY: the caller supplies a live diagram_alloc buffer of len bytes.
    unsafe { render_input(pointer, len, Format::Png) }
}

unsafe fn render_input(pointer: *const u8, len: usize, format: Format) -> *mut u8 {
    let result = if pointer.is_null() || len == 0 || len > MAX_SOURCE_BYTES {
        Err(format!(
            "diagram source length {len} must be in 1..={MAX_SOURCE_BYTES}"
        ))
    } else {
        // SAFETY: the caller passes a live diagram_alloc buffer of at least len bytes.
        let bytes = unsafe { std::slice::from_raw_parts(pointer, len) };
        std::str::from_utf8(bytes)
            .map_err(|error| format!("diagram source is not UTF-8: {error}"))
            .and_then(|source| match format {
                Format::Svg => render_source(source).map(String::into_bytes),
                Format::Png => render_png(source),
            })
    };
    let output = match result {
        Ok(bytes) => Output {
            bytes,
            error: false,
        },
        Err(message) => Output {
            bytes: message.into_bytes(),
            error: true,
        },
    };
    Box::into_raw(Box::new(output)).cast()
}

/// Returns the bytes owned by a result handle.
///
/// # Safety
/// `handle` must be a live result; copy the bytes before freeing it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn diagram_output_data(handle: *const u8) -> *const u8 {
    // SAFETY: the caller passes a live result from diagram_render_svg.
    unsafe { handle.cast::<Output>().as_ref() }
        .map_or(std::ptr::null(), |output| output.bytes.as_ptr())
}

/// Returns the byte length owned by a result handle.
///
/// # Safety
/// `handle` must be a live result from a render or `diagram_output_alloc`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn diagram_output_len(handle: *const u8) -> usize {
    // SAFETY: the caller passes a live result from diagram_render_svg.
    unsafe { handle.cast::<Output>().as_ref() }.map_or(0, |output| output.bytes.len())
}

/// Reports whether the result contains an error message.
///
/// # Safety
/// `handle` must be a live result from a render or `diagram_output_alloc`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn diagram_output_is_error(handle: *const u8) -> bool {
    // SAFETY: the caller passes a live result from diagram_render_svg.
    unsafe { handle.cast::<Output>().as_ref() }.is_some_and(|output| output.error)
}

/// Releases a result handle after its bytes have been copied.
///
/// # Safety
/// `handle` must be a live result and may be freed only once.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn diagram_output_free(handle: *mut u8) {
    if handle.is_null() {
        return;
    }
    // SAFETY: the caller passes a live result once, from diagram_render_svg.
    unsafe { drop(Box::from_raw(handle.cast::<Output>())) };
}

/// Allocates a DOT result buffer for the synchronous browser Graphviz import.
#[unsafe(no_mangle)]
pub extern "C" fn diagram_output_alloc(len: usize, error: bool) -> *mut u8 {
    if len == 0 || len > MAX_OUTPUT_BYTES {
        return std::ptr::null_mut();
    }
    Box::into_raw(Box::new(Output {
        bytes: vec![0; len],
        error,
    }))
    .cast()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_render_plantuml_as_svg() {
        // given
        let source = "@startuml\nAlice -> Bob : hello\n@enduml";

        // when
        let svg = render_source(source).unwrap();

        // then
        assert!(svg.contains("<svg"));
        assert!(svg.contains("Alice"));
    }

    #[test]
    fn should_return_an_error_for_invalid_plantuml() {
        // given
        let source = "@startuml\nAlice ->\n@enduml";

        // when
        let result = render_source(source);

        // then
        assert!(result.is_err());
    }

    #[test]
    fn should_render_plantuml_as_png() {
        // given
        let source = "@startuml\nAlice -> Bob : hello\n@enduml";

        // when
        let bytes = render_png(source).unwrap();
        let decoded = png::Decoder::new(std::io::Cursor::new(&bytes))
            .read_info()
            .unwrap();

        // then
        assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
        assert!(decoded.info().width > 0);
        assert!(decoded.info().height > 0);
    }

    #[test]
    fn should_resolve_generic_sans_serif_for_png_text() {
        // given
        let options = svg_options();
        let query = usvg::fontdb::Query {
            families: &[usvg::fontdb::Family::SansSerif],
            ..Default::default()
        };

        // when
        let face = options.fontdb.query(&query);

        // then
        assert!(face.is_some());
    }
}
