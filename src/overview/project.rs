use std::rc::Rc;

use gpui::{AnyElement, ClickEvent, Context, SharedString, div, prelude::*, rgb};

use super::{Listener, Location, ProjectOverview, ProjectSection, distance_tags, section_row};
use crate::editor::Tab;
use crate::paths::tilde_home;
use crate::project::{Checkout, Project, ProjectKind};
use crate::tabs::TabId;
use crate::theme::Theme;
use crate::workspace::Workspace;

/// The project overview tab: section row, then the selected section.
pub fn render_project_overview(id: TabId, overview: &ProjectOverview, workspace: &Workspace, cx: &mut Context<Workspace>) -> AnyElement {
    let theme = crate::theme::tabs(cx);
    let Some(project) = workspace.project(&overview.project) else {
        return div().p(theme.sizes.overview.page_message_padding.px()).text_color(rgb(theme.muted)).child("This project is closed").into_any_element();
    };
    let sections: Vec<(SharedString, bool, Listener)> = ProjectSection::ALL
        .iter()
        .filter(|section| **section != ProjectSection::Git || overview.git.is_some())
        .map(|section| {
            let section = *section;
            let select: Listener = Rc::new(cx.listener(move |workspace: &mut Workspace, _event: &ClickEvent, window, cx| {
                workspace.update_tab(id, |tab, cx| {
                    if let Tab::ProjectOverview(overview) = tab {
                        overview.select(section, cx);
                    }
                }, cx);
                workspace.section_shown(id, window, cx);
            }));
            (section.label().into(), section == overview.section, select)
        })
        .collect();
    let body = div().mt(theme.sizes.overview.page_body_gap.px());
    let body = match (overview.section, &overview.git) {
        (ProjectSection::Git, Some(git)) => body.child(git.section.clone()),
        // made by `section_shown` before the first frame shows it
        (ProjectSection::Scratches, _) => body.children(overview.scratches.clone()),
        // the list fills the page and scrolls itself: it draws only the rows in view
        (ProjectSection::Worktrees | ProjectSection::Git, _) => {
            body.flex().flex_col().flex_1().min_h_0().child(workspace.render_worktree_list(&overview.project, overview.git.as_ref().map(|git| &git.list), theme, cx))
        }
    };
    div()
        .id("project-overview")
        .flex()
        .flex_col()
        .flex_1()
        .min_h_0()
        .overflow_y_scroll()
        .px(theme.sizes.overview.page_padding_x.px())
        .pt(theme.sizes.overview.page_padding_top.px())
        .pb(theme.sizes.overview.page_padding_bottom.px())
        .child(section_row(&sections, location(project, theme), theme, cx))
        .child(body)
        .into_any_element()
}

fn location(project: &Project, theme: Theme) -> Location {
    let path = tilde_home(&project.root);
    let text = match project.kind {
        ProjectKind::Git => path,
        ProjectKind::Plain => format!("{path} · no git"),
    };
    Location { text: text.into(), full: project.root.display().to_string().into(), tags: Vec::new(), colour: theme.accent }
}

/// A worktree's tags: `N changes` or `clean` (`no git` for a plain folder, nothing while its status is unread),
/// `↑ahead ↓behind` against the base, `● N running`.
pub fn card_tags(checkout: &Checkout, kind: ProjectKind, running: usize) -> Vec<String> {
    let mut tags = match (&checkout.status, kind) {
        (None, ProjectKind::Plain) => vec!["no git".to_owned()],
        (None, ProjectKind::Git) => Vec::new(),
        (Some(status), _) if status.changes == 0 => vec!["clean".to_owned()],
        (Some(status), _) if status.changes == 1 => vec!["1 change".to_owned()],
        (Some(status), _) => vec![format!("{} changes", status.changes)],
    };
    if let Some(status) = checkout.status.as_ref().filter(|status| status.base.is_some()) {
        let distance = distance_tags(status.ahead, status.behind);
        if !distance.is_empty() {
            tags.push(distance.join(" "));
        }
    }
    if running > 0 {
        tags.push(format!("● {running} running"));
    }
    tags
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::project::CheckoutStatus;

    fn checkout(status: Option<CheckoutStatus>) -> Checkout {
        Checkout { path: PathBuf::from("/r/wt"), label: "wt".to_owned(), branch: None, head: None, is_main: false, status }
    }

    #[test]
    fn should_tag_changes_distance_and_running_processes() {
        // given
        let status = CheckoutStatus { changes: 3, ahead: 5, behind: 0, base: Some("dev".to_owned()) };

        // when
        let tags = card_tags(&checkout(Some(status)), ProjectKind::Git, 2);

        // then
        assert_eq!(tags, ["3 changes", "↑5", "● 2 running"]);
    }

    #[test]
    fn should_hide_distance_without_base_and_say_clean_or_no_git() {
        // given
        let no_base = CheckoutStatus { changes: 0, ahead: 4, behind: 4, base: None };

        // when
        let tags = [
            card_tags(&checkout(Some(no_base)), ProjectKind::Git, 0),
            card_tags(&checkout(None), ProjectKind::Plain, 0),
            card_tags(&checkout(None), ProjectKind::Git, 0),
        ];

        // then
        assert_eq!(tags, [vec!["clean".to_owned()], vec!["no git".to_owned()], Vec::new()]);
    }
}
