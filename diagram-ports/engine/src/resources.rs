use crate::{RenderControl, RenderError};
use cssparser::{ParseError, Parser, Token};
use merman::svg::{
    RenderError as BackendError, RenderResult, SvgPostprocessContext, SvgPostprocessor,
};
use serde_json::Value;
use std::borrow::Cow;

pub(crate) fn validate_model(model: &merman::ManagedSemanticJson) -> Result<(), RenderError> {
    for field in [
        "nodes",
        "edges",
        "subgraphs",
        "classes",
        "actors",
        "entities",
    ] {
        match model.as_value().get(field) {
            Some(Value::Array(items)) => {
                for item in items {
                    validate_model_item(item)?;
                }
            }
            Some(Value::Object(items)) => {
                for item in items.values() {
                    validate_model_item(item)?;
                }
            }
            _ => {}
        }
    }
    if let Some(classes) = model.as_value().get("classDefs").and_then(Value::as_object) {
        for styles in classes.values() {
            validate_model_styles(styles)?;
        }
    }
    Ok(())
}

fn validate_model_item(item: &Value) -> Result<(), RenderError> {
    if let Some(image) = item
        .get("img")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
    {
        return Err(RenderError::ExternalResource(format!("image {image:?}")));
    }
    if let Some(link) = item.get("link").and_then(Value::as_str) {
        validate_reference(link)?;
    }
    if let Some(links) = item.get("links").and_then(Value::as_object) {
        for link in links.values().filter_map(Value::as_str) {
            validate_reference(link)?;
        }
    }
    for field in ["styles", "style", "cssStyles", "textStyles"] {
        if let Some(styles) = item.get(field) {
            validate_model_styles(styles)?;
        }
    }
    Ok(())
}

fn validate_model_styles(value: &Value) -> Result<(), RenderError> {
    match value {
        Value::String(css) => validate_css(css),
        Value::Array(values) => {
            for value in values {
                validate_model_styles(value)?;
            }
            Ok(())
        }
        Value::Object(_) => validate_model_item(value),
        _ => Ok(()),
    }
}

pub(crate) struct ResourceAdmission {
    control: RenderControl,
}

impl ResourceAdmission {
    pub(crate) fn new(control: RenderControl) -> Self {
        Self { control }
    }
}

impl SvgPostprocessor for ResourceAdmission {
    fn name(&self) -> &'static str {
        "admit-native-resources"
    }

    fn process<'a>(
        &self,
        svg: Cow<'a, str>,
        _context: &SvgPostprocessContext<'_>,
    ) -> RenderResult<Cow<'a, str>> {
        let document = roxmltree::Document::parse(&svg)
            .map_err(|error| BackendError::svg_postprocess(self.name(), error.to_string()))?;
        for node in document.descendants().filter(roxmltree::Node::is_element) {
            self.control.checkpoint().map_err(BackendError::Cancelled)?;
            validate_resource_element(node)
                .map_err(|error| BackendError::svg_postprocess(self.name(), error.to_string()))?;
        }
        Ok(svg)
    }
}

fn validate_resource_element(node: roxmltree::Node<'_, '_>) -> Result<(), RenderError> {
    for attribute in node.attributes() {
        match attribute.name().to_ascii_lowercase().as_str() {
            "href" | "src" => validate_reference(attribute.value())?,
            "style" | "fill" | "stroke" | "filter" | "clip-path" | "mask" | "marker"
            | "marker-start" | "marker-mid" | "marker-end" | "cursor" => {
                validate_css(attribute.value())?
            }
            _ => {}
        }
    }
    if node.has_tag_name("style") {
        let css: String = node
            .children()
            .filter(roxmltree::Node::is_text)
            .filter_map(|node| node.text())
            .collect();
        validate_css(&css)?;
    }
    Ok(())
}

fn validate_reference(value: &str) -> Result<(), RenderError> {
    if value.is_empty() || value.starts_with('#') {
        return Ok(());
    }
    Err(RenderError::ExternalResource(value.into()))
}

fn validate_css(css: &str) -> Result<(), RenderError> {
    let mut parser = Parser::new(css);
    parser.set_nested_block_limit(64);
    validate_css_tokens(&mut parser)
        .map_err(|error| RenderError::ExternalResource(format!("CSS resource: {error:?}")))
}

fn validate_css_tokens(parser: &mut Parser<'_>) -> Result<(), ParseError<String>> {
    while let Ok(token) = parser.next_including_whitespace_and_comments() {
        match token.clone() {
            Token::UnquotedUrl(url) => {
                validate_reference(&url).map_err(|error| ParseError::custom(error.to_string()))?
            }
            Token::AtKeyword(name) if name.eq_ignore_ascii_case("import") => {
                return Err(ParseError::custom("CSS @import is forbidden"));
            }
            Token::Function(name) => {
                parser.parse_nested_block(|nested| validate_css_function(nested, &name))?;
            }
            Token::ParenthesisBlock | Token::SquareBracketBlock | Token::CurlyBracketBlock => {
                parser.parse_nested_block(validate_css_tokens)?;
            }
            _ => {}
        }
    }
    Ok(())
}

fn validate_css_function(parser: &mut Parser<'_>, name: &str) -> Result<(), ParseError<String>> {
    if name.eq_ignore_ascii_case("url") {
        let url = parser.expect_url_or_string()?;
        return validate_reference(&url).map_err(|error| ParseError::custom(error.to_string()));
    }
    if name.eq_ignore_ascii_case("image-set") || name.eq_ignore_ascii_case("-webkit-image-set") {
        return Err(ParseError::custom(format!(
            "CSS {name} resource function is forbidden"
        )));
    }
    validate_css_tokens(parser)
}
