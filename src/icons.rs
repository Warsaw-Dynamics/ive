use std::path::Path;

use gpui::{ImageSource, Resource, SharedString};

use crate::theme::Appearance;

// IntelliJ Platform New UI icons, Apache 2.0: see assets/icons/NOTICE
pub const FOLDER: &str = "folder.svg";
/// IntelliJ excludeRoot.svg: a folder git ignores or an exclusion rule names
pub const FOLDER_EXCLUDED: &str = "folder-excluded.svg";
pub const CHEVRON_RIGHT: &str = "chevron-right.svg";
pub const CHEVRON_DOWN: &str = "chevron-down.svg";
pub const SETTINGS: &str = "settings.svg";
/// IntelliJ settings@20x20.svg, drawn for 20 px toolbars
pub const SETTINGS_20: &str = "settings-20.svg";
/// IntelliJ toolwindows/notifications@20x20.svg: the tab row's bell for processes that wait for the user
pub const NOTIFICATIONS_20: &str = "notifications-20.svg";
pub const CLOSE: &str = "close.svg";
pub const ADD: &str = "add.svg";
pub const BRANCH: &str = "branch.svg";
/// IntelliJ nodes/star.svg, the favourite branch's: the main checkout's badge in the tree
pub const STAR: &str = "star.svg";
pub const HISTORY: &str = "history.svg";
pub const FILE_TEXT: &str = "file-text.svg";
pub const CHANGES: &str = "changes.svg";
pub const COMMIT: &str = "commit.svg";
pub const PROJECT_FOLDER: &str = "project-folder.svg";
pub const STOP: &str = "stop.svg";
pub const COPY: &str = "copy.svg";
/// a running static server's page, opened in the browser
pub const WEB: &str = "web.svg";
pub const AGENT: &str = "agent.svg";
/// drawn for IVE, not from the IntelliJ set
pub const TERMINAL: &str = "terminal.svg";
/// drawn for IVE: an eye struck through, Hide worktree
pub const HIDE: &str = "hide.svg";
/// drawn for IVE: Select Opened File's crosshair
pub const LOCATE: &str = "locate.svg";
/// modified IntelliJ icon: the run widget's bold restart arrow with rerun.svg's green triangle
pub const RESTART: &str = "restart.svg";
pub const RUN: &str = "run.svg";
pub const RUN_CONFIG: &str = "run-config.svg";
/// modified IntelliJ icon: nodes/module.svg's folder with a run triangle as its corner badge
pub const FOLDER_RUN: &str = "folder-run.svg";
/// a static server's configuration and run
pub const FILE_HTML: &str = "file-html.svg";
/// IntelliJ's scratch clock, drawn over a file icon
pub const SCRATCH_BADGE: &str = "scratch.svg";
pub const WARNING: &str = "warning.svg";
pub const WINDOW_MINIMIZE: &str = "window-minimize.svg";
pub const WINDOW_MAXIMIZE: &str = "window-maximize.svg";
pub const WINDOW_RESTORE: &str = "window-restore.svg";
pub const WINDOW_CLOSE: &str = "window-close.svg";
/// IntelliJ's Open in Tool Window: a detached tab's way back to the main window
pub const DOCK: &str = "dock.svg";

/// The embedded icon `name` in the variant drawn for `appearance`.
pub fn embedded(appearance: Appearance, name: &str) -> ImageSource {
    // built explicitly: `ImageSource::from(&str)` would read the path as a URI if it parses as one
    ImageSource::Resource(Resource::Embedded(path(appearance, name)))
}

/// The asset path of icon `name`, for elements such as `svg()` that take a path.
pub fn path(appearance: Appearance, name: &str) -> SharedString {
    asset_path(appearance, name).into()
}

fn asset_path(appearance: Appearance, name: &str) -> String {
    let variant = match appearance {
        Appearance::Dark => "dark",
        Appearance::Light => "light",
    };
    format!("icons/{variant}/{name}")
}

