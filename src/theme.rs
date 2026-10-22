use std::sync::Mutex;

use gpui::{App, FocusHandle, FontFallbacks, Global, SharedString, Window, WindowAppearance};

use crate::settings::{SettingsStore, ThemePreference};
use crate::theme_config::{GitColors, Palette, Panels, Sizes, ThemeSpec, TreeStates, VirtualFolders};

pub use codeview::style::{Appearance, ViewStyle};

/// `preference`, with the OS's appearance for [`ThemePreference::System`].
pub fn appearance(preference: ThemePreference, os: WindowAppearance) -> Appearance {
    match (preference, os) {
        (ThemePreference::Dark, _) => Appearance::Dark,
        (ThemePreference::Light, _) => Appearance::Light,
        (ThemePreference::System, WindowAppearance::Dark | WindowAppearance::VibrantDark) => Appearance::Dark,
        (ThemePreference::System, WindowAppearance::Light | WindowAppearance::VibrantLight) => Appearance::Light,
    }
}

/// The active theme: colours as 0xRRGGBB (0xRRGGBBAA where noted) for the current appearance, and the sizes; built
/// from the [`ThemeSpec`] that `ive.rn` and `init.rn` left.
#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub appearance: Appearance,
    pub background: u32,
    pub sidebar: u32,
    /// the projects panel, as IntelliJ's project tool window
    pub tree: u32,
    pub border: u32,
    pub text: u32,
    pub muted: u32,
    pub icon: u32,
    pub accent: u32,
    pub accent_hover: u32,
    pub on_accent: u32,
    pub selected: u32,
    pub hover: u32,
    pub error: u32,
    /// backgrounds of added and removed diff lines, 0xRRGGBBAA
    pub diff_added: u32,
    pub diff_removed: u32,
    /// change marks: A and ?, D and U, the rest
    pub mark_added: u32,
    pub mark_removed: u32,
    pub mark_modified: u32,
    /// background of the window close button under the mouse
    pub close_hover: u32,
    /// 0xRRGGBBAA: translucent so the thumb reads over any background
    pub scrollbar_thumb: u32,
    pub scrollbar_thumb_hover: u32,
    /// the shown editor tab: fill and 1 px outline, as the IntelliJ New UI chip
    pub tab_active: u32,
    pub tab_active_border: u32,
    /// 0xRRGGBBAA: behind prompts, and the window's shadow
    pub scrim: u32,
    pub shadow: u32,
    /// behind rounded panels
    pub frame: u32,
    pub checkout_hues: [u32; 8],
    pub glow: [u32; 9],
    pub terminal: TerminalPalette,
    pub git: GitColors,
    pub tree_states: TreeStates,
    pub virtual_folders: VirtualFolders,
    pub panels: Panels,
    pub sizes: &'static Sizes,
}

impl Global for Theme {}

/// Colours of the terminal tab, as 0xRRGGBB.
#[derive(Debug, Clone, Copy)]
pub struct TerminalPalette {
    /// black, red, green, yellow, blue, magenta, cyan, white, then the same eight bright
    pub ansi: [u32; 16],
    pub foreground: u32,
    pub background: u32,
    pub cursor: u32,
    pub selection: u32,
}

#[cfg(test)]
impl Theme {
    /// The default theme in `appearance`.
    pub fn of(appearance: Appearance) -> Self {
        from_spec(&crate::theme_config::defaults(), appearance)
    }
}

/// `spec`'s colours for `appearance`, and its sizes.
pub fn from_spec(spec: &ThemeSpec, appearance: Appearance) -> Theme {
    let palette = match appearance {
        Appearance::Dark => &spec.dark,
        Appearance::Light => &spec.light,
    };
    from_palette(spec, palette, appearance)
}

/// [`from_spec`] with `spec`'s unfocused palette: what a panel draws with while it does not have the keyboard.
pub fn unfocused_from_spec(spec: &ThemeSpec, appearance: Appearance) -> Theme {
    let palette = match appearance {
        Appearance::Dark => &spec.unfocused.dark,
        Appearance::Light => &spec.unfocused.light,
    };
    from_palette(spec, palette, appearance)
}

/// The theme drawn while the window is not active or the panel does not have the keyboard.
#[derive(Debug, Clone, Copy)]
pub struct UnfocusedTheme(pub Theme);

impl Global for UnfocusedTheme {}

/// Where something is drawn, for which theme it takes: the sidebar, the tab row and the shown tab (in a detached
/// tab's window too), or the window around both.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    Window,
    Sidebar,
    Tabs,
}

