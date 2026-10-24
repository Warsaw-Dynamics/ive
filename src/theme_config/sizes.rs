//! The theme's sizes, shared by the dark and light appearance: one table per screen area.

use serde::Deserialize;

use super::{Factor, FontPx, Fraction, Height, Px, RootPx, Weight};

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sizes {
    /// what `"1rem"` is; 16 when `init.rn` removed it
    #[serde(default)]
    pub rem: RootPx,
    /// every icon: file types, tools, chevrons, actions
    pub icon: Px,
    pub tree: TreeSizes,
    pub tabs: TabsSizes,
    pub window: WindowSizes,
    pub editor: EditorSizes,
    pub diff: DiffSizes,
    pub terminal: TerminalSizes,
    pub overview: OverviewSizes,
    pub git: GitSizes,
    pub settings: SettingsSizes,
    pub controls: ControlsSizes,
    pub menu: MenuSizes,
}

/// The menus of tabs and tree rows (IntelliJ's popup menus): their panel, lines, keys and separators.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MenuSizes {
    /// a panel grows past this to fit its longest line
    pub min_width: Px,
    pub padding_y: Px,
    pub radius: Px,
    pub border: Px,
    pub text_size: FontPx,
    pub weight: Weight,
    pub item_height: Height,
    /// the highlight's distance from the panel's sides
    pub item_margin_x: Px,
    pub item_padding_x: Px,
    pub item_radius: Px,
    /// at least this between a line's label and its key
    pub shortcut_gap: Px,
    pub separator_margin_y: Px,
    /// a separator's distance from the panel's sides
    pub separator_inset_x: Px,
    /// a line waiting for its `when` predicate
    pub pending_opacity: Fraction,
}

/// Controls used everywhere: prompts, tooltips, text fields, paging, progress bars and scrollbars.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ControlsSizes {
    /// prompts, such as the quit prompt
    pub prompt_width: Px,
    pub prompt_gap: Px,
    pub prompt_padding: Px,
    pub prompt_radius: Px,
    pub prompt_border: Px,
    pub prompt_title_weight: Weight,
    pub prompt_detail_text_size: FontPx,
    pub prompt_buttons_gap: Px,
    pub prompt_buttons_margin_top: Px,
    pub prompt_button_padding_x: Px,
    pub prompt_button_padding_y: Px,
    pub prompt_button_radius: Px,
    pub prompt_button_text_size: FontPx,
    pub prompt_button_border: Px,
    /// tooltips
    pub tooltip_padding_x: Px,
    pub tooltip_padding_y: Px,
    pub tooltip_radius: Px,
    pub tooltip_border: Px,
    pub tooltip_text_size: FontPx,
    /// Show 10 more · Show all
    pub paging_gap: Px,
    pub paging_text_size: FontPx,
    /// text fields
    pub input_height: Height,
    pub input_caret_width: Px,
    pub input_padding_x: Px,
    pub input_radius: Px,
    pub input_border: Px,
    pub input_text_size: FontPx,
    pub input_disabled_opacity: Fraction,
    pub input_marked_underline: Px,
    /// a running process's progress
    pub progress_width: Px,
    pub progress_height: Height,
    pub progress_segment: Px,
    /// scrollbars
    pub scrollbar_thickness: Px,
    pub scrollbar_min_thumb: Px,
    pub scrollbar_inset: Px,
}

/// The Settings tab: its page list and its pages.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SettingsSizes {
    /// the page and its list of pages
    pub page_gap: Px,
    pub page_padding: Px,
    pub text_size: FontPx,
    pub title_text_size: FontPx,
    pub title_weight: Weight,
    pub nav_width: Px,
    pub nav_padding_y: Px,
    pub nav_border: Px,
    pub nav_base_indent: Px,
    pub nav_item_padding_y: Px,
    /// Appearance
    pub appearance_gap: Px,
    pub appearance_label_width: Px,
    pub segmented_radius: Px,
    pub segmented_border: Px,
    pub segment_padding_x: Px,
    pub segment_padding_y: Px,
    /// Third-Party Software
    pub table_gap: Px,
    pub table_padding_right: Px,
    pub list_gap: Px,
    pub row_padding_y: Px,
    pub licence_width: Px,
    pub table_header_weight: Weight,
    pub licence_text_padding_bottom: Px,
    pub licence_text_margin_bottom: Px,
    pub licence_text_border: Px,
}

