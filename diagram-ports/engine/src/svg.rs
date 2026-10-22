use crate::{RenderError, RenderLimits};
use merman::svg::{
    RenderError as BackendError, RenderResult, SvgPostprocessContext, SvgPostprocessor,
};
use std::borrow::Cow;

pub(crate) struct NormalizeOutput {
    control: crate::RenderControl,
}

impl NormalizeOutput {
    pub(crate) fn new(control: crate::RenderControl) -> Self {
        Self { control }
    }
}

impl SvgPostprocessor for NormalizeOutput {
    fn name(&self) -> &'static str {
        "normalize-native-svg"
    }

    fn process<'a>(
        &self,
        svg: Cow<'a, str>,
        context: &SvgPostprocessContext<'_>,
    ) -> RenderResult<Cow<'a, str>> {
        let normalized = normalize_styles(&svg, &self.control)?;
        if context.diagram_type() == Some("info") {
            // Merman's info model records a 400 × 100 canvas but intentionally omits its viewBox.
            return Ok(Cow::Owned(normalized.replacen(
                "<svg ",
                "<svg viewBox=\"0 0 400 100\" ",
                1,
            )));
        }
        Ok(Cow::Owned(normalized))
    }
}

fn normalize_styles(svg: &str, control: &crate::RenderControl) -> RenderResult<String> {
    let document = roxmltree::Document::parse(svg).map_err(|error| {
        BackendError::svg_postprocess("normalize-native-svg", error.to_string())
    })?;
    let mut output = String::with_capacity(svg.len());
    let mut cursor = 0;
    for node in document
        .descendants()
        .filter(|node| node.has_tag_name("style"))
    {
        control.checkpoint().map_err(BackendError::Cancelled)?;
        let range = node.range();
        let fragment = &svg[range.clone()];
        let Some(start) = fragment.find('>').map(|offset| range.start + offset + 1) else {
            continue;
        };
        let Some(end) = fragment.rfind("</").map(|offset| range.start + offset) else {
            continue;
        };
        let css: String = node
            .children()
            .filter(roxmltree::Node::is_text)
            .filter_map(|node| node.text())
            .collect();
        // XML character-data quotes must reach the CSS parser as quotes, not &quot; tokens.
        output.push_str(&svg[cursor..start]);
        output.push_str(&css.replace('&', "&amp;").replace('<', "&lt;"));
        cursor = end;
    }
    output.push_str(&svg[cursor..]);
    Ok(output)
}

pub(crate) fn validate(svg: &str, limits: &RenderLimits) -> Result<(), RenderError> {
    if svg.len() > limits.svg_bytes {
        return Err(RenderError::UnsafeSvg(format!(
            "SVG has {} bytes, exceeding {}",
            svg.len(),
            limits.svg_bytes
        )));
    }
    let document = roxmltree::Document::parse(svg)
        .map_err(|error| RenderError::UnsafeSvg(error.to_string()))?;
    if document.root_element().tag_name().name() != "svg" {
        return Err(RenderError::UnsafeSvg("root element must be svg".into()));
    }
    for node in document.descendants().filter(roxmltree::Node::is_element) {
        validate_element(node)?;
    }
    Ok(())
}

fn validate_element(node: roxmltree::Node<'_, '_>) -> Result<(), RenderError> {
    let name = node.tag_name().name();
    if matches!(
        name,
        "script" | "foreignObject" | "iframe" | "object" | "embed" | "image" | "feImage"
    ) {
        return Err(RenderError::UnsafeSvg(format!(
            "element {name:?} is forbidden"
        )));
    }
    for attribute in node.attributes() {
        let name = attribute.name().to_ascii_lowercase();
        if name.starts_with("on") || name == "src" {
            return Err(RenderError::UnsafeSvg(format!(
                "attribute {name:?} is forbidden"
            )));
        }
        if name == "href" {
            attribute
                .value()
                .strip_prefix('#')
                .ok_or_else(|| RenderError::ExternalResource(attribute.value().into()))?;
        }
    }
    Ok(())
}
