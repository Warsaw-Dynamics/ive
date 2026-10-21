//! The tab asking for the values a run needs and has none of: one field per variable, Run and Cancel.

use std::path::{Path, PathBuf};

use gpui::{AnyElement, Context, Entity, Focusable, FontWeight, SharedString, div, prelude::*, px, rgb};

use super::file::VarDecl;
use crate::click_targets::target;
use crate::project::CheckoutRef;
use crate::tabs::TabId;
use crate::text_input::{TextInput, TextInputEvent};
use crate::workspace::Workspace;

/// Whether starting a run also brings its tab to the front.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShowRun {
    Tab,
    /// the run appears in Processes and its row changes in place
    Stay,
}

/// A configuration to run, and where its output shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunRequest {
    pub checkout: CheckoutRef,
    /// the folder of its `.run.toml`
    pub folder: PathBuf,
    pub key: String,
    pub show: ShowRun,
}

impl RunRequest {
    /// The request once its values were asked for: the values tab took the user's place, so the run's own tab shows.
    pub fn after_values(&self) -> RunRequest {
        RunRequest { show: ShowRun::Tab, ..self.clone() }
    }
}

pub struct RunValuesTab {
    pub request: RunRequest,
    /// the configuration's name, as the title says it
    pub name: String,
    pub fields: Vec<ValueField>,
    /// why saving failed (the wallet, vars.toml), shown above the buttons
    pub error: Option<String>,
}

pub struct ValueField {
    pub var: VarDecl,
    pub input: Entity<TextInput>,
}

impl RunValuesTab {
    /// A field per variable in `missing`, the first one focused; Enter in any of them runs.
    pub fn new(request: RunRequest, name: String, missing: Vec<VarDecl>, window: &mut gpui::Window, cx: &mut Context<Workspace>) -> Self {
        let fields: Vec<ValueField> = missing
            .into_iter()
            .map(|var| {
                let secret = var.secret;
                let target_name = format!("value:{}", var.name);
                let input = cx.new(|cx| {
                    let mut input = TextInput::new(cx);
                    input.set_masked(secret);
                    input.set_target(target_name);
                    input
                });
                let submitted = request.clone();
                cx.subscribe_in(&input, window, move |workspace, _input, event: &TextInputEvent, window, cx| {
                    if *event == TextInputEvent::Submitted {
                        workspace.submit_run_values(&submitted, window, cx);
                    }
                })
                .detach();
                ValueField { var, input }
            })
            .collect();
        if let Some(first) = fields.first() {
            let handle = first.input.focus_handle(cx);
            window.focus(&handle, cx);
        }
        Self { request, name, fields, error: None }
    }

    /// Same checkout, folder and key: how the run is shown does not tell values tabs apart.
    pub fn shows(&self, request: &RunRequest) -> bool {
        self.request.checkout == request.checkout && self.request.folder == request.folder && self.request.key == request.key
    }

    pub fn title(&self) -> String {
        format!("Run {}", self.name)
    }
}

/// `<name> needs N value(s)`, where they come from, the fields with where each is kept, Run and Cancel.
pub fn render_values_tab(id: TabId, tab: &RunValuesTab, cx: &mut Context<Workspace>) -> AnyElement {
    let theme = crate::theme::tabs(cx);
    let count = tab.fields.len();
    let heading = format!("{} needs {count} value{}", tab.name, if count == 1 { "" } else { "s" });
    let source = format!("from {}/.run.toml; asked once, then remembered", relative(&tab.request.folder, &tab.request.checkout.path));
    let fields = tab.fields.iter().map(|field| {
        let kept = if field.var.secret { "stored in the OS wallet; vars.toml keeps a reference" } else { "saved to vars.toml" };
        let label = if field.var.description.is_empty() { field.var.name.clone() } else { field.var.description.clone() };
        div()
            .flex()
            .flex_col()
            .gap_1()
            .child(div().flex().gap_2().child(label).child(div().font_family("JetBrains Mono").text_color(rgb(theme.muted)).child(field.var.name.clone())))
            .child(field.input.clone())
            .child(div().text_xs().text_color(rgb(theme.muted)).child(kept))
    });
    let request = tab.request.clone();
    let run = target(format!("run-values:{id}", id = id.value()), cx.listener(move |workspace, _event, window, cx| {
        workspace.submit_run_values(&request, window, cx);
    }), cx);
    let cancel = target(format!("cancel-values:{}", id.value()), cx.listener(move |workspace, _event, window, cx| workspace.close_tab(id, window, cx)), cx);
    div()
        .id("run-values")
        .flex()
        .flex_col()
        .gap_3()
        .max_w(px(520.))
        .px(px(24.))
        .pt(px(24.))
        .text_sm()
        .child(div().font_weight(FontWeight::SEMIBOLD).child(heading))
        .child(div().text_xs().text_color(rgb(theme.muted)).child(source))
        .children(fields)
        .children(tab.error.clone().map(|error| div().text_color(rgb(theme.error)).child(SharedString::from(error))))
        .child(
            div()
                .flex()
                .gap_2()
                .child(button("run-values-run", "Run", theme.accent, theme.on_accent).on_click(run))
                .child(button("run-values-cancel", "Cancel", theme.hover, theme.text).on_click(cancel)),
        )
        .into_any_element()
}

fn button(id: &'static str, label: &'static str, background: u32, color: u32) -> gpui::Stateful<gpui::Div> {
    div().id(id).px_3().py_1().rounded_md().cursor_pointer().bg(rgb(background)).text_color(rgb(color)).child(label)
}

/// The variables whose field is blank: saving them would remember an empty value for good.
pub fn empty_fields(values: &[(VarDecl, String)]) -> Vec<String> {
    values.iter().filter(|(_, value)| value.trim().is_empty()).map(|(var, _)| var.name.clone()).collect()
}

/// `folder` relative to `checkout`, for display.
pub fn relative(folder: &Path, checkout: &Path) -> String {
    folder.strip_prefix(checkout).unwrap_or(folder).display().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::ProjectId;

    fn var(name: &str) -> VarDecl {
        VarDecl { name: name.to_owned(), description: String::new(), secret: false, default: None }
    }

    #[test]
    fn should_name_the_fields_left_empty() {
        // given
        let values = [(var("a"), "x".to_owned()), (var("b"), "  ".to_owned()), (var("c"), String::new())];

        // when
        let empty = empty_fields(&values);

        // then
        assert_eq!(empty, ["b", "c"]);
    }

    #[test]
    fn should_match_a_request_whatever_it_shows() {
        // given
        let request = |show| RunRequest { checkout: CheckoutRef { project: ProjectId::parse("p").unwrap(), path: "/p".into() }, folder: "/p/app".into(), key: "serve".to_owned(), show };
        let tab = RunValuesTab { request: request(ShowRun::Tab), name: "Serve".to_owned(), fields: Vec::new(), error: None };

        // when
        let found = tab.shows(&request(ShowRun::Stay));

        // then
        assert!(found);
    }

    #[test]
    fn should_show_a_run_once_its_values_were_asked() {
        // given
        let request = RunRequest { checkout: CheckoutRef { project: ProjectId::parse("p").unwrap(), path: "/p".into() }, folder: "/p/app".into(), key: "serve".to_owned(), show: ShowRun::Stay };

        // when
        let after = request.after_values();

        // then
        assert_eq!(after, RunRequest { show: ShowRun::Tab, ..request });
    }
}