/// Git views: a project's log and stashes, commit and diff tabs.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GitSizes {
    /// list rows: changed files, commits, stashes
    pub row_gap: Px,
    pub row_height: Height,
    pub row_padding_x: Px,
    pub row_border: Px,
    pub text_size: FontPx,
    pub detail_text_size: FontPx,
    pub body_weight: Weight,
    pub author_max_width: Px,
    /// a file's change mark: M, A, D …
    pub mark_width: Px,
    pub mark_text_size: FontPx,
    pub mark_weight: Weight,
    /// small tags
    pub tag_padding_x: Px,
    pub tag_radius: Px,
    pub tag_text_size: FontPx,
    /// Uncommitted / Since / Commits
    pub segment_gap: Px,
    pub segment_padding_x: Px,
    pub segment_padding_y: Px,
    pub segment_radius: Px,
    pub segment_border: Px,
    pub segment_text_size: FontPx,
    /// Name / Modified, Flat / Tree
    pub toggle_gap: Px,
    pub toggle_padding_x: Px,
    pub toggle_padding_y: Px,
    pub toggle_radius: Px,
    pub toggle_border: Px,
    pub toggle_text_size: FontPx,
    /// a group's title, such as Log
    pub group_title_gap: Px,
    pub group_title_padding_top: Px,
    pub group_title_padding_bottom: Px,
    pub group_title_text_size: FontPx,
    pub group_title_weight: Weight,
    pub group_hint_text_size: FontPx,
    pub group_hint_weight: Weight,
    /// an error with Retry
    pub error_gap: Px,
    pub error_padding_y: Px,
    /// bordered boxes: lists, panes, the summary
    pub box_radius: Px,
    pub box_border: Px,
    pub message_padding: Px,
    /// a project's Git section
    pub section_gap: Px,
    pub summary_gap: Px,
    pub summary_padding_x: Px,
    pub summary_padding_y: Px,
    /// the Git log tab
    pub log_toolbar_gap: Px,
    pub log_toolbar_padding_x: Px,
    pub log_toolbar_padding_top: Px,
    pub log_buttons_gap: Px,
    pub log_padding_bottom: Px,
    pub log_margin_bottom: Px,
    pub log_more_padding_x: Px,
    /// a commit tab's header
    pub commit_message_padding: Px,
    pub commit_header_gap: Px,
    pub commit_header_padding_x: Px,
    pub commit_header_padding_y: Px,
    pub commit_header_border: Px,
    pub commit_subject_padding_top: Px,
    pub commit_subject_weight: Weight,
    /// a diff's header: file name and source
    pub diff_header_gap: Px,
    pub diff_header_padding_x: Px,
    pub diff_header_padding_y: Px,
    pub diff_header_border: Px,
    pub diff_header_weight: Weight,
}

/// The project overview tab: its pages, section row, cards, scratches and git views.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OverviewSizes {
    /// the page around a section
    pub page_padding_x: Px,
    pub page_padding_top: Px,
    pub page_padding_bottom: Px,
    pub page_body_gap: Px,
    pub page_message_padding: Px,
    /// the section buttons and the location at the right
    pub section_height: Height,
    pub section_padding_x: Px,
    pub section_text_size: FontPx,
    pub section_radius: Px,
    pub section_underline: Px,
    pub section_selected_weight: Weight,
    pub section_gap: Px,
    pub section_row_border: Px,
    pub location_padding_left: Px,
    pub location_text_size: FontPx,
    /// small tags: ahead/behind, a card's state
    pub tag_margin_left: Px,
    pub tag_padding_x: Px,
    pub tag_radius: Px,
    pub tag_text_size: FontPx,
    /// a project's worktree cards
    pub cards_gap: Px,
    pub card_gap: Px,
    pub card_padding_x: Px,
    pub card_padding_y: Px,
    pub card_title_gap: Px,
    pub card_title_weight: Weight,
    pub card_detail_text_size: FontPx,
    pub card_tags_gap: Px,
    pub card_min_width: Px,
    pub card_radius: Px,
    pub card_border: Px,
    pub card_stripe_width: Px,
    /// a project's scratches
    pub scratches_gap: Px,
    pub scratches_toolbar_gap: Px,
    pub scratches_hint_text_size: FontPx,
    pub scratches_search_width: Px,
    pub scratches_text_size: FontPx,
    pub scratches_row_height: Height,
    pub scratches_row_padding_x: Px,
    pub scratches_row_radius: Px,
    pub scratches_group_gap: Px,
    pub scratches_group_weight: Weight,
    pub scratches_count_weight: Weight,
    pub scratches_input_row_height: Height,
    pub scratches_name_input_width: Px,
    pub scratches_error_text_size: FontPx,
    pub link_padding_x: Px,
    pub link_radius: Px,
    /// the New worktree card and its branch menu
    pub new_card_width: Px,
    pub new_card_gap: Px,
    pub new_card_padding: Px,
    pub new_card_radius: Px,
    pub new_card_border: Px,
    pub new_card_title_gap: Px,
    pub new_card_title_weight: Weight,
    pub new_card_status_text_size: FontPx,
    pub branch_menu_max_height: Height,
    pub branch_menu_padding: Px,
    pub branch_menu_radius: Px,
    pub branch_menu_border: Px,
    pub branch_menu_message_padding: Px,
    pub branch_item_height: Height,
    pub branch_item_gap: Px,
    pub branch_item_padding_x: Px,
    pub branch_item_radius: Px,
    pub branch_item_text_size: FontPx,
}

/// Terminal and Claude Code tabs.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TerminalSizes {
    pub text_size: FontPx,
    /// lines are this many font sizes apart
    pub line_height_factor: Factor,
    /// around the grid of cells
    pub padding: Px,
    pub cursor_thickness: Px,
    /// underlines and strike-throughs
    pub line_thickness: Px,
    pub normal_weight: Weight,
    pub bold_weight: Weight,
}

/// Diffs in the git views, diff tabs and commit tabs.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiffSizes {
    pub row_height: Height,
    pub text_size: FontPx,
    /// before a code line's line numbers
    pub line_padding: Px,
    /// before a hunk header or a note
    pub hunk_padding_left: Px,
    /// a commit tab's file header
    pub header_gap: Px,
    pub header_padding_x: Px,
    pub header_weight: Weight,
    /// after each line number
    pub number_padding_right: Px,
}

