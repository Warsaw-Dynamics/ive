//! The theme's leaf values, each checked as it is read so a bad one is reported with its key path.

use std::cell::Cell;
use std::fmt;

use gpui::{FontWeight, Pixels, px};
use serde::de::{self, Deserializer, Visitor};
use serde::{Deserialize, Serialize};

/// Larger than any sensible row, gap or panel; a larger size is a mistake.
const MAX_PX: f32 = 2000.;
const FONT_PX: std::ops::RangeInclusive<f32> = 6.0..=96.0;

/// An opaque colour, 0xRRGGBB, written `"#RRGGBB"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb(pub u32);

/// A colour with alpha, 0xRRGGBBAA, written `"#RRGGBBAA"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgba(pub u32);

/// A size in pixels, 0 to 2000.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Px(pub f32);

/// A height in pixels, 1 to 2000: a row, a bar or a field, which may never collapse to nothing (a list of
/// zero-height rows would draw every row each frame).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Height(pub f32);

impl Height {
    pub fn px(self) -> Pixels {
        px(self.0)
    }
}

impl<'de> Deserialize<'de> for Height {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        length(deserializer, 1.0..=MAX_PX, "a height in pixels").map(Height)
    }
}

/// A font size in pixels, 6 to 96.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct FontPx(pub f32);

/// What `"1rem"` is, in pixels, 8 to 32: `sizes.rem`, which zooms every size written in rem.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct RootPx(pub f32);

impl Default for RootPx {
    fn default() -> Self {
        Self(16.)
    }
}

impl RootPx {
    pub fn px(self) -> Pixels {
        px(self.0)
    }
}

impl<'de> Deserialize<'de> for RootPx {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let number = f64::deserialize(deserializer)?;
        if number.is_finite() && (8.0..=32.0).contains(&number) {
            return Ok(RootPx(number as f32));
        }
        Err(de::Error::custom(format!("expected a root size from 8 to 32 px, not {number}")))
    }
}

/// A share from 0 to 1, such as an opacity or a blend.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Fraction(pub f32);

/// A multiple from 0.5 to 4, such as a line height in font sizes.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Factor(pub f32);

/// The gamma text is drawn with, from 1 (none) to 2.2.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Gamma(pub f32);

/// Extra weight for text darker than 75% brightness, from 0 (none) to 4.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Contrast(pub f32);

impl Px {
    pub fn px(self) -> Pixels {
        px(self.0)
    }
}

impl FontPx {
    pub fn px(self) -> Pixels {
        px(self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Weight {
    Normal,
    Medium,
    Semibold,
    Bold,
}

impl Weight {
    pub fn font_weight(self) -> FontWeight {
        match self {
            Self::Normal => FontWeight::NORMAL,
            Self::Medium => FontWeight::MEDIUM,
            Self::Semibold => FontWeight::SEMIBOLD,
            Self::Bold => FontWeight::BOLD,
        }
    }
}

/// How a tree row in one state looks; what it leaves unset comes from the states below it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RowStyle {
    pub text: Option<Rgb>,
    pub background: Option<Rgba>,
    pub weight: Option<Weight>,
    pub italic: Option<bool>,
}

/// `#` and `digits` hex digits, as a number.
fn hex(text: &str, digits: usize) -> Option<u32> {
    let body = text.strip_prefix('#').filter(|body| body.len() == digits && body.chars().all(|c| c.is_ascii_hexdigit()))?;
    u32::from_str_radix(body, 16).ok()
}

struct ColourVisitor {
    digits: usize,
    form: &'static str,
}

impl Visitor<'_> for ColourVisitor {
    type Value = u32;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a colour {:?}", self.form)
    }

    fn visit_str<E: de::Error>(self, text: &str) -> Result<u32, E> {
        hex(text, self.digits).ok_or_else(|| E::custom(format!("expected a colour {:?}, not {text:?}", self.form)))
    }
}

impl<'de> Deserialize<'de> for Rgb {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_str(ColourVisitor { digits: 6, form: "#RRGGBB" }).map(Rgb)
    }
}

impl<'de> Deserialize<'de> for Rgba {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_str(ColourVisitor { digits: 8, form: "#RRGGBBAA" }).map(Rgba)
    }
}

thread_local! {
    /// What `"1rem"` is while sizes are read; `None` where rem is not allowed.
    static ROOT: Cell<Option<f32>> = const { Cell::new(None) };
}

/// Runs `read` with sizes written in rem resolved against `rem` px; `None` rejects rem.
pub(super) fn with_rem<T>(rem: Option<f32>, read: impl FnOnce() -> T) -> T {
    struct Restore(Option<f32>);
    impl Drop for Restore {
        fn drop(&mut self) {
            ROOT.with(|root| root.set(self.0));
        }
    }
    let _restore = Restore(ROOT.with(|root| root.replace(rem)));
    read()
}

