//! How the host looks, as far as codeview cares.

use gpui::{App, FontWeight, Global, SharedString};

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
    /// the find box's matches and the current one, 0xRRGGBBAA
    pub find_match: u32,
    pub find_current: u32,
    /// the editor's change markers: bars on added and changed lines, triangles where lines were deleted
    pub change_added: u32,
    pub change_modified: u32,
    pub change_deleted: u32,
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
    /// the bar above a file's text saying it was deleted on disk, or why it could not be re-read
    pub editor_notice_padding_x: f32,
    pub editor_notice_padding_y: f32,
    pub editor_notice_text_size: f32,
    pub scrollbar_thickness: f32,
    pub scrollbar_min_thumb: f32,
    pub scrollbar_inset: f32,
    pub mono_font: SharedString,
    /// the size of the icons the views draw
    pub icon_size: f32,
    pub diff: DiffSizes,
}

/// Sizes of a diff view, in pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DiffSizes {
    pub row_height: f32,
    pub text_size: f32,
    /// before a code line's line numbers
    pub line_padding: f32,
    /// before a hunk header or a note
    pub hunk_padding_left: f32,
    /// a commit tab's file header
    pub header_gap: f32,
    pub header_padding_x: f32,
    pub header_weight: FontWeight,
    /// after each line number
    pub number_padding_right: f32,
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

    /// A style for tests that build views: plain colours and sizes, set as the global.
    #[cfg(test)]
    pub fn set_for_test(cx: &mut App) {
        cx.set_global(Self::for_test());
    }

    /// Plain colours and sizes, for tests.
    #[cfg(test)]
    pub fn for_test() -> Self {
        let diff = DiffSizes {
            row_height: 20.,
            text_size: 14.,
            line_padding: 4.,
            hunk_padding_left: 4.,
            header_gap: 4.,
            header_padding_x: 4.,
            header_weight: FontWeight::BOLD,
            number_padding_right: 4.,
        };
        ViewStyle {
            appearance: Appearance::Dark,
            text: 0xffffff,
            muted: 0x888888,
            background: 0x000000,
            border: 0x333333,
            hover: 0x222222,
            selected: 0x444444,
            accent: 0x3366ff,
            error: 0xff0000,
            sidebar: 0x111111,
            tab_active: 0x222222,
            tab_active_border: 0x3366ff,
            diff_added: 0x00ff0020,
            diff_removed: 0xff000020,
            find_match: 0xffc43c47,
            find_current: 0xe8a33dcc,
            change_added: 0x00ff00,
            change_modified: 0x0000ff,
            change_deleted: 0x888888,
            scrollbar_thumb: 0xffffff40,
            scrollbar_thumb_hover: 0xffffff80,
            panels_radius: 0.,
            panels_shown: false,
            editor_text_size: 14.,
            editor_line_height_factor: 1.4,
            editor_caret_width: 2.,
            editor_notice_padding_x: 4.,
            editor_notice_padding_y: 2.,
            editor_notice_text_size: 12.,
            scrollbar_thickness: 8.,
            scrollbar_min_thumb: 16.,
            scrollbar_inset: 2.,
            mono_font: "monospace".into(),
            icon_size: 14.,
            diff,
        }
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