/// A file or scratch tab: its text, and the scratch's header, bar and notes.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EditorSizes {
    pub text_size: FontPx,
    /// lines are this many font sizes apart
    pub line_height_factor: Factor,
    pub caret_width: Px,
    /// a file that cannot be shown or saved
    pub notice_padding_x: Px,
    pub notice_padding_y: Px,
    pub notice_text_size: FontPx,
    /// a scratch's header, its bar and its notes
    pub scratch_strip_padding_x: Px,
    pub scratch_strip_padding_y: Px,
    pub scratch_strip_text_size: FontPx,
    pub scratch_header_gap: Px,
    pub scratch_header_border: Px,
    pub scratch_bar_gap: Px,
    pub scratch_error_padding: Px,
    /// Run, and the bar's buttons
    pub scratch_button_gap: Px,
    pub scratch_button_padding_x: Px,
    pub scratch_button_radius: Px,
    pub scratch_button_border: Px,
}

/// The tab row: its tabs, the tools at its right end, a dragged tab and the preview of a tab being detached.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TabsSizes {
    pub row_height: Height,
    pub row_border: Px,
    pub strip_padding_x: Px,
    /// between tabs
    pub gap: Px,
    pub tools_gap: Px,
    pub tools_padding_left: Px,
    pub tab_height: Height,
    /// a tab's padding, the right one beside its close button
    pub tab_padding_left: Px,
    pub tab_padding_right: Px,
    pub tab_radius: Px,
    pub tab_border: Px,
    pub tab_text_size: FontPx,
    /// between a tab's icon, title and close button
    pub tab_inner_gap: Px,
    pub tab_icon_size: Px,
    pub title_max_width: Px,
    pub close_radius: Px,
    /// the checkout colour bar under a tab
    pub underline_inset: Px,
    pub underline_height: Px,
    pub underline_radius: Px,
    /// where a dragged tab would land
    pub drop_marker_width: Px,
    pub drop_marker_inset: Px,
    pub drop_marker_radius: Px,
    pub drag_chip_opacity: Fraction,
    /// a project's or module's favicon on a tab's icon, lower right, overhanging it, cut out by a ring
    pub badge_size: Px,
    pub badge_offset: Px,
    pub badge_ring: Px,
    /// a worktree's colour bar, in the tree and the project group's menu
    pub rail_width: Px,
    pub rail_inset: Px,
    /// a worktree tab's background, of its colour: as it is, hovered, shown, and the shown one's outline
    pub tint: Fraction,
    pub tint_hover: Fraction,
    pub tint_shown: Fraction,
    pub tint_outline: Fraction,
    /// a project's group of tabs: the gap around it, its head's text, its count's pill
    pub group_gap: Px,
    pub group_head_text_size: FontPx,
    pub group_count_radius: Px,
    /// Restart and Settings
    pub toolbar_button_padding: Px,
    pub toolbar_button_radius: Px,
    pub toolbar_icon_size: Px,
    /// the window a dragged tab would detach into
    pub preview_header_height: Height,
    pub preview_padding_x: Px,
    pub preview_border: Px,
    pub preview_text_size: FontPx,
    pub preview_radius: Px,
    pub preview_opacity: Fraction,
}

/// The window's own frame and buttons, and the welcome screen.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WindowSizes {
    pub corner_radius: Px,
    pub border: Px,
    /// minimise, maximise and close
    pub control_width: Px,
    pub control_icon_size: Px,
    pub welcome_gap: Px,
    pub welcome_title_size: FontPx,
    pub welcome_recent_gap: Px,
    pub welcome_recent_padding_x: Px,
    pub welcome_recent_padding_y: Px,
    pub welcome_recent_radius: Px,
    /// Open Folder… on the welcome screen
    pub button_padding_x: Px,
    pub button_padding_y: Px,
    pub button_radius: Px,
}

/// The projects panel: its header, Open project menu, rows and their parts.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TreeSizes {
    pub header_height: Height,
    pub header_padding_left: Px,
    pub header_padding_right: Px,
    pub header_title_size: FontPx,
    pub header_title_weight: Weight,
    pub header_tools_gap: Px,
    pub header_button_padding: Px,
    pub header_button_radius: Px,
    /// the panel's right edge and the header's bottom edge
    pub border: Px,
    /// the Open project menu's distance below the header and from the panel's right edge
    pub menu_offset: Px,
    pub menu_width: Px,
    pub menu_padding: Px,
    pub menu_radius: Px,
    pub menu_text_size: FontPx,
    pub menu_weight: Weight,
    pub menu_section_padding_x: Px,
    pub menu_section_padding_top: Px,
    pub menu_section_padding_bottom: Px,
    pub menu_section_text_size: FontPx,
    pub menu_item_height: Height,
    pub menu_item_gap: Px,
    pub menu_item_padding_x: Px,
    pub menu_item_radius: Px,
    /// a project's initials
    pub badge_size: Px,
    pub badge_radius: Px,
    pub badge_text_size: FontPx,
    pub badge_weight: Weight,
    pub row_height: Height,
    pub text_size: FontPx,
    pub icon_gap: Px,
    /// before a top-level row; each level below adds one icon and gap
    pub padding_left: Px,
    pub padding_right: Px,
    pub project_name_weight: Weight,
    /// a project's branch, "no git" or "missing"
    pub tag_gap: Px,
    pub tag_padding_x: Px,
    pub tag_radius: Px,
    pub tag_text_size: FontPx,
    pub chevron_radius: Px,
    /// a virtual folder's `@name`: `@hidden`
    pub chip_padding_x: Px,
    pub chip_radius: Px,
    /// a count pill, such as a folder of changed files' count
    pub count_padding_x: Px,
    pub count_radius: Px,
    pub count_text_size: FontPx,
    /// a folder's count after its label
    pub folder_count_padding_x: Px,
    pub folder_count_text_size: FontPx,
    /// a row's button at its right end
    pub action_padding: Px,
    pub action_radius: Px,
    pub resize_handle_width: Px,
    /// the panel's width until the user resizes it
    pub default_width: Px,
    /// a highlighted row's background: its corner radius and its distance from the panel's sides; 0 and 0 fill the
    /// row from edge to edge, IntelliJ draws 8 and 12
    pub selection_radius: Px,
    pub selection_inset: Px,
}

