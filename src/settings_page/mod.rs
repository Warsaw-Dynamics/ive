//! The Settings tab: Appearance, Scripts, one Color Scheme page per language, and Third-Party Software.

mod appearance;
mod color_page;
mod key_state;
mod preview;
mod scripts;
mod third_party;

use gpui::{Context, Div, SharedString, Stateful, div, prelude::*, px, rgb};

pub use key_state::KeyState;
pub use preview::Preview;
pub use third_party::TITLE as THIRD_PARTY_TITLE;

use crate::click_targets::target;
use crate::colors::{ColorKey, Page};
use crate::settings::SettingsStore;
use crate::theme::Theme;
use crate::workspace::Workspace;

/// Which page of the Settings tab is shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsPage {
    Appearance,
    Scripts,
    Colors(Page),
    ThirdParty,
}

/// The Settings tab: its page, the selected colour key, and why the last change was not saved.
pub struct SettingsView {
    page: SettingsPage,
    selected: ColorKey,
    error: Option<SharedString>,
    /// demo text of `preview_page`; a language's is plain until its highlighting comes from the background
    preview: Preview,
    preview_page: Page,
    third_party: third_party::ThirdPartyView,
}

impl SettingsView {
    pub fn new(cx: &mut Context<Workspace>) -> Self {
        Self {
            page: SettingsPage::Appearance,
            selected: first_key(Page::LANGUAGE_DEFAULTS),
            error: cx.global::<SettingsStore>().load_error().map(|reason| reason.to_owned().into()),
            preview: Preview::of(Page::LANGUAGE_DEFAULTS),
            preview_page: Page::LANGUAGE_DEFAULTS,
            third_party: third_party::ThirdPartyView::new(),
        }
    }

    /// Shows `page`; a colour page starts with its first key selected. Returns the preview to highlight, when the
    /// page changed: run it on the background executor and hand the result to [`Self::take_preview`].
    /// The shown page's name, as in its list: Appearance, Scripts, or a colour page's title.
    pub fn page_name(&self) -> &'static str {
        match self.page {
            SettingsPage::Appearance => "Appearance",
            SettingsPage::Scripts => "Scripts",
            SettingsPage::Colors(page) => page.title(),
            SettingsPage::ThirdParty => third_party::TITLE,
        }
    }

    pub fn show_page(&mut self, page: SettingsPage) -> Option<PreviewJob> {
        self.page = page;
        // its entry in the page list leads back from a licence to the list
        self.third_party.show_licence(None);
        let SettingsPage::Colors(colors) = page else {
            return None;
        };
        self.selected = first_key(colors);
        self.show_preview_of(colors)
    }

    /// Selects `key`, showing its page; returns the preview to highlight as [`Self::show_page`] does.
    pub fn select(&mut self, key: ColorKey) -> Option<PreviewJob> {
        let job = self.show_preview_of(key.page());
        self.page = SettingsPage::Colors(key.page());
        self.selected = key;
        job
    }

    fn show_preview_of(&mut self, page: Page) -> Option<PreviewJob> {
        if self.preview_page == page {
            return None;
        }
        self.preview = Preview::plain(page);
        self.preview_page = page;
        Some(PreviewJob(page))
    }

    /// A highlighted preview, unless another page is shown by now.
    pub fn take_preview(&mut self, page: Page, preview: Preview) {
        if self.preview_page == page {
            self.preview = preview;
        }
    }

    /// Shows the licence of the `row`th third-party software, or their list for `None`.
    pub fn show_licence(&mut self, row: Option<usize>) {
        self.third_party.show_licence(row);
    }

    pub fn preview(&self) -> &Preview {
        &self.preview
    }

    pub fn selected(&self) -> ColorKey {
        self.selected
    }

    pub fn set_error(&mut self, error: Option<SharedString>) {
        self.error = error;
    }

}

/// Highlighting a page's preview, which may compile the language's queries first.
pub struct PreviewJob(Page);