/// Whether `panel` has the keyboard: the window is active, and the keys are in the sidebar for the sidebar or
/// anywhere else for the tabs; the window itself only needs to be active.
pub fn has_keys(panel: Panel, window_active: bool, sidebar_has_keys: bool) -> bool {
    window_active
        && match panel {
            Panel::Window => true,
            Panel::Sidebar => sidebar_has_keys,
            Panel::Tabs => !sidebar_has_keys,
        }
}

/// The sidebar's focus handle, so any view can tell whether the sidebar has the keyboard.
pub struct SidebarFocus(pub FocusHandle);

impl Global for SidebarFocus {}

/// The theme `panel` draws with in `window` now: the focused one while it has the keyboard, else the unfocused one.
pub fn for_panel(panel: Panel, window: &Window, cx: &App) -> Theme {
    let sidebar_has_keys = cx.try_global::<SidebarFocus>().is_some_and(|sidebar| sidebar.0.contains_focused(window, cx));
    if has_keys(panel, window.is_window_active(), sidebar_has_keys) {
        return *cx.global::<Theme>();
    }
    cx.try_global::<UnfocusedTheme>().map_or_else(|| *cx.global::<Theme>(), |unfocused| unfocused.0)
}

/// The theme of the tab area in the window being drawn: set by each window's root view as it renders, read by
/// the views drawn inside it, which have no window at hand. Windows draw one at a time.
#[derive(Debug, Clone, Copy)]
pub struct TabsTheme(pub Theme);

impl Global for TabsTheme {}

/// Sets [`TabsTheme`] for `window`, which is about to be drawn, and returns it.
pub fn set_tabs(window: &Window, cx: &mut App) -> Theme {
    let theme = for_panel(Panel::Tabs, window, cx);
    cx.set_global(TabsTheme(theme));
    cx.set_global(codeview::style::PanelStyle(view_style(&theme).with_mono_font(mono_font(cx))));
    theme
}

/// The tab area's theme in the window being drawn; the focused theme outside a draw.
pub fn tabs(cx: &App) -> Theme {
    cx.try_global::<TabsTheme>().map_or_else(|| *cx.global::<Theme>(), |tabs| tabs.0)
}

fn from_palette(spec: &ThemeSpec, palette: &Palette, appearance: Appearance) -> Theme {
    let terminal = &palette.terminal;
    Theme {
        appearance,
        background: palette.background.0,
        sidebar: palette.sidebar.0,
        tree: palette.tree.0,
        border: palette.border.0,
        text: palette.text.0,
        muted: palette.muted.0,
        icon: palette.icon.0,
        accent: palette.accent.0,
        accent_hover: palette.accent_hover.0,
        on_accent: palette.on_accent.0,
        selected: palette.selected.0,
        hover: palette.hover.0,
        error: palette.error.0,
        diff_added: palette.diff_added.0,
        diff_removed: palette.diff_removed.0,
        mark_added: palette.mark_added.0,
        mark_removed: palette.mark_removed.0,
        mark_modified: palette.mark_modified.0,
        close_hover: palette.close_hover.0,
        scrollbar_thumb: palette.scrollbar_thumb.0,
        scrollbar_thumb_hover: palette.scrollbar_thumb_hover.0,
        tab_active: palette.tab_active.0,
        tab_active_border: palette.tab_active_border.0,
        scrim: palette.scrim.0,
        shadow: palette.shadow.0,
        frame: palette.frame.0,
        checkout_hues: palette.checkout_hues.map(|hue| hue.0),
        glow: palette.glow.map(|colour| colour.0),
        terminal: TerminalPalette {
            ansi: terminal.ansi.map(|colour| colour.0),
            foreground: terminal.foreground.0,
            background: terminal.background.0,
            cursor: terminal.cursor.0,
            selection: terminal.selection.0,
        },
        git: palette.git,
        tree_states: palette.tree_states,
        virtual_folders: palette.virtual_folders,
        panels: spec.panels,
        sizes: intern(&spec.sizes),
    }
}

