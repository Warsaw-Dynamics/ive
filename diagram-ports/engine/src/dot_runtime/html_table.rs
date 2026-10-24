//! Bounded Graphviz HTML-like table labels: one row and one text cell.

use super::{Attributes, RenderError, font_metrics};

const MAX_LABEL_BYTES: usize = 4096;
const FONT_SIZE: f64 = 14.;

pub(super) struct HtmlTable {
    text: String,
    text_width: f64,
    table_width: f64,
    table_height: f64,
    cell_width: f64,
    cell_height: f64,
    table_border: f64,
    cell_border: f64,
}

impl HtmlTable {
    pub(super) fn parse(label: &str, attrs: &Attributes) -> Result<Self, RenderError> {
        if label.len() > MAX_LABEL_BYTES {
            return Err(unsupported("HTML table label exceeds 4096 bytes"));
        }
        if label.contains("<!") || label.contains("<?") {
            return Err(unsupported(
                "HTML table declarations and processing instructions",
            ));
        }
        if attrs.get("shape").map(String::as_str) != Some("plaintext") {
            return Err(unsupported("HTML table requires shape=plaintext"));
        }
        if attrs
            .get("fontname")
            .is_some_and(|font| font != "Times-Roman")
            || attrs.get("fontsize").is_some_and(|size| size != "14")
            || attrs.get("fontcolor").is_some_and(|color| color != "black")
        {
            return Err(unsupported(
                "HTML table supports default Times-Roman 14 black text",
            ));
        }
        if [
            "style",
            "color",
            "fillcolor",
            "margin",
            "image",
            "imagescale",
        ]
        .iter()
        .any(|name| attrs.contains_key(*name))
        {
            return Err(unsupported(
                "HTML table node paint, margin, or image attributes",
            ));
        }
        let document = roxmltree::Document::parse(label)
            .map_err(|error| unsupported(&format!("HTML table markup {error}")))?;
        let table = document.root_element();
        if !table.tag_name().name().eq_ignore_ascii_case("TABLE") {
            return Err(unsupported("HTML label requires a TABLE root"));
        }
        let border = numeric_attr(table, "BORDER", 1.)?;
        let spacing = numeric_attr(table, "CELLSPACING", 2.)?;
        let padding = numeric_attr(table, "CELLPADDING", 2.)?;
        table_attributes(table)?;
        let row = one_child(table, "TR")?;
        if row.attributes().len() > 0 {
            return Err(unsupported("HTML table TR attributes"));
        }
        let cell = one_child(row, "TD")?;
        let cell_border = numeric_attr(cell, "BORDER", border)?;
        let cell_padding = numeric_attr(cell, "CELLPADDING", padding)?;
        cell_attributes(cell)?;
        let children = cell.children().collect::<Vec<_>>();
        let text = if children.len() == 1 && children[0].is_text() {
            children[0].text().unwrap_or("")
        } else {
            ""
        };
        if text.is_empty() || !text.bytes().all(|byte| (32..=126).contains(&byte)) {
            return Err(unsupported(
                "HTML table TD requires one nonempty ASCII text span",
            ));
        }
        let text_width = font_metrics::ascii_advance(text, "Times-Roman", FONT_SIZE)
            .ok_or_else(|| unsupported("HTML table text width"))?;
        let cell_width = text_width + 2. * (cell_padding + cell_border);
        let cell_height = FONT_SIZE * 1.2 + 2. * (cell_padding + cell_border);
        Ok(Self {
            text: text.into(),
            text_width,
            table_width: cell_width + 2. * (spacing + border),
            table_height: cell_height + 2. * (spacing + border),
            cell_width,
            cell_height,
            table_border: border,
            cell_border,
        })
    }

    pub(super) fn node_dimensions(&self, minimum: (f64, f64)) -> (f64, f64) {
        (
            minimum.0.max(self.table_width + 16.),
            minimum.1.max(self.table_height + 8.),
        )
    }

    pub(super) fn svg(&self, center: (f64, f64)) -> String {
        let mut output = String::new();
        if self.table_border > 0. {
            output.push_str(&outline(center, (self.table_width, self.table_height)));
        }
        if self.cell_border > 0. {
            output.push_str(&outline(center, (self.cell_width, self.cell_height)));
        }
        let x = number(center.0 - self.text_width / 2.);
        let y = number(center.1 + FONT_SIZE * 0.4);
        output.push_str(&format!("<text xml:space=\"preserve\" text-anchor=\"start\" x=\"{x}\" y=\"{y}\" font-family=\"Times,serif\" font-size=\"14.00\">{}</text>\n", escape(&self.text)));
        output
    }
}

