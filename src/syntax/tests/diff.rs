use super::keys_at;

#[test]
fn should_tell_diff_added_from_removed_and_context_lines() {
    // given
    let source = "--- a/run.rs\n+++ b/run.rs\n@@ -1,2 +1,2 @@\n keep();\n-old();\n+new();\n";

    // when
    let keys = keys_at("Diff", source, ["keep", "old", "new", "--- a", "+++ b"]);

    // then
    assert_eq!(
        keys,
        [
            Some("DIFF.CONTEXT"),
            Some("DIFF.REMOVED"),
            Some("DIFF.ADDED"),
            Some("DIFF.FILE_HEADER"),
            Some("DIFF.FILE_HEADER"),
        ]
    );
}

#[test]
fn should_mark_diff_hunk_header_line_range_and_extended_header() {
    // given
    let source = "diff --git a/x b/x\nindex 3f2a9c1..b7e04d2 100644\n--- a/x\n+++ b/x\n@@ -12,9 +12,11 @@ fn run\n x\n";

    // when
    let keys = keys_at("Diff", source, ["--git", "index", "3f2a9c1", "100644", "@@ -", "-12,9", "+12,11"]);

    // then
    assert_eq!(
        keys,
        [
            Some("DIFF.COMMAND"),
            Some("DIFF.EXTENDED_HEADER"),
            Some("DIFF.COMMIT"),
            Some("DIFF.MODE"),
            Some("DIFF.HUNK_HEADER"),
            Some("DIFF.LINE_RANGE"),
            Some("DIFF.LINE_RANGE"),
        ]
    );
}

#[test]
fn should_mark_diff_renamed_file_name_and_comment() {
    // given
    let source = "# moved\ndiff --git a/a b/b\nrename from src/paint.rs\nrename to src/highlight.rs\n";

    // when
    let keys = keys_at("Diff", source, ["moved", "rename from", "src/paint.rs"]);

    // then
    assert_eq!(keys, [Some("DIFF.COMMENT"), Some("DIFF.EXTENDED_HEADER"), Some("DIFF.FILE_NAME")]);
}