/// What the code views draw with: `theme`'s colours and sizes they read, in the default monospace font until
/// [`ViewStyle::with_mono_font`] names the one in use.
pub fn view_style(theme: &Theme) -> ViewStyle {
    let (editor, controls, diff) = (&theme.sizes.editor, &theme.sizes.controls, &theme.sizes.diff);
    ViewStyle {
        appearance: theme.appearance,
        text: theme.text,
        muted: theme.muted,
        background: theme.background,
        border: theme.border,
        hover: theme.hover,
        selected: theme.selected,
        accent: theme.accent,
        error: theme.error,
        sidebar: theme.sidebar,
        tab_active: theme.tab_active,
        tab_active_border: theme.tab_active_border,
        diff_added: theme.diff_added,
        diff_removed: theme.diff_removed,
        scrollbar_thumb: theme.scrollbar_thumb,
        scrollbar_thumb_hover: theme.scrollbar_thumb_hover,
        panels_radius: theme.panels.radius.0,
        panels_shown: theme.panels.shown(),
        editor_text_size: editor.text_size.0,
        editor_line_height_factor: editor.line_height_factor.0,
        editor_caret_width: editor.caret_width.0,
        editor_notice_padding_x: editor.notice_padding_x.0,
        editor_notice_padding_y: editor.notice_padding_y.0,
        editor_notice_text_size: editor.notice_text_size.0,
        scrollbar_thickness: controls.scrollbar_thickness.0,
        scrollbar_min_thumb: controls.scrollbar_min_thumb.0,
        scrollbar_inset: controls.scrollbar_inset.0,
        mono_font: MONOSPACE.into(),
        icon_size: theme.sizes.icon.0,
        diff: codeview::style::DiffSizes {
            row_height: diff.row_height.0,
            text_size: diff.text_size.0,
            line_padding: diff.line_padding.0,
            hunk_padding_left: diff.hunk_padding_left.0,
            header_gap: diff.header_gap.0,
            header_padding_x: diff.header_padding_x.0,
            header_weight: diff.header_weight.font_weight(),
            number_padding_right: diff.number_padding_right.0,
        },
    }
}

/// `sizes` as a reference that lives as long as IVE, so [`Theme`] stays `Copy`; each distinct value is kept once, so
/// saving `init.rn` again and again does not grow memory.
pub fn intern(sizes: &Sizes) -> &'static Sizes {
    static INTERNED: Mutex<Vec<&'static Sizes>> = Mutex::new(Vec::new());
    let mut interned = INTERNED.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(known) = interned.iter().find(|known| **known == sizes) {
        return known;
    }
    let leaked: &'static Sizes = Box::leak(Box::new(sizes.clone()));
    interned.push(leaked);
    leaked
}

/// Re-resolves the theme from the saved preference and `os` appearance, and redraws if it changed.
/// Inside window callbacks pass `window.appearance()`: on X11 `cx.window_appearance()` there panics
/// (gpui 0.2.2 re-borrows platform state the event loop already holds).
pub fn apply(os: WindowAppearance, cx: &mut App) {
    let preference = cx.global::<SettingsStore>().current().theme;
    let appearance = appearance(preference, os);
    if cx.try_global::<Theme>().is_some_and(|theme| theme.appearance == appearance) {
        return;
    }
    set(appearance, cx);
}

/// Rebuilds the theme in the current appearance after the [`ThemeSpec`] changed, and redraws.
pub fn reapply(cx: &mut App) {
    let appearance = cx.global::<Theme>().appearance;
    set(appearance, cx);
}

fn set(appearance: Appearance, cx: &mut App) {
    let theme = from_spec(cx.global::<ThemeSpec>(), appearance);
    let unfocused = unfocused_from_spec(cx.global::<ThemeSpec>(), appearance);
    cx.set_global(view_style(&theme).with_mono_font(mono_font(cx)));
    cx.set_global(theme);
    cx.set_global(UnfocusedTheme(unfocused));
    crate::colors::apply(cx);
    cx.refresh_windows();
}

/// IntelliJ's New UI fonts, bundled under `assets/fonts` so every OS renders the same; `ive.rn` names them too.
const UI_FONT: &str = "Inter";
const MONOSPACE: &str = "JetBrains Mono";

/// The font families in use: `config.ui_font` and `config.font` from `ive.rn` and `init.rn`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fonts {
    pub ui: SharedString,
    pub mono: SharedString,
}

impl Global for Fonts {}

impl Fonts {
    /// These fonts with each family not in `installed` replaced by `fallback`'s, and the families replaced.
    pub fn installed_or(self, fallback: &Fonts, installed: &[String]) -> (Fonts, Vec<SharedString>) {
        let is_installed = |family: &SharedString| installed.iter().any(|name| name == family.as_ref());
        let missing = [&self.ui, &self.mono].into_iter().filter(|family| !is_installed(family)).cloned().collect();
        let ui = if is_installed(&self.ui) { self.ui } else { fallback.ui.clone() };
        let mono = if is_installed(&self.mono) { self.mono } else { fallback.mono.clone() };
        (Fonts { ui, mono }, missing)
    }
}

