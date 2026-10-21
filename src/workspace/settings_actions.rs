use gpui::{Context, Window, prelude::*};

use super::Workspace;
use crate::colors::ColorKey;
use crate::editor::Tab;
use crate::settings::{SettingsError, SettingsStore, ThemePreference};
use crate::settings_page::{PreviewJob, SettingsPage, SettingsView};
use crate::theme;

impl Workspace {
    pub fn open_settings(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match self.tabs.find(Tab::is_settings) {
            Some(open) => self.activate_tab(open, window, cx),
            None => {
                let view = SettingsView::new(cx);
                self.open_tab(Tab::Settings(view), window, cx);
            }
        }
    }

    pub fn set_theme(&mut self, preference: ThemePreference, window: &mut Window, cx: &mut Context<Self>) {
        let saved = cx.update_global::<SettingsStore, _>(|store, _| {
            let settings = store.current().with_theme(preference);
            store.replace(settings)
        });
        theme::apply(window.appearance(), cx);
        self.after_settings_change(saved, cx);
    }

    pub fn show_settings_page(&mut self, page: SettingsPage, cx: &mut Context<Self>) {
        let job = self.settings_view().and_then(|view| view.show_page(page));
        self.highlight_preview(job, cx);
        cx.notify();
    }

    /// On the Third-Party Software page: the `row`th software's licence, or the list again for `None`.
    pub fn show_licence(&mut self, row: Option<usize>, cx: &mut Context<Self>) {
        if let Some(view) = self.settings_view() {
            view.show_licence(row);
        }
        cx.notify();
    }

    pub fn select_color_key(&mut self, key: ColorKey, cx: &mut Context<Self>) {
        let job = self.settings_view().and_then(|view| view.select(key));
        self.highlight_preview(job, cx);
        cx.notify();
    }

    /// Off the UI thread: a slow query compile must not freeze the window.
    fn highlight_preview(&mut self, job: Option<PreviewJob>, cx: &mut Context<Self>) {
        let Some(job) = job else {
            return;
        };
        let highlighting = cx.background_executor().spawn(async move { job.run() });
        cx.spawn(async move |workspace, cx| {
            let (page, preview) = highlighting.await;
            workspace
                .update(cx, |workspace, cx| {
                    if let Some(view) = workspace.settings_view() {
                        view.take_preview(page, preview);
                    }
                    cx.notify();
                })
                .ok();
        })
        .detach();
    }

    fn after_settings_change(&mut self, saved: Result<(), SettingsError>, cx: &mut Context<Self>) {
        if let Some(view) = self.settings_view() {
            view.set_error(saved.err().map(|error| error.to_string().into()));
        }
        cx.notify();
    }

    fn settings_view(&mut self) -> Option<&mut SettingsView> {
        let id = self.tabs.find(Tab::is_settings)?;
        match self.tabs.get_mut(id) {
            Some(Tab::Settings(view)) => Some(view),
            _ => None,
        }
    }
}
