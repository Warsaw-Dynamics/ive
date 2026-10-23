//! What dropping terminals, Claude Code sessions and runs on a worktree does, and the hint that says so: terminals
//! and Claude Code move there, runs stay; with the copy modifier, the same things start there instead.

use crate::process::{ProcessId, ProcessKind};
use crate::project::CheckoutRef;
use crate::transfer::Verb;

/// A dragged process, as the drop sees it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dragged {
    pub id: ProcessId,
    pub kind: ProcessKind,
    /// as its row shows it
    pub name: String,
    pub checkout: Option<CheckoutRef>,
    /// a run: whether the target checkout has its `.run.toml` folder
    pub config_in_target: bool,
}

/// The checkout dropped on, and its label in the tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Labelled {
    pub checkout: CheckoutRef,
    pub label: String,
}

/// The modifiers held at the drop: the verb they force, and the copy modifier's name for hints.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Held {
    pub forced: Option<Verb>,
    pub copy_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessDrop {
    /// terminals and Claude Code sessions listed under the target from now on
    Move(Vec<ProcessId>),
    /// the same processes started again in the target
    Start(Vec<ProcessId>),
    Refused,
}

/// What dropping `dragged` on `target` does, and the hint saying it.
pub fn plan_process_drop(dragged: &[Dragged], target: &Labelled, held: &Held) -> (ProcessDrop, String) {
    let elsewhere: Vec<&Dragged> = dragged.iter().filter(|process| process.checkout.as_ref() != Some(&target.checkout)).collect();
    if elsewhere.is_empty() {
        return (ProcessDrop::Refused, format!("Already in {}", target.label));
    }
    match held.forced {
        Some(Verb::Copy) => start_copies(&elsewhere, target),
        _ => move_terminals(&elsewhere, target, held),
    }
}

fn move_terminals(elsewhere: &[&Dragged], target: &Labelled, held: &Held) -> (ProcessDrop, String) {
    let (runs, moved): (Vec<&Dragged>, Vec<&Dragged>) = elsewhere.iter().partition(|process| process.kind == ProcessKind::Run);
    let copy = &held.copy_name;
    if moved.is_empty() {
        let hint = match runs.len() {
            1 => format!("A run stays in its worktree; {copy}+drop starts its copy in {}", target.label),
            _ => format!("Runs stay in their worktree; {copy}+drop starts their copies in {}", target.label),
        };
        return (ProcessDrop::Refused, hint);
    }
    let staying = match runs.len() {
        0 => String::new(),
        1 => format!(" · 1 run stays ({copy}+drop starts its copy there)"),
        count => format!(" · {count} runs stay ({copy}+drop starts their copies there)"),
    };
    let hint = format!("Move {} to {}{staying}", named(&moved), target.label);
    (ProcessDrop::Move(moved.iter().map(|process| process.id).collect()), hint)
}

fn start_copies(elsewhere: &[&Dragged], target: &Labelled) -> (ProcessDrop, String) {
    let (missing, started): (Vec<&Dragged>, Vec<&Dragged>) = elsewhere.iter().partition(|process| process.kind == ProcessKind::Run && !process.config_in_target);
    let missing_note = match missing.is_empty() {
        true => String::new(),
        false => format!(" · {}: no such configuration there", missing.iter().map(|process| process.name.as_str()).collect::<Vec<_>>().join(", ")),
    };
    if started.is_empty() {
        return (ProcessDrop::Refused, format!("Nothing to start in {}{missing_note}", target.label));
    }
    let hint = format!("Start {} in {}{missing_note} · the originals keep running", named(&started), target.label);
    (ProcessDrop::Start(started.iter().map(|process| process.id).collect()), hint)
}

/// One process by its name, several by their count.
fn named(processes: &[&Dragged]) -> String {
    match processes {
        [only] => only.name.clone(),
        many => format!("{} processes", many.len()),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::project::ProjectId;

    fn checkout(path: &str) -> CheckoutRef {
        CheckoutRef { project: ProjectId::new("p"), path: PathBuf::from(path) }
    }

    fn dev() -> Labelled {
        Labelled { checkout: checkout("/p"), label: "dev".to_owned() }
    }

    fn dragged(number: u64, kind: ProcessKind, at: &str) -> Dragged {
        Dragged { id: ProcessId::for_tests(number), kind, name: format!("p{number}"), checkout: Some(checkout(at)), config_in_target: true }
    }

    fn held(forced: Option<Verb>) -> Held {
        Held { forced, copy_name: "Ctrl".to_owned() }
    }

    #[test]
    fn should_move_terminals_and_leave_runs() {
        // given
        let set = [dragged(1, ProcessKind::Terminal, "/p/wt"), dragged(2, ProcessKind::ClaudeCode, "/p/wt"), dragged(3, ProcessKind::Run, "/p/wt")];

        // when
        let (drop, hint) = plan_process_drop(&set, &dev(), &held(None));

        // then
        assert_eq!(drop, ProcessDrop::Move(vec![ProcessId::for_tests(1), ProcessId::for_tests(2)]));
        assert_eq!(hint, "Move 2 processes to dev · 1 run stays (Ctrl+drop starts its copy there)");
    }

    #[test]
    fn should_name_one_moved_process() {
        // when
        let (_, hint) = plan_process_drop(&[dragged(1, ProcessKind::Terminal, "/p/wt")], &dev(), &held(None));

        // then
        assert_eq!(hint, "Move p1 to dev");
    }

    #[test]
    fn should_refuse_only_runs() {
        // when
        let (drop, hint) = plan_process_drop(&[dragged(3, ProcessKind::Run, "/p/wt")], &dev(), &held(None));

        // then
        assert_eq!((drop, hint.as_str()), (ProcessDrop::Refused, "A run stays in its worktree; Ctrl+drop starts its copy in dev"));
    }

    #[test]
    fn should_refuse_already_there() {
        // when
        let (drop, hint) = plan_process_drop(&[dragged(1, ProcessKind::Terminal, "/p")], &dev(), &held(None));

        // then
        assert_eq!((drop, hint.as_str()), (ProcessDrop::Refused, "Already in dev"));
    }

    #[test]
    fn should_start_copies_when_copy_forced() {
        // given
        let set = [dragged(1, ProcessKind::Terminal, "/p/wt"), dragged(3, ProcessKind::Run, "/p/wt")];

        // when
        let (drop, hint) = plan_process_drop(&set, &dev(), &held(Some(Verb::Copy)));

        // then
        assert_eq!(drop, ProcessDrop::Start(vec![ProcessId::for_tests(1), ProcessId::for_tests(3)]));
        assert_eq!(hint, "Start 2 processes in dev · the originals keep running");
    }

    #[test]
    fn should_name_runs_missing_in_target() {
        // given
        let missing = Dragged { config_in_target: false, ..dragged(3, ProcessKind::Run, "/p/wt") };
        let set = [dragged(1, ProcessKind::Terminal, "/p/wt"), missing];

        // when
        let (drop, hint) = plan_process_drop(&set, &dev(), &held(Some(Verb::Copy)));

        // then
        assert_eq!(drop, ProcessDrop::Start(vec![ProcessId::for_tests(1)]));
        assert_eq!(hint, "Start p1 in dev · p3: no such configuration there · the originals keep running");
    }

    #[test]
    fn should_move_with_forced_move() {
        // when
        let (drop, _) = plan_process_drop(&[dragged(1, ProcessKind::Terminal, "/p/wt")], &dev(), &held(Some(Verb::Move)));

        // then
        assert_eq!(drop, ProcessDrop::Move(vec![ProcessId::for_tests(1)]));
    }
}
