//! A commit's label in the tree: its message without the prefix most of the worktree's commits share.

use std::cmp::Reverse;
use std::collections::HashMap;

const SEPARATOR: &str = " - ";

/// The longest prefix ending in ` - ` that at least two and at least half of `messages` start with; empty when none.
pub fn shared_prefix<'a>(messages: &[&'a str]) -> &'a str {
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for message in messages {
        for (at, _) in message.match_indices(SEPARATOR) {
            *counts.entry(&message[..at + SEPARATOR.len()]).or_default() += 1;
        }
    }
    let shared = counts.into_iter().filter(|(_, count)| *count >= 2 && count * 2 >= messages.len());
    // longest, then most shared, then first by name: the same pick on every rebuild
    shared.max_by_key(|(prefix, count)| (prefix.len(), *count, Reverse(*prefix))).map_or("", |(prefix, _)| prefix)
}

/// `message` without `shared` when it is longer than it, else without a leading `[KEY-123] ` Jira key.
pub fn commit_label<'a>(message: &'a str, shared: &str) -> &'a str {
    match message.strip_prefix(shared) {
        Some(rest) if !shared.is_empty() && !rest.is_empty() => rest,
        _ => without_jira_key(message),
    }
}

/// `Fix it` of `[GEN-817] Fix it`.
fn without_jira_key(message: &str) -> &str {
    let Some((key, rest)) = message.strip_prefix('[').and_then(|after| after.split_once("] ")) else {
        return message;
    };
    let Some((project, number)) = key.split_once('-') else {
        return message;
    };
    let is_project = !project.is_empty() && project.chars().all(|c| c.is_ascii_uppercase());
    let is_number = !number.is_empty() && number.chars().all(|c| c.is_ascii_digit());
    if is_project && is_number { rest } else { message }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GEN_817: [&str; 4] = [
        "[GEN-817] IVE - Integrated Vibe Editor - dragged tab previews its window",
        "[GEN-817] IVE - Integrated Vibe Editor - ive MCP server for the repo",
        "[GEN-817] IVE - Integrated Vibe Editor - no resize in fullscreen, drag leaves it",
        "[GEN-817] IVE - initial Rust app, icons and HTML mock",
    ];

    #[test]
    fn should_drop_the_prefix_most_commits_share() {
        // given
        let shared = shared_prefix(&GEN_817);

        // when
        let labels: Vec<&str> = GEN_817.iter().map(|message| commit_label(message, shared)).collect();

        // then
        assert_eq!(shared, "[GEN-817] IVE - Integrated Vibe Editor - ");
        assert_eq!(labels, [
            "dragged tab previews its window",
            "ive MCP server for the repo",
            "no resize in fullscreen, drag leaves it",
            "IVE - initial Rust app, icons and HTML mock",
        ]);
    }

    #[test]
    fn should_share_no_prefix_with_fewer_than_two_messages() {
        // given
        let messages = ["[GEN-817] IVE - Integrated Vibe Editor - one"];

        // when
        let shared = shared_prefix(&messages);

        // then
        assert_eq!(shared, "");
        assert_eq!(commit_label(messages[0], shared), "IVE - Integrated Vibe Editor - one");
    }

    #[test]
    fn should_share_no_prefix_when_fewer_than_half_have_it() {
        // given
        let messages = ["a - x", "a - y", "b", "c", "d"];

        // when
        let shared = shared_prefix(&messages);

        // then
        assert_eq!(shared, "");
    }

    #[test]
    fn should_keep_messages_without_a_separator() {
        // given
        let messages = ["Fix the thing", "[OHQ-3] Add WAL", "LiveView shell"];

        // when
        let shared = shared_prefix(&messages);
        let labels: Vec<&str> = messages.iter().map(|message| commit_label(message, shared)).collect();

        // then
        assert_eq!(labels, ["Fix the thing", "Add WAL", "LiveView shell"]);
    }

    #[test]
    fn should_pick_the_same_prefix_every_time_when_two_tie() {
        // given
        let messages = ["[GEN-2] Web - a", "[GEN-1] App - b", "[GEN-2] Web - c", "[GEN-1] App - d"];

        // when
        let picked: Vec<&str> = (0..50).map(|_| shared_prefix(&messages)).collect();

        // then
        assert!(picked.iter().all(|prefix| *prefix == "[GEN-1] App - "), "{picked:?}");
    }

    #[test]
    fn should_never_label_a_commit_empty() {
        // given
        let messages = ["[GEN-1] a - ", "[GEN-1] a - b", "[GEN-1] a - c"];

        // when
        let shared = shared_prefix(&messages);

        // then
        assert_eq!(shared, "[GEN-1] a - ");
        assert_eq!(commit_label(messages[0], shared), "a - ");
    }
}