impl PreviewJob {
    pub fn run(self) -> (Page, Preview) {
        (self.0, Preview::of(self.0))
    }
}

fn first_key(page: Page) -> ColorKey {
    ColorKey::on_page(page).first().copied().unwrap_or(ColorKey::TEXT)
}

pub fn render(view: &SettingsView, theme: Theme, cx: &mut Context<Workspace>) -> Div {
    let saved_to = cx.global::<SettingsStore>().path().map(|path| format!("Saved to {}", path.display()));
    let content = match view.page {
        SettingsPage::Appearance => appearance::render(theme, cx),
        SettingsPage::Scripts => scripts::render(theme, cx),
        SettingsPage::Colors(page) => color_page::render(view, page, theme, cx),
        SettingsPage::ThirdParty => third_party::render(&view.third_party, theme, cx),
    };
    let body = div()
        .flex()
        .flex_col()
        .flex_1()
        .min_w_0()
        .min_h_0()
        .gap(theme.sizes.settings.page_gap.px())
        .p(theme.sizes.settings.page_padding.px())
        .child(content)
        .children(view.error.as_ref().map(|error| div().text_color(rgb(theme.error)).child(error.clone())))
        .children(saved_to.map(|saved_to| div().text_size(theme.sizes.settings.text_size.px()).text_color(rgb(theme.muted)).child(saved_to)));
    div().flex().flex_1().min_h_0().child(render_nav(view.page, theme, cx)).child(body)
}

/// Scrolls: the Color Scheme pages are longer than a window is tall.
fn render_nav(current: SettingsPage, theme: Theme, cx: &mut Context<Workspace>) -> Stateful<Div> {
    let languages: Vec<_> =
        Page::listed().iter().map(|&page| nav_item(page.title(), 2, SettingsPage::Colors(page), current, theme, cx)).collect();
    div()
        .id("settings-nav")
        .flex()
        .flex_col()
        .flex_none()
        .w(theme.sizes.settings.nav_width.px())
        .min_h_0()
        .overflow_y_scroll()
        .py(theme.sizes.settings.nav_padding_y.px())
        .bg(rgb(theme.sidebar))
        .border_r(theme.sizes.settings.nav_border.px())
        .border_color(rgb(theme.border))
        .child(nav_item("Appearance", 0, SettingsPage::Appearance, current, theme, cx))
        .child(nav_item("Scripts", 0, SettingsPage::Scripts, current, theme, cx))
        .child(nav_item(third_party::TITLE, 0, SettingsPage::ThirdParty, current, theme, cx))
        .child(nav_heading("Editor", 0, theme))
        .child(nav_heading("Color Scheme", 1, theme))
        .children(languages)
}

fn nav_heading(label: &'static str, depth: usize, theme: Theme) -> Div {
    div().py(theme.sizes.settings.nav_item_padding_y.px()).pl(nav_indent(depth, theme)).text_size(theme.sizes.settings.text_size.px()).font_weight(theme.sizes.settings.nav_heading_weight.font_weight()).text_color(rgb(theme.muted)).child(label)
}

fn nav_item(
    label: &'static str,
    depth: usize,
    page: SettingsPage,
    current: SettingsPage,
    theme: Theme,
    cx: &mut Context<Workspace>,
) -> Stateful<Div> {
    let item = div()
        .id(label)
        .py(theme.sizes.settings.nav_item_padding_y.px())
        .pl(nav_indent(depth, theme))
        .text_size(theme.sizes.settings.text_size.px())
        .cursor_pointer()
        .child(label)
        .on_click(target(
            format!("settings-page:{label}"),
            cx.listener(move |workspace, _event, _window, cx| workspace.show_settings_page(page, cx)),
            cx,
        ));
    if page == current {
        return item.bg(rgb(theme.selected));
    }
    item.hover(|style| style.bg(rgb(theme.hover)))
}

fn nav_indent(depth: usize, theme: Theme) -> gpui::Pixels {
    px(theme.sizes.settings.nav_base_indent.0 + depth as f32 * theme.sizes.settings.nav_indent.0)
}
