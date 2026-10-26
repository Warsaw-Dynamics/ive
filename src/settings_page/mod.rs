//! The Settings tab: Appearance, Scripts, Updates, and Third-Party Software.

mod appearance;
mod scripts;
mod third_party;
mod updates;

use gpui::{Context, Div, SharedString, Stateful, div, prelude::*, rgb};

pub use third_party::TITLE as THIRD_PARTY_TITLE;
pub use updates::TITLE as UPDATES_TITLE;

use crate::click_targets::target;
use crate::selectable::selectable;
use crate::settings::SettingsStore;
use crate::theme::Theme;
use crate::workspace::Workspace;

/// Which page of the Settings tab is shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsPage {
    Appearance,
    Scripts,
    Updates,
    ThirdParty,
}

/// The Settings tab: its page, and why the last change was not saved.
pub struct SettingsView {
    page: SettingsPage,
    error: Option<SharedString>,
    third_party: third_party::ThirdPartyView,
}

impl SettingsView {
    pub fn new(cx: &mut Context<Workspace>) -> Self {
        Self {
            page: SettingsPage::Appearance,
            error: cx.global::<SettingsStore>().load_error().map(|reason| reason.to_owned().into()),
            third_party: third_party::ThirdPartyView::new(),
        }
    }

    /// The shown page's name, as in its list.
    pub fn page_name(&self) -> &'static str {
        match self.page {
            SettingsPage::Appearance => "Appearance",
            SettingsPage::Scripts => "Scripts",
            SettingsPage::Updates => updates::TITLE,
            SettingsPage::ThirdParty => third_party::TITLE,
        }
    }

    pub fn show_page(&mut self, page: SettingsPage) {
        self.page = page;
        // its entry in the page list leads back from a licence to the list
        self.third_party.show_licence(None);
    }

    /// Shows the licence of the `row`th third-party software, or their list for `None`.
    pub fn show_licence(&mut self, row: Option<usize>) {
        self.third_party.show_licence(row);
    }

    pub fn set_error(&mut self, error: Option<SharedString>) {
        self.error = error;
    }
}

/// What the Settings tab draws from: its own view, and what the workspace says about updates.
pub struct Page<'a> {
    pub view: &'a SettingsView,
    pub update_status: String,
}

pub fn render(page: Page, theme: Theme, cx: &mut Context<Workspace>) -> Div {
    let view = page.view;
    let saved_to = cx.global::<SettingsStore>().path().map(|path| format!("Saved to {}", path.display()));
    let content = match view.page {
        SettingsPage::Appearance => appearance::render(theme, cx).into_any_element(),
        // longer than a small window: it scrolls on its own, above the line saying where settings are saved
        SettingsPage::Scripts => div().id("scripts-page").flex_1().min_h_0().overflow_y_scroll().child(scripts::render(theme, cx)).into_any_element(),
        SettingsPage::Updates => updates::render(page.update_status, theme, cx).into_any_element(),
        SettingsPage::ThirdParty => third_party::render(&view.third_party, theme, cx).into_any_element(),
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
        .children(view.error.as_ref().map(|error| div().text_color(rgb(theme.error)).child(selectable("settings-error", error.clone(), cx))))
        .children(saved_to.map(|saved_to| div().text_size(theme.sizes.settings.text_size.px()).text_color(rgb(theme.muted)).child(selectable("settings-saved-to", saved_to, cx))));
    div().flex().flex_1().min_h_0().child(render_nav(view.page, theme, cx)).child(body)
}

fn render_nav(current: SettingsPage, theme: Theme, cx: &mut Context<Workspace>) -> Div {
    div()
        .flex()
        .flex_col()
        .flex_none()
        .w(theme.sizes.settings.nav_width.px())
        .py(theme.sizes.settings.nav_padding_y.px())
        .bg(rgb(theme.sidebar))
        .border_r(theme.sizes.settings.nav_border.px())
        .border_color(rgb(theme.border))
        .child(nav_item("Appearance", SettingsPage::Appearance, current, theme, cx))
        .child(nav_item("Scripts", SettingsPage::Scripts, current, theme, cx))
        .child(nav_item(updates::TITLE, SettingsPage::Updates, current, theme, cx))
        .child(nav_item(third_party::TITLE, SettingsPage::ThirdParty, current, theme, cx))
}

fn nav_item(
    label: &'static str,
    page: SettingsPage,
    current: SettingsPage,
    theme: Theme,
    cx: &mut Context<Workspace>,
) -> Stateful<Div> {
    let item = div()
        .id(label)
        .py(theme.sizes.settings.nav_item_padding_y.px())
        .pl(theme.sizes.settings.nav_base_indent.px())
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