/// The icon name for a file, by name or extension; files without a known type get the plain text icon.
pub fn file_icon(file_name: &str) -> &'static str {
    match file_name {
        "Dockerfile" | "Containerfile" => return "file-docker.svg",
        "BUILD" | "WORKSPACE" | "MODULE.bazel" => return "file-bazel.svg",
        _ => {}
    }
    let extension = Path::new(file_name)
        .extension()
        .map(|extension| extension.to_string_lossy().to_ascii_lowercase());
    match extension.as_deref() {
        Some("java") => "file-java.svg",
        Some("js" | "mjs" | "cjs" | "jsx") => "file-javaScript.svg",
        Some("ts" | "mts" | "cts" | "tsx") => "file-typeScript.svg",
        Some("json") => "file-json.svg",
        Some("xml" | "pom" | "xsd" | "svg") => "file-xml.svg",
        Some("yml" | "yaml") => "file-yaml.svg",
        Some("toml") => "file-toml.svg",
        Some("md" | "markdown") => "file-markdown.svg",
        Some("html" | "htm" | "jte") => "file-html.svg",
        Some("css" | "scss" | "less") => "file-css.svg",
        Some("sh" | "bash" | "zsh") => "file-shell.svg",
        Some("sql") => "file-sql.svg",
        Some("properties" | "env") => "file-properties.svg",
        Some("png" | "jpg" | "jpeg" | "gif" | "webp" | "ico" | "bmp") => "file-image.svg",
        Some("zip" | "jar" | "war" | "tar" | "gz" | "tgz" | "7z" | "rar") => "file-archive.svg",
        Some("csv" | "tsv") => "file-csv.svg",
        Some("ini" | "conf" | "cfg" | "editorconfig") => "file-config.svg",
        Some("gradle" | "kts") => "file-gradle.svg",
        Some("dockerfile") => "file-docker.svg",
        Some("tf" | "tfvars" | "hcl") => "file-terraform.svg",
        Some("graphql" | "gql" | "graphqls") => "file-graphql.svg",
        Some("groovy" | "gvy") => "file-groovy.svg",
        Some("ipynb") => "file-jupyter.svg",
        Some("bat" | "cmd" | "ps1" | "exe" | "dll") => "file-microsoftWindows.svg",
        Some("pl" | "pm") => "file-perl.svg",
        Some("swift") => "file-swiftLang.svg",
        Some("cs") => "file-Csharp.svg",
        Some("c") => "file-c.svg",
        Some("cpp" | "cc" | "cxx" | "hpp" | "hh") => "file-cpp.svg",
        Some("h") => "file-h.svg",
        Some("bazel" | "bzl") => "file-bazel.svg",
        _ => "file-text.svg",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::Assets;

    #[test]
    fn should_pick_icon_by_name_or_extension_ignoring_case() {
        // given
        let names = ["Main.java", "README.MD", "Cargo.toml", "build.gradle.kts", "Dockerfile"];

        // when
        let icons: Vec<_> = names.iter().map(|name| file_icon(name)).collect();

        // then
        assert_eq!(icons, ["file-java.svg", "file-markdown.svg", "file-toml.svg", "file-gradle.svg", "file-docker.svg"]);
    }

    #[test]
    fn should_use_text_icon_for_unknown_or_missing_extension() {
        // given
        let names = ["Makefile", "main.rs", ".gitignore"];

        // when
        let icons: Vec<_> = names.iter().map(|name| file_icon(name)).collect();

        // then
        assert!(icons.iter().all(|icon| *icon == "file-text.svg"));
    }

    #[test]
    fn should_embed_every_referenced_icon_in_both_appearances() {
        // given
        let extensions = [
            "java", "js", "ts", "json", "xml", "yml", "toml", "md", "html", "css", "sh", "sql", "properties",
            "png", "zip", "csv", "ini", "gradle", "txt", "tf", "graphql", "groovy", "ipynb", "bat", "pl", "swift",
            "cs", "c", "cpp", "h", "bzl",
        ];
        let mut icons: Vec<&str> = extensions.iter().map(|ext| file_icon(&format!("f.{ext}"))).collect();
        icons.extend([file_icon("Dockerfile"), FOLDER, CHEVRON_RIGHT, CHEVRON_DOWN, SETTINGS, CLOSE]);
        icons.extend([WINDOW_MINIMIZE, WINDOW_MAXIMIZE, WINDOW_RESTORE, WINDOW_CLOSE]);
        icons.extend([ADD, BRANCH, PROJECT_FOLDER, STOP, TERMINAL, AGENT, WARNING]);
        icons.extend([HISTORY, CHANGES, COMMIT]);
        icons.extend([RUN, SCRATCH_BADGE, DOCK, FOLDER_EXCLUDED]);
        icons.extend([RESTART, RUN_CONFIG, FOLDER_RUN, FILE_HTML, STAR]);

        // when
        let missing: Vec<_> = [Appearance::Dark, Appearance::Light]
            .into_iter()
            .flat_map(|appearance| icons.iter().map(move |icon| asset_path(appearance, icon)))
            .filter(|path| Assets::get(path).is_none())
            .collect();

        // then
        assert!(missing.is_empty(), "icons not embedded: {missing:?}");
    }
}