impl TreeSizes {
    /// Whether highlighted rows are filled from edge to edge, without rounded corners.
    pub fn flat_selection(&self) -> bool {
        self.selection_radius.0 == 0. && self.selection_inset.0 == 0.
    }
}

#[cfg(test)]
mod tests {
    use crate::theme_config::{Factor, FontPx, Fraction, Height, Px, Weight, defaults};

    #[test]
    fn should_default_overview_sizes_to_todays_values() {
        // when
        let sizes = defaults().sizes.overview;

        // then
        assert_eq!(sizes.page_padding_x, Px(24.0), "page_padding_x");
        assert_eq!(sizes.page_padding_top, Px(6.0), "page_padding_top");
        assert_eq!(sizes.page_padding_bottom, Px(32.0), "page_padding_bottom");
        assert_eq!(sizes.page_body_gap, Px(18.0), "page_body_gap");
        assert_eq!(sizes.page_message_padding, Px(24.0), "page_message_padding");
        assert_eq!(sizes.section_height, Height(34.0), "section_height");
        assert_eq!(sizes.section_padding_x, Px(12.0), "section_padding_x");
        assert_eq!(sizes.section_text_size, FontPx(14.0), "section_text_size");
        assert_eq!(sizes.section_radius, Px(4.0), "section_radius");
        assert_eq!(sizes.section_underline, Px(2.0), "section_underline");
        assert_eq!(sizes.section_selected_weight, Weight::Semibold, "section_selected_weight");
        assert_eq!(sizes.section_gap, Px(2.0), "section_gap");
        assert_eq!(sizes.section_row_border, Px(1.0), "section_row_border");
        assert_eq!(sizes.location_padding_left, Px(16.0), "location_padding_left");
        assert_eq!(sizes.location_text_size, FontPx(12.0), "location_text_size");
        assert_eq!(sizes.tag_margin_left, Px(8.0), "tag_margin_left");
        assert_eq!(sizes.tag_padding_x, Px(5.0), "tag_padding_x");
        assert_eq!(sizes.tag_radius, Px(4.0), "tag_radius");
        assert_eq!(sizes.tag_text_size, FontPx(12.0), "tag_text_size");
        assert_eq!(sizes.cards_gap, Px(10.0), "cards_gap");
        assert_eq!(sizes.card_gap, Px(6.0), "card_gap");
        assert_eq!(sizes.card_padding_x, Px(12.0), "card_padding_x");
        assert_eq!(sizes.card_padding_y, Px(10.0), "card_padding_y");
        assert_eq!(sizes.card_title_gap, Px(6.0), "card_title_gap");
        assert_eq!(sizes.card_title_weight, Weight::Semibold, "card_title_weight");
        assert_eq!(sizes.card_detail_text_size, FontPx(12.0), "card_detail_text_size");
        assert_eq!(sizes.card_tags_gap, Px(4.0), "card_tags_gap");
        assert_eq!(sizes.card_min_width, Px(220.0), "card_min_width");
        assert_eq!(sizes.card_radius, Px(8.0), "card_radius");
        assert_eq!(sizes.card_border, Px(1.0), "card_border");
        assert_eq!(sizes.card_stripe_width, Px(3.0), "card_stripe_width");
        assert_eq!(sizes.scratches_gap, Px(8.0), "scratches_gap");
        assert_eq!(sizes.scratches_toolbar_gap, Px(12.0), "scratches_toolbar_gap");
        assert_eq!(sizes.scratches_hint_text_size, FontPx(12.0), "scratches_hint_text_size");
        assert_eq!(sizes.scratches_search_width, Px(420.0), "scratches_search_width");
        assert_eq!(sizes.scratches_text_size, FontPx(14.0), "scratches_text_size");
        assert_eq!(sizes.scratches_row_height, Height(24.0), "scratches_row_height");
        assert_eq!(sizes.scratches_row_padding_x, Px(8.0), "scratches_row_padding_x");
        assert_eq!(sizes.scratches_row_radius, Px(4.0), "scratches_row_radius");
        assert_eq!(sizes.scratches_group_gap, Px(4.0), "scratches_group_gap");
        assert_eq!(sizes.scratches_group_weight, Weight::Semibold, "scratches_group_weight");
        assert_eq!(sizes.scratches_count_weight, Weight::Normal, "scratches_count_weight");
        assert_eq!(sizes.scratches_input_row_height, Height(28.0), "scratches_input_row_height");
        assert_eq!(sizes.scratches_name_input_width, Px(260.0), "scratches_name_input_width");
        assert_eq!(sizes.scratches_error_text_size, FontPx(12.0), "scratches_error_text_size");
        assert_eq!(sizes.link_padding_x, Px(4.0), "link_padding_x");
        assert_eq!(sizes.link_radius, Px(4.0), "link_radius");
        assert_eq!(sizes.new_card_width, Px(260.0), "new_card_width");
        assert_eq!(sizes.new_card_gap, Px(6.0), "new_card_gap");
        assert_eq!(sizes.new_card_padding, Px(12.0), "new_card_padding");
        assert_eq!(sizes.new_card_radius, Px(8.0), "new_card_radius");
        assert_eq!(sizes.new_card_border, Px(1.0), "new_card_border");
        assert_eq!(sizes.new_card_title_gap, Px(6.0), "new_card_title_gap");
        assert_eq!(sizes.new_card_title_weight, Weight::Semibold, "new_card_title_weight");
        assert_eq!(sizes.new_card_status_text_size, FontPx(12.0), "new_card_status_text_size");
        assert_eq!(sizes.branch_menu_max_height, Height(360.0), "branch_menu_max_height");
        assert_eq!(sizes.branch_menu_padding, Px(4.0), "branch_menu_padding");
        assert_eq!(sizes.branch_menu_radius, Px(8.0), "branch_menu_radius");
        assert_eq!(sizes.branch_menu_border, Px(1.0), "branch_menu_border");
        assert_eq!(sizes.branch_menu_message_padding, Px(8.0), "branch_menu_message_padding");
        assert_eq!(sizes.branch_item_height, Height(28.0), "branch_item_height");
        assert_eq!(sizes.branch_item_gap, Px(8.0), "branch_item_gap");
        assert_eq!(sizes.branch_item_padding_x, Px(8.0), "branch_item_padding_x");
        assert_eq!(sizes.branch_item_radius, Px(4.0), "branch_item_radius");
        assert_eq!(sizes.branch_item_text_size, FontPx(14.0), "branch_item_text_size");
    }

