//! Packetdiag range/config semantics translated from authorized PlantUML MIT packetdiag sources.
//! Copyright 2009-2024/2009-2026 Arnaud Roques; kolulu23, Duy Nguyen, The-Lum.
//! MIT grant and precise per-file credits/hashes retained in provenance/.
use super::{
    budget::{Budget, bound, invalid, unsupported},
    source::Line,
};
use crate::RenderError;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Direction {
    Ltr,
    Rtl,
}
pub(super) struct Config {
    pub columns: usize,
    pub interval: usize,
    pub direction: Direction,
    pub node_height: usize,
    pub same_height: bool,
}
pub(super) struct Field {
    pub start: usize,
    pub end: usize,
    pub label: String,
    pub height: usize,
}
#[derive(Debug)]
pub(super) struct Fragment {
    pub field: usize,
    pub row: usize,
    pub offset: usize,
    pub width: usize,
}
pub(super) struct Packet {
    pub config: Config,
    pub fields: Vec<Field>,
    pub fragments: Vec<Fragment>,
    pub rows: usize,
}

pub(super) fn parse(lines: &[Line], budget: &mut Budget) -> Result<Packet, RenderError> {
    let mut config = Config {
        columns: 16,
        interval: 0,
        direction: Direction::Ltr,
        node_height: 32,
        same_height: false,
    };
    let mut fields = Vec::new();
    let mut braces = 0;
    let mut seen = false;
    for line in lines {
        budget.charge(line.text.len() + 1)?;
        let text = line.text.trim();
        if text.is_empty() {
            continue;
        }
        if text == "packetdiag {" || text == "packetdiag{" {
            if seen || braces != 0 {
                return Err(invalid(
                    line.number,
                    format!("unexpected packet opener {text:?}"),
                ));
            }
            braces = 1;
            seen = true;
            continue;
        }
        if text == "}" {
            if braces != 1 {
                return Err(invalid(line.number, "unexpected packet closing brace"));
            }
            braces = 2;
            continue;
        }
        if braces == 2 {
            return Err(invalid(
                line.number,
                format!("packet input after closing brace {text:?}"),
            ));
        }
        seen = true;
        if text.contains('=')
            && text
                .split_whitespace()
                .next()
                .unwrap_or("")
                .bytes()
                .next()
                .is_some_and(|c| c.is_ascii_alphabetic())
        {
            configure(&mut config, line, budget)?;
        } else {
            field(&mut fields, line, budget)?;
        }
    }
    if braces == 1 {
        return Err(invalid(1, "unterminated packetdiag brace"));
    }
    if fields.is_empty() {
        return Err(invalid(1, "packetdiag requires fields"));
    }
    if config.interval == 0 {
        config.interval = (config.columns / 2).max(1);
    }
    config.interval = config.interval.min(config.columns);
    let fragments = fragment(&fields, &config, budget)?;
    let rows = fragments.last().map_or(0, |f| f.row + 1);
    Ok(Packet {
        config,
        fields,
        fragments,
        rows,
    })
}