/// `n` rem in whole pixels; a size above zero never rounds away to nothing.
pub(super) fn rem_px(n: f32, rem: f32) -> f32 {
    if n == 0. {
        return 0.;
    }
    (n * rem).round().max(1.)
}

/// A size as written: a number of pixels, `"Npx"`, or `"Nrem"` resolved to pixels.
struct Length {
    px: f64,
    /// the string and the root size, for a value written in rem
    rem: Option<(String, f32)>,
}

struct LengthVisitor;

impl Visitor<'_> for LengthVisitor {
    type Value = Length;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(r#"a size like 13, "13px" or "0.8125rem""#)
    }

    fn visit_f64<E: de::Error>(self, number: f64) -> Result<Length, E> {
        Ok(Length { px: number, rem: None })
    }

    fn visit_i64<E: de::Error>(self, number: i64) -> Result<Length, E> {
        Ok(Length { px: number as f64, rem: None })
    }

    fn visit_u64<E: de::Error>(self, number: u64) -> Result<Length, E> {
        Ok(Length { px: number as f64, rem: None })
    }

    fn visit_str<E: de::Error>(self, text: &str) -> Result<Length, E> {
        let bad = || E::custom(format!(r#"expected a size like 13, "13px" or "0.8125rem", not {text:?}"#));
        if let Some(number) = text.strip_suffix("rem") {
            let rem = ROOT.with(Cell::get).ok_or_else(|| E::custom(format!("rem sizes are only allowed under theme.sizes, not {text:?}")))?;
            let n = unsigned(number).ok_or_else(bad)?;
            return Ok(Length { px: f64::from(rem_px(n as f32, rem)), rem: Some((text.to_owned(), rem)) });
        }
        let number = text.strip_suffix("px").and_then(unsigned).ok_or_else(bad)?;
        Ok(Length { px: number, rem: None })
    }
}

/// Digits with at most one `.`: no sign, exponent or spaces.
fn unsigned(text: &str) -> Option<f64> {
    let plain = !text.is_empty() && text.chars().all(|c| c.is_ascii_digit() || c == '.') && text.matches('.').count() <= 1;
    plain.then(|| text.parse().ok()).flatten()
}

/// A size in `range` pixels, written as a number, `"Npx"` or `"Nrem"`; a range error names a rem value as written.
fn length<'de, D: Deserializer<'de>>(deserializer: D, range: std::ops::RangeInclusive<f32>, what: &str) -> Result<f32, D::Error> {
    let length = deserializer.deserialize_any(LengthVisitor)?;
    let (low, high) = (range.start(), range.end());
    if length.px.is_finite() && (f64::from(*low)..=f64::from(*high)).contains(&length.px) {
        return Ok(length.px as f32);
    }
    let written = length.rem.map_or_else(String::new, |(text, rem)| format!(" ({text:?} at rem {rem})"));
    Err(de::Error::custom(format!("expected {what} from {low} to {high}, not {}{written}", length.px)))
}

/// A number in `range`, or an error naming `what` and the number.
fn bounded<'de, D: Deserializer<'de>>(deserializer: D, range: std::ops::RangeInclusive<f32>, what: &str) -> Result<f32, D::Error> {
    let number = f64::deserialize(deserializer)?;
    let (low, high) = (range.start(), range.end());
    if number.is_finite() && (f64::from(*low)..=f64::from(*high)).contains(&number) {
        return Ok(number as f32);
    }
    Err(de::Error::custom(format!("expected {what} from {low} to {high}, not {number}")))
}

impl<'de> Deserialize<'de> for Px {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        length(deserializer, 0.0..=MAX_PX, "a size in pixels").map(Px)
    }
}

impl<'de> Deserialize<'de> for FontPx {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        length(deserializer, FONT_PX, "a font size in pixels").map(FontPx)
    }
}

impl<'de> Deserialize<'de> for Factor {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        bounded(deserializer, 0.5..=4.0, "a factor").map(Factor)
    }
}

impl<'de> Deserialize<'de> for Gamma {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        bounded(deserializer, 1.0..=2.2, "a gamma").map(Gamma)
    }
}

impl<'de> Deserialize<'de> for Contrast {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        bounded(deserializer, 0.0..=4.0, "a contrast").map(Contrast)
    }
}

impl<'de> Deserialize<'de> for Fraction {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        bounded(deserializer, 0.0..=1.0, "a share").map(Fraction)
    }
}
