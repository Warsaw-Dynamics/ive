use super::{
    budget::{Budget, bound},
    scene::{Decoration, Scene, Shape},
};
use crate::RenderError;

pub(super) fn serialize(scene: &Scene, budget: &mut Budget) -> Result<String, RenderError> {
    let mut writer = Writer {
        value: String::new(),
        limit: budget.limits.svg_bytes,
    };
    let width = scene.width - scene.min_x;
    let height = scene.height - scene.min_y;
    writer.add(&format!("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}\" height=\"{height}\" viewBox=\"{} {} {width} {height}\"><rect x=\"{}\" y=\"{}\" width=\"{width}\" height=\"{height}\" fill=\"white\"/>", scene.min_x, scene.min_y, scene.min_x, scene.min_y))?;
    for element in &scene.elements {
        budget.charge(1)?;
        writer.add("<g stroke=\"#334155\" stroke-width=\"1.5\" stroke-linejoin=\"round\"")?;
        for (name, value) in [
            ("id", element.id.as_str()),
            ("data-kind", element.kind),
            ("data-name", &element.name),
            ("data-owner", &element.owner),
            ("data-target", &element.target),
        ] {
            writer.attribute(name, value)?;
        }
        writer.add(">")?;
        for shape in &element.shapes {
            budget.charge(1)?;
            writer.shape(shape)?;
        }
        writer.add("</g>")?;
    }
    writer.add("</svg>")?;
    Ok(writer.value)
}

struct Writer {
    value: String,
    limit: usize,
}
impl Writer {
    fn add(&mut self, value: &str) -> Result<(), RenderError> {
        bound(
            "SVG bytes",
            self.value.len().saturating_add(value.len()),
            self.limit,
        )?;
        self.value.push_str(value);
        Ok(())
    }
    fn escaped(&mut self, text: &str) -> Result<(), RenderError> {
        for ch in text.chars() {
            match ch {
                '&' => self.add("&amp;")?,
                '<' => self.add("&lt;")?,
                '>' => self.add("&gt;")?,
                '"' => self.add("&quot;")?,
                '\'' => self.add("&apos;")?,
                _ => {
                    let mut bytes = [0; 4];
                    self.add(ch.encode_utf8(&mut bytes))?;
                }
            }
        }
        Ok(())
    }
    fn attribute(&mut self, name: &str, value: &str) -> Result<(), RenderError> {
        self.add(&format!(" {name}=\""))?;
        self.escaped(value)?;
        self.add("\"")
    }
    fn shape(&mut self, shape: &Shape) -> Result<(), RenderError> {
        match shape {
            Shape::Rect(r, radius, fill) => self.add(&format!(
                "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{radius}\" fill=\"{}\"/>",
                r.x,
                r.y,
                r.width,
                r.height,
                fill.value()
            )),
            Shape::Ellipse(r, fill) => self.add(&format!(
                "<ellipse cx=\"{}\" cy=\"{}\" rx=\"{}\" ry=\"{}\" fill=\"{}\"/>",
                r.x + r.width / 2.,
                r.y + r.height / 2.,
                r.width / 2.,
                r.height / 2.,
                fill.value()
            )),
            Shape::Line(points, dash) => {
                self.add(&format!(
                    "<polyline fill=\"none\" stroke-dasharray=\"{}\" points=\"",
                    match dash {
                        super::scene::Dash::Solid => "none",
                        _ => "6 4",
                    }
                ))?;
                self.points(points)?;
                self.add("\"/>")
            }
            Shape::Polygon(points, fill) => {
                self.add(&format!("<polygon fill=\"{}\" points=\"", fill.value()))?;
                self.points(points)?;
                self.add("\"/>")
            }
            Shape::Text(p, label, decoration) => {
                for (i, line) in label.lines.iter().enumerate() {
                    self.add(&format!("<text x=\"{}\" y=\"{}\" fill=\"#172033\" stroke=\"none\" font-family=\"{}\" font-size=\"14\" text-decoration=\"{}\" font-style=\"{}\" xml:space=\"preserve\">", p.x, p.y + label.line_height * (i+1) as f64 - 14./4.5, label.font_family, match decoration { Decoration::Underlined => "underline", _ => "none" }, match decoration { Decoration::Italic => "italic", _ => "normal" }))?;
                    self.escaped(line)?;
                    self.add("</text>")?;
                }
                Ok(())
            }
        }
    }
    fn points(&mut self, points: &[super::scene::Point]) -> Result<(), RenderError> {
        for p in points {
            self.add(&format!("{},{} ", p.x, p.y))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_include_negative_scene_coordinates_in_the_canvas() {
        use super::super::scene::{Element, Paint, Rect};
        // given
        let mut budget = Budget::new(&crate::RenderOptions::default());
        let mut scene = Scene::new();
        let mut element = Element::new("negative", "test");
        element.shapes.push(Shape::Rect(
            Rect {
                x: -40.,
                y: -30.,
                width: 20.,
                height: 20.,
            },
            0.,
            Paint::Paper,
        ));
        scene.push(element, &mut budget).unwrap();
        // when
        let svg = serialize(&scene, &mut budget).unwrap();
        let xml = roxmltree::Document::parse(&svg).unwrap();
        let bounds = xml
            .root_element()
            .attribute("viewBox")
            .unwrap()
            .split_whitespace()
            .map(|s| s.parse::<f64>().unwrap())
            .collect::<Vec<_>>();
        // then
        assert!(bounds[0] <= -40. && bounds[1] <= -30.);
        assert!(bounds[0] + bounds[2] >= -20. && bounds[1] + bounds[3] >= -10.);
    }

    #[test]
    fn should_reject_svg_growth_before_appending_or_allocating() {
        // given
        let mut writer = Writer {
            value: "abc".into(),
            limit: 5,
        };
        let capacity = writer.value.capacity();
        // when
        let result = writer.add("oversized");
        // then
        assert!(result.is_err());
        assert_eq!(writer.value, "abc");
        assert_eq!(writer.value.capacity(), capacity);
    }
}