    #[test]
    fn should_default_git_sizes_to_todays_values() {
        // when
        let sizes = defaults().sizes.git;

        // then
        assert_eq!(sizes.row_gap, Px(8.0), "row_gap");
        assert_eq!(sizes.row_height, Height(30.0), "row_height");
        assert_eq!(sizes.row_padding_x, Px(10.0), "row_padding_x");
        assert_eq!(sizes.row_border, Px(1.0), "row_border");
        assert_eq!(sizes.text_size, FontPx(14.0), "text_size");
        assert_eq!(sizes.detail_text_size, FontPx(12.0), "detail_text_size");
        assert_eq!(sizes.body_weight, Weight::Normal, "body_weight");
        assert_eq!(sizes.author_max_width, Px(160.0), "author_max_width");
        assert_eq!(sizes.mark_width, Px(12.0), "mark_width");
        assert_eq!(sizes.mark_text_size, FontPx(12.0), "mark_text_size");
        assert_eq!(sizes.mark_weight, Weight::Bold, "mark_weight");
        assert_eq!(sizes.tag_padding_x, Px(4.0), "tag_padding_x");
        assert_eq!(sizes.tag_radius, Px(4.0), "tag_radius");
        assert_eq!(sizes.tag_text_size, FontPx(12.0), "tag_text_size");
        assert_eq!(sizes.segment_gap, Px(6.0), "segment_gap");
        assert_eq!(sizes.segment_padding_x, Px(10.0), "segment_padding_x");
        assert_eq!(sizes.segment_padding_y, Px(4.0), "segment_padding_y");
        assert_eq!(sizes.segment_radius, Px(6.0), "segment_radius");
        assert_eq!(sizes.segment_border, Px(1.0), "segment_border");
        assert_eq!(sizes.segment_text_size, FontPx(14.0), "segment_text_size");
        assert_eq!(sizes.toggle_gap, Px(6.0), "toggle_gap");
        assert_eq!(sizes.toggle_padding_x, Px(8.0), "toggle_padding_x");
        assert_eq!(sizes.toggle_padding_y, Px(2.0), "toggle_padding_y");
        assert_eq!(sizes.toggle_radius, Px(6.0), "toggle_radius");
        assert_eq!(sizes.toggle_border, Px(1.0), "toggle_border");
        assert_eq!(sizes.toggle_text_size, FontPx(14.0), "toggle_text_size");
        assert_eq!(sizes.group_title_gap, Px(6.0), "group_title_gap");
        assert_eq!(sizes.group_title_padding_top, Px(12.0), "group_title_padding_top");
        assert_eq!(sizes.group_title_padding_bottom, Px(4.0), "group_title_padding_bottom");
        assert_eq!(sizes.group_title_text_size, FontPx(14.0), "group_title_text_size");
        assert_eq!(sizes.group_title_weight, Weight::Semibold, "group_title_weight");
        assert_eq!(sizes.group_hint_text_size, FontPx(12.0), "group_hint_text_size");
        assert_eq!(sizes.group_hint_weight, Weight::Normal, "group_hint_weight");
        assert_eq!(sizes.error_gap, Px(8.0), "error_gap");
        assert_eq!(sizes.error_padding_y, Px(8.0), "error_padding_y");
        assert_eq!(sizes.box_radius, Px(8.0), "box_radius");
        assert_eq!(sizes.box_border, Px(1.0), "box_border");
        assert_eq!(sizes.message_padding, Px(8.0), "message_padding");
        assert_eq!(sizes.section_gap, Px(4.0), "section_gap");
        assert_eq!(sizes.summary_gap, Px(8.0), "summary_gap");
        assert_eq!(sizes.summary_padding_x, Px(10.0), "summary_padding_x");
        assert_eq!(sizes.summary_padding_y, Px(6.0), "summary_padding_y");
        assert_eq!(sizes.log_toolbar_gap, Px(10.0), "log_toolbar_gap");
        assert_eq!(sizes.log_toolbar_padding_x, Px(24.0), "log_toolbar_padding_x");
        assert_eq!(sizes.log_toolbar_padding_top, Px(12.0), "log_toolbar_padding_top");
        assert_eq!(sizes.log_buttons_gap, Px(4.0), "log_buttons_gap");
        assert_eq!(sizes.log_padding_bottom, Px(32.0), "log_padding_bottom");
        assert_eq!(sizes.log_margin_bottom, Px(24.0), "log_margin_bottom");
        assert_eq!(sizes.log_more_padding_x, Px(8.0), "log_more_padding_x");
        assert_eq!(sizes.commit_message_padding, Px(16.0), "commit_message_padding");
        assert_eq!(sizes.commit_header_gap, Px(4.0), "commit_header_gap");
        assert_eq!(sizes.commit_header_padding_x, Px(16.0), "commit_header_padding_x");
        assert_eq!(sizes.commit_header_padding_y, Px(12.0), "commit_header_padding_y");
        assert_eq!(sizes.commit_header_border, Px(1.0), "commit_header_border");
        assert_eq!(sizes.commit_subject_padding_top, Px(4.0), "commit_subject_padding_top");
        assert_eq!(sizes.commit_subject_weight, Weight::Medium, "commit_subject_weight");
        assert_eq!(sizes.diff_header_gap, Px(8.0), "diff_header_gap");
        assert_eq!(sizes.diff_header_padding_x, Px(10.0), "diff_header_padding_x");
        assert_eq!(sizes.diff_header_padding_y, Px(6.0), "diff_header_padding_y");
        assert_eq!(sizes.diff_header_border, Px(1.0), "diff_header_border");
        assert_eq!(sizes.diff_header_weight, Weight::Medium, "diff_header_weight");
    }