fn number(value: &str, line: usize) -> Result<usize, RenderError> {
    if value.is_empty() || value.len() > 7 || !value.bytes().all(|b| b.is_ascii_digit()) {
        return Err(invalid(line, format!("invalid packet integer {value:?}")));
    }
    value
        .parse()
        .map_err(|_| invalid(line, format!("invalid packet integer {value:?}")))
}
fn positive(value: &str, line: usize) -> Result<usize, RenderError> {
    let n = number(value, line)?;
    if n == 0 {
        return Err(invalid(
            line,
            format!("packet value must be positive: {value:?}"),
        ));
    }
    Ok(n)
}
fn configure(config: &mut Config, line: &Line, budget: &mut Budget) -> Result<(), RenderError> {
    let text = line.text.trim();
    let text = text.strip_suffix(';').unwrap_or(text);
    let Some((key, value)) = text.split_once('=') else {
        return Err(invalid(line.number, &line.text));
    };
    budget.item()?;
    let value = value.trim();
    match key.trim() {
        "colwidth" => {
            config.columns = positive(value, line.number)?;
            bound("packet colwidth", config.columns, 999)?;
        }
        "scale_interval" => config.interval = positive(value, line.number)?,
        "node_height" => {
            config.node_height = positive(value, line.number)?;
            bound("packet node height", config.node_height, 999)?;
        }
        "scale_direction" => {
            config.direction = match value {
                "ltr" => Direction::Ltr,
                "rtl" => Direction::Rtl,
                _ => {
                    return Err(invalid(
                        line.number,
                        format!("invalid packet direction {value:?}"),
                    ));
                }
            }
        }
        "same_height" => {
            config.same_height = match value {
                "true" => true,
                "false" => false,
                _ => {
                    return Err(invalid(
                        line.number,
                        format!("invalid packet same_height {value:?}"),
                    ));
                }
            }
        }
        _ => return Err(unsupported(line.number, &line.text)),
    }
    Ok(())
}
fn field(fields: &mut Vec<Field>, line: &Line, budget: &mut Budget) -> Result<(), RenderError> {
    let text = line.text.trim();
    let split = text.find(char::is_whitespace).ok_or_else(|| {
        invalid(
            line.number,
            format!("packet field needs label separator: {text:?}"),
        )
    })?;
    let token = &text[..split];
    let range = if token.starts_with('*') {
        token
    } else {
        token.strip_suffix(':').unwrap_or(token)
    };
    let (label, length, height) = attributes(text[split..].trim(), line.number, budget)?;
    bound("label bytes", label.len(), 16 * 1024)?;
    let (start, end) = if let Some((a, b)) = range.split_once('-') {
        if length != 0 {
            return Err(invalid(
                line.number,
                "len attribute cannot override explicit packet range",
            ));
        }
        (number(a, line.number)?, number(b, line.number)?)
    } else {
        let start = if range == "*" {
            fields.last().map_or(0, |f| f.end + 1)
        } else {
            number(range, line.number)?
        };
        let end = start
            .checked_add(length.max(1) - 1)
            .ok_or_else(|| invalid(line.number, "packet end overflow"))?;
        (start, end)
    };
    if end < start || end > 9_999_999 {
        return Err(invalid(
            line.number,
            format!("invalid inclusive packet range {start}-{end}"),
        ));
    }
    budget.item()?;
    budget.text(label.len())?;
    fields.push(Field {
        start,
        end,
        label: label.into(),
        height,
    });
    Ok(())
}
fn attributes<'a>(
    text: &'a str,
    line: usize,
    budget: &mut Budget,
) -> Result<(&'a str, usize, usize), RenderError> {
    let Some(open) = text.find('[') else {
        return Ok((text, 0, 1));
    };
    if !text.ends_with(']') {
        return Err(invalid(
            line,
            format!("malformed packet attributes {text:?}"),
        ));
    }
    let mut length = 0;
    let mut height = 0;
    for entry in text[open + 1..text.len() - 1].split(',') {
        budget.charge(entry.len() + 1)?;
        budget.item()?;
        let Some((key, value)) = entry.split_once('=') else {
            return Err(invalid(
                line,
                format!("malformed packet attribute {entry:?}"),
            ));
        };
        match key.trim() {
            "len" if length == 0 => length = positive(value.trim(), line)?,
            "height" if height == 0 => height = positive(value.trim(), line)?,
            _ => return Err(unsupported(line, entry)),
        }
    }
    Ok((text[..open].trim_end(), length, height.max(1)))
}
fn fragment(
    fields: &[Field],
    config: &Config,
    budget: &mut Budget,
) -> Result<Vec<Fragment>, RenderError> {
    let mut result = Vec::new();
    let mut row = 0;
    let mut offset = 0;
    for (id, field) in fields.iter().enumerate() {
        let mut remaining = field.end - field.start + 1;
        while remaining > 0 {
            budget.item()?;
            let width = remaining.min(config.columns - offset);
            result.push(Fragment {
                field: id,
                row,
                offset,
                width,
            });
            remaining -= width;
            offset += width;
            if offset == config.columns {
                row += 1;
                offset = 0;
            }
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RenderOptions;

    #[test]
    fn should_reject_oversized_packet_label_before_fragment_generation() {
        // given
        let lines = vec![Line {
            number: 3,
            text: format!("0: {}", "x".repeat(16_385)),
        }];
        let mut budget = Budget::new(&RenderOptions::default());
        // when
        let result = parse(&lines, &mut budget);
        // then
        assert!(
            result.is_err(),
            "oversized field retained before fragmentation"
        );
    }
}