fn one_child<'a>(
    parent: roxmltree::Node<'a, 'a>,
    expected: &str,
) -> Result<roxmltree::Node<'a, 'a>, RenderError> {
    if parent.children().any(|child| {
        !child.is_element()
            && (!child.is_text() || child.text().is_some_and(|text| !text.trim().is_empty()))
    }) {
        return Err(unsupported(&format!(
            "HTML table {expected} contains extra content"
        )));
    }
    let mut children = parent.children().filter(|child| child.is_element());
    let child = children
        .next()
        .ok_or_else(|| unsupported(&format!("HTML table requires {expected}")))?;
    if children.next().is_some() || !child.tag_name().name().eq_ignore_ascii_case(expected) {
        return Err(unsupported(&format!("HTML table requires one {expected}")));
    }
    Ok(child)
}

fn numeric_attr(
    node: roxmltree::Node<'_, '_>,
    name: &str,
    default: f64,
) -> Result<f64, RenderError> {
    let Some(value) = node
        .attributes()
        .find(|attr| attr.name().eq_ignore_ascii_case(name))
    else {
        return Ok(default);
    };
    let number = value
        .value()
        .parse::<u8>()
        .map_err(|_| unsupported(&format!("HTML table {name}={:?}", value.value())))?;
    let maximum = if name == "BORDER" { 1 } else { 8 };
    if number > maximum {
        return Err(unsupported(&format!(
            "HTML table {name}={} exceeds {maximum}",
            number
        )));
    }
    Ok(f64::from(number))
}

fn table_attributes(table: roxmltree::Node<'_, '_>) -> Result<(), RenderError> {
    for attr in table.attributes() {
        if !["BORDER", "CELLSPACING", "CELLPADDING"]
            .iter()
            .any(|name| attr.name().eq_ignore_ascii_case(name))
        {
            return Err(unsupported(&format!(
                "HTML table attribute {:?}",
                attr.name()
            )));
        }
    }
    Ok(())
}

fn cell_attributes(cell: roxmltree::Node<'_, '_>) -> Result<(), RenderError> {
    for attr in cell.attributes() {
        if !["BORDER", "CELLPADDING"]
            .iter()
            .any(|name| attr.name().eq_ignore_ascii_case(name))
        {
            return Err(unsupported(&format!(
                "HTML table TD attribute {:?}",
                attr.name()
            )));
        }
    }
    Ok(())
}

fn outline(center: (f64, f64), size: (f64, f64)) -> String {
    let (left, right) = (
        number(center.0 - size.0 / 2.),
        number(center.0 + size.0 / 2.),
    );
    let (top, bottom) = (
        number(center.1 - size.1 / 2.),
        number(center.1 + size.1 / 2.),
    );
    format!(
        "<polygon fill=\"none\" stroke=\"black\" points=\"{left},{bottom} {left},{top} {right},{top} {right},{bottom} {left},{bottom}\"/>\n"
    )
}

fn number(value: f64) -> String {
    if value.abs() < 0.005 {
        return "0".into();
    }
    let number = format!("{value:.2}");
    number.trim_end_matches('0').trim_end_matches('.').into()
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn unsupported(message: &str) -> RenderError {
    RenderError::UnsupportedFeature(format!("DOT {message}"))
}

#[cfg(test)]
mod tests {
    use super::HtmlTable;
    use crate::dot_runtime::Attributes;

    fn attrs() -> Attributes {
        [("shape".into(), "plaintext".into())].into()
    }

    #[test]
    fn should_size_single_cell_like_graphviz_htmltable() {
        // given
        let label = "<TABLE BORDER=\"1\"><TR><TD>alpha</TD></TR></TABLE>";
        // when
        let table = HtmlTable::parse(label, &attrs()).unwrap();
        // then
        assert_eq!(table.node_dimensions((54., 36.)), (58.3173828125, 36.8));
        assert!(
            table
                .svg((29.15869140625, -90.4))
                .contains("x=\"14\" y=\"-84.8\"")
        );
    }

    #[test]
    fn should_reject_nested_table_and_unmeasured_attributes() {
        // given
        let nested = "<TABLE><TR><TD><TABLE><TR><TD>x</TD></TR></TABLE></TD></TR></TABLE>";
        let colored = "<TABLE BGCOLOR=\"red\"><TR><TD>x</TD></TR></TABLE>";
        let wide_border = "<TABLE BORDER=\"2\"><TR><TD>x</TD></TR></TABLE>";
        // when
        let errors = [nested, colored, wide_border]
            .map(|label| HtmlTable::parse(label, &attrs()).err().unwrap());
        // then
        assert!(
            errors
                .iter()
                .all(|error| error.to_string().contains("unsupported"))
        );
    }
}