    #[test]
    fn should_default_settings_sizes_to_todays_values() {
        // when
        let sizes = defaults().sizes.settings;

        // then
        assert_eq!(sizes.page_gap, Px(16.0), "page_gap");
        assert_eq!(sizes.page_padding, Px(24.0), "page_padding");
        assert_eq!(sizes.text_size, FontPx(14.0), "text_size");
        assert_eq!(sizes.title_text_size, FontPx(20.0), "title_text_size");
        assert_eq!(sizes.title_weight, Weight::Bold, "title_weight");
        assert_eq!(sizes.nav_width, Px(200.0), "nav_width");
        assert_eq!(sizes.nav_padding_y, Px(8.0), "nav_padding_y");
        assert_eq!(sizes.nav_border, Px(1.0), "nav_border");
        assert_eq!(sizes.nav_base_indent, Px(12.0), "nav_base_indent");
        assert_eq!(sizes.nav_item_padding_y, Px(4.0), "nav_item_padding_y");
        assert_eq!(sizes.appearance_gap, Px(16.0), "appearance_gap");
        assert_eq!(sizes.appearance_label_width, Px(80.0), "appearance_label_width");
        assert_eq!(sizes.segmented_radius, Px(6.0), "segmented_radius");
        assert_eq!(sizes.segmented_border, Px(1.0), "segmented_border");
        assert_eq!(sizes.segment_padding_x, Px(12.0), "segment_padding_x");
        assert_eq!(sizes.segment_padding_y, Px(4.0), "segment_padding_y");
        assert_eq!(sizes.table_gap, Px(16.0), "table_gap");
        assert_eq!(sizes.table_padding_right, Px(16.0), "table_padding_right");
        assert_eq!(sizes.list_gap, Px(8.0), "list_gap");
        assert_eq!(sizes.row_padding_y, Px(4.0), "row_padding_y");
        assert_eq!(sizes.licence_width, Px(240.0), "licence_width");
        assert_eq!(sizes.table_header_weight, Weight::Semibold, "table_header_weight");
        assert_eq!(sizes.licence_text_padding_bottom, Px(16.0), "licence_text_padding_bottom");
        assert_eq!(sizes.licence_text_margin_bottom, Px(16.0), "licence_text_margin_bottom");
        assert_eq!(sizes.licence_text_border, Px(1.0), "licence_text_border");
    }

