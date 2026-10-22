//! Which terminals were used last: Send to Terminal lists them in that order, and the hub's Alt+Enter types into
//! the first. A terminal is used when its tab is shown or Enter is pressed in it.

use std::collections::HashMap;

use crate::process::ProcessId;

/// What a process is, for the list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Place {
    /// a live terminal, Claude Code included: something to type into
    Terminal,
    /// a run, or a terminal whose shell ended
    Run,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Candidate {
    pub id: ProcessId,
    pub place: Place,
}

/// The order terminals were last used in, counted rather than timed so two uses in one second keep their order.
#[derive(Debug, Clone, Default)]
pub struct TerminalUse {
    used: HashMap<ProcessId, u64>,
    uses: u64,
}

impl TerminalUse {
    pub fn used(&mut self, id: ProcessId) {
        self.uses += 1;
        self.used.insert(id, self.uses);
    }

    /// The terminals of `candidates`, the most recently used first, then the never used ones, newest first.
    pub fn by_recency(&self, candidates: impl IntoIterator<Item = Candidate>) -> Vec<ProcessId> {
        let mut terminals: Vec<ProcessId> = candidates.into_iter().filter(|candidate| candidate.place == Place::Terminal).map(|candidate| candidate.id).collect();
        terminals.sort_by_key(|id| std::cmp::Reverse((self.used.get(id).copied(), id.value())));
        terminals
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn terminal(id: u64) -> Candidate {
        Candidate { id: ProcessId::for_tests(id), place: Place::Terminal }
    }

    fn ids(order: Vec<ProcessId>) -> Vec<u64> {
        order.into_iter().map(ProcessId::value).collect()
    }

    #[test]
    fn should_order_terminals_by_last_use() {
        // given
        let mut used = TerminalUse::default();
        used.used(ProcessId::for_tests(1));
        used.used(ProcessId::for_tests(2));
        used.used(ProcessId::for_tests(1));

        // when
        let order = used.by_recency([terminal(1), terminal(2)]);

        // then
        assert_eq!(ids(order), [1, 2]);
    }

    #[test]
    fn should_leave_runs_out_of_the_terminal_list() {
        // given
        let used = TerminalUse::default();
        let run = Candidate { id: ProcessId::for_tests(3), place: Place::Run };

        // when
        let order = used.by_recency([terminal(1), run]);

        // then
        assert_eq!(ids(order), [1]);
    }

    #[test]
    fn should_list_never_used_terminals_last_newest_first() {
        // given
        let mut used = TerminalUse::default();
        used.used(ProcessId::for_tests(1));

        // when
        let order = used.by_recency([terminal(1), terminal(2), terminal(5)]);

        // then
        assert_eq!(ids(order), [1, 5, 2]);
    }
}
