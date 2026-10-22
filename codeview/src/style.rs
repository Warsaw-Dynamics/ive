//! How the host looks, as far as codeview cares.

use gpui::{App, Global, SharedString};

/// Dark or light: picks the colour scheme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Appearance {
    Dark,
    Light,
}

/// The colours (0xRRGGBB, 0xRRGGBBAA where noted) and sizes (pixels) the views draw with. The host sets it as a gpui
/// global, and again whenever its theme changes; cheap to clone.
#[derive(Debug, Clone, PartialEq)]
pub struct ViewStyle {
    pub appearance: Appearance,
    pub text: u32,
    pub muted: u32,
    pub background: u32,
    pub border: u32,
    pub hover: u32,
    pub selected: u32,
    pub accent: u32,
    pub error: u32,
    pub sidebar: u32,
    pub tab_active: u32,
    pub tab_active_border: u32,
    /// backgrounds of added and removed diff lines, 0xRRGGBBAA
    pub diff_added: u32,
    pub diff_removed: u32,
    /// 0xRRGGBBAA: translucent so the thumb reads over any background
    pub scrollbar_thumb: u32,
    pub scrollbar_thumb_hover: u32,
    /// the host's rounded panels: their corner radius, and whether they are drawn at all
    pub panels_radius: f32,
    pub panels_shown: bool,
    /// the editor's text size, and how many text sizes apart its lines are
    pub editor_text_size: f32,
    pub editor_line_height_factor: f32,
    pub editor_caret_width: f32,
    pub scrollbar_thickness: f32,
    pub scrollbar_min_thumb: f32,
    pub scrollbar_inset: f32,
    pub mono_font: SharedString,
}

impl Global for ViewStyle {}

/// The style of the panel a window is drawing, set by the host before it draws the panel; see [`ViewStyle::drawing`].
#[derive(Debug, Clone, PartialEq)]
pub struct PanelStyle(pub ViewStyle);

impl Global for PanelStyle {}

impl ViewStyle {
    /// This style drawing code in `font`.
    pub fn with_mono_font(self, font: SharedString) -> Self {
        Self { mono_font: font, ..self }
    }

    pub fn get(cx: &App) -> &ViewStyle {
        cx.global::<ViewStyle>()
    }

    /// The style of the panel being drawn when the host sets one (an unfocused panel draws in other colours), else
    /// the global one.
    pub fn drawing(cx: &App) -> &ViewStyle {
        cx.try_global::<PanelStyle>().map_or_else(|| cx.global::<ViewStyle>(), |panel| &panel.0)
    }
}