    #[test]
    fn should_default_controls_sizes_to_todays_values() {
        // when
        let sizes = defaults().sizes.controls;

        // then
        assert_eq!(sizes.prompt_width, Px(360.0), "prompt_width");
        assert_eq!(sizes.prompt_gap, Px(8.0), "prompt_gap");
        assert_eq!(sizes.prompt_padding, Px(16.0), "prompt_padding");
        assert_eq!(sizes.prompt_radius, Px(8.0), "prompt_radius");
        assert_eq!(sizes.prompt_border, Px(1.0), "prompt_border");
        assert_eq!(sizes.prompt_title_weight, Weight::Semibold, "prompt_title_weight");
        assert_eq!(sizes.prompt_detail_text_size, FontPx(14.0), "prompt_detail_text_size");
        assert_eq!(sizes.prompt_buttons_gap, Px(8.0), "prompt_buttons_gap");
        assert_eq!(sizes.prompt_buttons_margin_top, Px(8.0), "prompt_buttons_margin_top");
        assert_eq!(sizes.prompt_button_padding_x, Px(12.0), "prompt_button_padding_x");
        assert_eq!(sizes.prompt_button_padding_y, Px(4.0), "prompt_button_padding_y");
        assert_eq!(sizes.prompt_button_radius, Px(6.0), "prompt_button_radius");
        assert_eq!(sizes.prompt_button_text_size, FontPx(14.0), "prompt_button_text_size");
        assert_eq!(sizes.prompt_button_border, Px(1.0), "prompt_button_border");
        assert_eq!(sizes.tooltip_padding_x, Px(8.0), "tooltip_padding_x");
        assert_eq!(sizes.tooltip_padding_y, Px(4.0), "tooltip_padding_y");
        assert_eq!(sizes.tooltip_radius, Px(6.0), "tooltip_radius");
        assert_eq!(sizes.tooltip_border, Px(1.0), "tooltip_border");
        assert_eq!(sizes.tooltip_text_size, FontPx(12.0), "tooltip_text_size");
        assert_eq!(sizes.paging_gap, Px(12.0), "paging_gap");
        assert_eq!(sizes.paging_text_size, FontPx(12.0), "paging_text_size");
        assert_eq!(sizes.input_height, Height(28.0), "input_height");
        assert_eq!(sizes.input_caret_width, Px(2.0), "input_caret_width");
        assert_eq!(sizes.input_padding_x, Px(8.0), "input_padding_x");
        assert_eq!(sizes.input_radius, Px(6.0), "input_radius");
        assert_eq!(sizes.input_border, Px(1.0), "input_border");
        assert_eq!(sizes.input_text_size, FontPx(14.0), "input_text_size");
        assert_eq!(sizes.input_disabled_opacity, Fraction(0.6), "input_disabled_opacity");
        assert_eq!(sizes.input_marked_underline, Px(1.0), "input_marked_underline");
        assert_eq!(sizes.progress_width, Px(16.0), "progress_width");
        assert_eq!(sizes.progress_height, Height(4.0), "progress_height");
        assert_eq!(sizes.progress_segment, Px(6.0), "progress_segment");
        assert_eq!(sizes.scrollbar_thickness, Px(10.0), "scrollbar_thickness");
        assert_eq!(sizes.scrollbar_min_thumb, Px(24.0), "scrollbar_min_thumb");
        assert_eq!(sizes.scrollbar_inset, Px(2.0), "scrollbar_inset");
    }

    #[test]
    fn should_default_terminal_sizes_to_todays_values() {
        // when
        let terminal = defaults().sizes.terminal;

        // then
        assert_eq!((terminal.text_size, terminal.line_height_factor, terminal.padding), (FontPx(14.), Factor(1.3), Px(6.)));
        assert_eq!((terminal.cursor_thickness, terminal.line_thickness), (Px(2.), Px(1.)));
        assert_eq!((terminal.normal_weight, terminal.bold_weight), (Weight::Normal, Weight::Bold));
    }

    #[test]
    fn should_default_diff_sizes_to_todays_values() {
        // when
        let diff = defaults().sizes.diff;

        // then
        assert_eq!((diff.row_height, diff.text_size, diff.line_padding, diff.hunk_padding_left), (Height(20.), FontPx(13.), Px(4.), Px(8.)));
        assert_eq!((diff.header_gap, diff.header_padding_x, diff.header_weight), (Px(6.), Px(4.), Weight::Medium));
        assert_eq!(diff.number_padding_right, Px(4.));
    }

    #[test]
    fn should_default_editor_sizes_to_todays_values() {
        // when
        let editor = defaults().sizes.editor;

        // then
        assert_eq!((editor.text_size, editor.line_height_factor, editor.caret_width), (FontPx(14.), Factor(1.5), Px(2.)));
        assert_eq!((editor.notice_padding_x, editor.notice_padding_y, editor.notice_text_size), (Px(12.), Px(4.), FontPx(14.)));
        assert_eq!(
            (editor.scratch_strip_padding_x, editor.scratch_strip_padding_y, editor.scratch_strip_text_size),
            (Px(12.), Px(4.), FontPx(12.))
        );
        assert_eq!(
            (editor.scratch_header_gap, editor.scratch_header_border, editor.scratch_bar_gap, editor.scratch_error_padding),
            (Px(8.), Px(1.), Px(12.), Px(12.))
        );
        assert_eq!(
            (editor.scratch_button_gap, editor.scratch_button_padding_x, editor.scratch_button_radius, editor.scratch_button_border),
            (Px(4.), Px(8.), Px(4.), Px(1.))
        );
    }

    #[test]
    fn should_read_the_tab_badge_tint_and_group_sizes() {
        // when
        let tabs = defaults().sizes.tabs;

        // then
        assert_eq!((tabs.badge_size, tabs.badge_offset, tabs.badge_ring), (Px(10.), Px(3.), Px(1.5)));
        assert_eq!((tabs.rail_width, tabs.rail_inset), (Px(2.), Px(4.)));
        assert_eq!((tabs.tint, tabs.tint_hover, tabs.tint_shown, tabs.tint_outline), (Fraction(0.12), Fraction(0.18), Fraction(0.22), Fraction(0.45)));
        assert_eq!((tabs.group_gap, tabs.group_head_text_size, tabs.group_count_radius), (Px(4.), FontPx(12.), Px(8.)));
    }