impl Default for Fonts {
    fn default() -> Self {
        Self { ui: UI_FONT.into(), mono: MONOSPACE.into() }
    }
}

pub fn ui_font(cx: &App) -> SharedString {
    cx.global::<Fonts>().ui.clone()
}

pub fn mono_font(cx: &App) -> SharedString {
    cx.global::<Fonts>().mono.clone()
}

/// Tried before the system's own fallback for characters the font lacks, so symbols with a colour emoji form, such as
/// Claude Code's `✳`, draw in colour as terminals draw them rather than from a monochrome symbol font.
pub fn emoji_fallbacks() -> FontFallbacks {
    #[cfg(target_os = "macos")]
    let font = "Apple Color Emoji";
    #[cfg(target_os = "windows")]
    let font = "Segoe UI Emoji";
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let font = "Noto Color Emoji";
    FontFallbacks::from_fonts(vec![font.to_owned()])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme_config::{Rgb, defaults};

    #[test]
    fn should_give_keys_to_one_panel_at_a_time() {
        // when
        let cases = [(true, true), (true, false), (false, true), (false, false)].map(|(active, sidebar)| {
            (has_keys(Panel::Sidebar, active, sidebar), has_keys(Panel::Tabs, active, sidebar), has_keys(Panel::Window, active, sidebar))
        });

        // then
        assert_eq!(cases, [(true, false, true), (false, true, true), (false, false, false), (false, false, false)]);
    }

    #[test]
    fn should_build_unfocused_theme_from_unfocused_palette() {
        // when
        let theme = unfocused_from_spec(&defaults(), Appearance::Dark);

        // then
        assert_eq!((theme.selected, theme.tab_active, theme.tree), (0x33353b, 0x26282c, 0x191a1c));
    }

    #[test]
    fn should_follow_os_only_when_preference_is_system() {
        // given
        let cases = [
            (ThemePreference::System, WindowAppearance::VibrantLight, Appearance::Light),
            (ThemePreference::System, WindowAppearance::Dark, Appearance::Dark),
            (ThemePreference::Dark, WindowAppearance::Light, Appearance::Dark),
            (ThemePreference::Light, WindowAppearance::Dark, Appearance::Light),
        ];

        // when
        let resolved: Vec<_> = cases.iter().map(|(preference, os, _)| appearance(*preference, *os)).collect();

        // then
        let expected: Vec<_> = cases.iter().map(|(_, _, appearance)| *appearance).collect();
        assert_eq!(resolved, expected);
    }

    #[test]
    fn should_build_dark_theme_from_spec() {
        // when
        let theme = from_spec(&defaults(), Appearance::Dark);

        // then
        assert_eq!((theme.tree, theme.background, theme.terminal.cursor), (0x191a1c, 0x1e1f22, 0xced0d6));
    }

    #[test]
    fn should_keep_overrides_when_appearance_changes() {
        // given
        let mut spec = defaults();
        spec.dark.tree = Rgb(0x202124);
        spec.light.tree = Rgb(0xfafafa);

        // when
        let (dark, light) = (from_spec(&spec, Appearance::Dark), from_spec(&spec, Appearance::Light));

        // then
        assert_eq!((dark.tree, light.tree), (0x202124, 0xfafafa));
    }

    #[test]
    fn should_intern_equal_sizes_once() {
        // given
        let sizes = defaults().sizes;

        // when
        let (first, second) = (intern(&sizes), intern(&sizes.clone()));

        // then
        assert!(std::ptr::eq(first, second));
    }

    #[test]
    fn should_build_view_style_from_theme() {
        // given
        let theme = from_spec(&defaults(), Appearance::Dark);

        // when
        let style = view_style(&theme);

        // then
        assert_eq!(style.text, theme.text);
        assert_eq!(style.diff_added, theme.diff_added);
        assert_eq!(style.editor_text_size, theme.sizes.editor.text_size.0);
    }

    #[test]
    fn should_keep_fallback_for_font_not_installed() {
        // given
        let wanted = Fonts { ui: "Inter".into(), mono: "No Such Mono".into() };
        let installed = ["Inter".to_owned(), "JetBrains Mono".to_owned()];

        // when
        let (fonts, missing) = wanted.installed_or(&Fonts::default(), &installed);

        // then
        assert_eq!(fonts, Fonts { ui: "Inter".into(), mono: "JetBrains Mono".into() });
        assert_eq!(missing, vec![SharedString::from("No Such Mono")]);
    }
}