    #[test]
    fn should_default_tab_and_window_sizes_to_todays_values() {
        // when
        let sizes = defaults().sizes;

        // then
        let tabs = &sizes.tabs;
        assert_eq!((tabs.row_height, tabs.row_border, tabs.strip_padding_x, tabs.gap), (Height(36.), Px(1.), Px(4.), Px(2.)));
        assert_eq!((tabs.tools_gap, tabs.tools_padding_left), (Px(4.), Px(8.)));
        assert_eq!(
            (tabs.tab_height, tabs.tab_padding_left, tabs.tab_padding_right, tabs.tab_radius, tabs.tab_border),
            (Height(28.), Px(10.), Px(6.), Px(6.), Px(1.))
        );
        assert_eq!((tabs.tab_text_size, tabs.tab_inner_gap, tabs.tab_icon_size, tabs.title_max_width), (FontPx(14.), Px(6.), Px(16.), Px(220.)));
        assert_eq!(tabs.close_radius, Px(4.));
        assert_eq!((tabs.underline_inset, tabs.underline_height, tabs.underline_radius), (Px(6.), Px(2.), Px(4.)));
        assert_eq!((tabs.drop_marker_width, tabs.drop_marker_inset, tabs.drop_marker_radius), (Px(2.), Px(2.), Px(4.)));
        assert_eq!(tabs.drag_chip_opacity, Fraction(0.85));
        assert_eq!((tabs.toolbar_button_padding, tabs.toolbar_button_radius, tabs.toolbar_icon_size), (Px(4.), Px(6.), Px(20.)));
        assert_eq!(
            (tabs.preview_header_height, tabs.preview_padding_x, tabs.preview_border, tabs.preview_text_size),
            (Height(24.), Px(8.), Px(1.), FontPx(12.))
        );
        assert_eq!((tabs.preview_radius, tabs.preview_opacity), (Px(6.), Fraction(0.9)));
        let window = &sizes.window;
        assert_eq!((window.corner_radius, window.border, window.control_width, window.control_icon_size), (Px(8.), Px(1.), Px(46.), Px(16.)));
        assert_eq!((window.welcome_gap, window.welcome_title_size), (Px(16.), FontPx(24.)));
        assert_eq!(
            (window.welcome_recent_gap, window.welcome_recent_padding_x, window.welcome_recent_padding_y, window.welcome_recent_radius),
            (Px(8.), Px(12.), Px(4.), Px(6.))
        );
        assert_eq!((window.button_padding_x, window.button_padding_y, window.button_radius), (Px(16.), Px(8.), Px(6.)));
    }

    #[test]
    fn should_default_tree_sizes_to_todays_values() {
        // when
        let sizes = defaults().sizes;

        // then
        let tree = &sizes.tree;
        assert_eq!(sizes.icon, Px(16.));
        assert_eq!((tree.header_height, tree.header_padding_left, tree.header_padding_right), (Height(36.), Px(12.), Px(4.)));
        assert_eq!((tree.header_title_size, tree.header_title_weight), (FontPx(14.), Weight::Semibold));
        assert_eq!((tree.header_tools_gap, tree.header_button_padding, tree.header_button_radius), (Px(4.), Px(4.), Px(6.)));
        assert_eq!(tree.border, Px(1.));
        assert_eq!((tree.menu_offset, tree.menu_width, tree.menu_padding, tree.menu_radius), (Px(4.), Px(260.), Px(4.), Px(8.)));
        assert_eq!((tree.menu_text_size, tree.menu_weight), (FontPx(14.), Weight::Normal));
        assert_eq!(
            (tree.menu_section_padding_x, tree.menu_section_padding_top, tree.menu_section_padding_bottom, tree.menu_section_text_size),
            (Px(8.), Px(6.), Px(2.), FontPx(12.))
        );
        assert_eq!(
            (tree.menu_item_height, tree.menu_item_gap, tree.menu_item_padding_x, tree.menu_item_radius),
            (Height(28.), Px(8.), Px(8.), Px(4.))
        );
        assert_eq!(
            (tree.badge_size, tree.badge_radius, tree.badge_text_size, tree.badge_weight),
            (Px(18.), Px(4.), FontPx(12.), Weight::Bold)
        );
        assert_eq!((tree.row_height, tree.text_size, tree.icon_gap), (Height(24.), FontPx(13.), Px(4.)));
        assert_eq!((tree.padding_left, tree.padding_right, tree.project_name_weight), (Px(8.), Px(8.), Weight::Semibold));
        assert_eq!(
            (tree.tag_gap, tree.tag_padding_x, tree.tag_radius, tree.tag_text_size),
            (Px(4.), Px(6.), Px(4.), FontPx(12.))
        );
        assert_eq!(tree.chevron_radius, Px(4.));
        assert_eq!((tree.count_padding_x, tree.count_radius, tree.count_text_size), (Px(6.), Px(8.), FontPx(12.)));
        assert_eq!((tree.folder_count_padding_x, tree.folder_count_text_size), (Px(2.), FontPx(11.)));
        assert_eq!((tree.action_padding, tree.action_radius), (Px(2.), Px(4.)));
        assert_eq!((tree.resize_handle_width, tree.default_width), (Px(6.), Px(288.)));
    }

    #[test]
    fn should_default_menu_sizes_to_intellij_popups() {
        // when
        let menu = defaults().sizes.menu;

        // then
        assert_eq!((menu.min_width, menu.padding_y, menu.radius, menu.border), (Px(240.), Px(6.), Px(0.), Px(1.)));
        assert_eq!((menu.text_size, menu.weight), (FontPx(13.), Weight::Normal));
        assert_eq!(
            (menu.item_height, menu.item_margin_x, menu.item_padding_x, menu.item_radius),
            (Height(26.), Px(6.), Px(10.), Px(4.))
        );
        assert_eq!((menu.shortcut_gap, menu.separator_margin_y, menu.separator_inset_x), (Px(32.), Px(4.), Px(10.)));
        assert_eq!(menu.pending_opacity, Fraction(0.5));
    }

    #[test]
    fn should_default_selection_shape_to_flat() {
        // when
        let tree = defaults().sizes.tree;

        // then
        assert_eq!((tree.selection_radius, tree.selection_inset), (Px(0.), Px(0.)));
        assert!(tree.flat_selection());
    }
}
