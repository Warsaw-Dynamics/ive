//! A run's `before` steps: started one after another, the run itself once each exited 0.

use std::collections::VecDeque;

/// A `before` step: its configuration's key and name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    pub key: String,
    pub name: String,
}

/// What to do next.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Next {
    /// start this step, or wait for its run when one is live already
    Start(Step),
    /// every step exited 0: start the run itself
    Launch,
    /// a step failed: the run does not start, and says why
    Fail(String),
}

/// The steps still to run, and the one whose process the chain waits for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chain {
    steps: VecDeque<Step>,
    current: Option<Step>,
    waiting_on: Option<u64>,
}

impl Chain {
    pub fn new(steps: Vec<Step>) -> Self {
        Self { steps: steps.into(), current: None, waiting_on: None }
    }

    /// The next step, or Launch once none is left.
    pub fn next(&mut self) -> Next {
        self.waiting_on = None;
        self.current = self.steps.pop_front();
        match &self.current {
            Some(step) => Next::Start(step.clone()),
            None => Next::Launch,
        }
    }

    /// The current step runs as process `process` (`ProcessId::value`).
    pub fn wait_for(&mut self, process: u64) {
        self.waiting_on = Some(process);
    }

    /// Process `process` exited with `code`, `stopped` by IVE or not; `None` when the chain does not wait for it.
    pub fn on_exit(&mut self, process: u64, code: i32, stopped: bool) -> Option<Next> {
        if self.waiting_on != Some(process) {
            return None;
        }
        if code == 0 && !stopped {
            return Some(self.next());
        }
        let name = self.current.as_ref().map_or("", |step| step.name.as_str());
        let reason = if stopped { "stopped".to_owned() } else { format!("exit {code}") };
        self.waiting_on = None;
        Some(Next::Fail(format!("before: {name} failed ({reason})")))
    }

    /// Stop: the chain waits for nothing more, so no step's exit starts the run.
    pub fn cancel(&mut self) {
        self.waiting_on = None;
        self.steps.clear();
    }

    /// Process `process` was removed (or restarted) before it exited; `None` when the chain does not wait for it.
    pub fn on_removed(&mut self, process: u64) -> Option<Next> {
        if self.waiting_on != Some(process) {
            return None;
        }
        self.waiting_on = None;
        let name = self.current.as_ref().map_or("", |step| step.name.as_str());
        Some(Next::Fail(format!("before: {name} was removed")))
    }

    /// The name of the step the chain waits for.
    pub fn waiting_name(&self) -> Option<&str> {
        self.waiting_on.and(self.current.as_ref()).map(|step| step.name.as_str())
    }

    /// The process the chain waits for.
    pub fn waiting_on(&self) -> Option<u64> {
        self.waiting_on
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn steps() -> Vec<Step> {
        vec![Step { key: "build".to_owned(), name: "Build misc".to_owned() }, Step { key: "migrate".to_owned(), name: "Migrate".to_owned() }]
    }

    #[test]
    fn should_launch_after_all_steps_exit_zero() {
        // given
        let mut chain = Chain::new(steps());
        let first = chain.next();
        chain.wait_for(10);
        let second = chain.on_exit(10, 0, false);
        chain.wait_for(11);

        // when
        let last = chain.on_exit(11, 0, false);

        // then
        assert_eq!((first, second, last), (Next::Start(steps()[0].clone()), Some(Next::Start(steps()[1].clone())), Some(Next::Launch)));
    }

    #[test]
    fn should_fail_chain_when_a_step_fails() {
        // given
        let mut chain = Chain::new(steps());
        chain.next();
        chain.wait_for(10);

        // when
        let failed = chain.on_exit(10, 1, false);

        // then
        assert_eq!(failed, Some(Next::Fail("before: Build misc failed (exit 1)".to_owned())));
    }

    #[test]
    fn should_fail_chain_when_a_step_is_stopped() {
        // given
        let mut chain = Chain::new(steps());
        chain.next();
        chain.wait_for(10);

        // when
        let failed = chain.on_exit(10, 143, true);

        // then
        assert_eq!(failed, Some(Next::Fail("before: Build misc failed (stopped)".to_owned())));
    }

    #[test]
    fn should_wait_for_a_step_already_running() {
        // given
        let mut chain = Chain::new(steps()[..1].to_vec());
        chain.next();
        chain.wait_for(42);

        // when
        let other = chain.on_exit(7, 0, false);
        let awaited = chain.on_exit(42, 0, false);

        // then
        assert_eq!((other, awaited, chain.waiting_name()), (None, Some(Next::Launch), None));
    }

    #[test]
    fn should_not_launch_once_stopped() {
        // given
        let mut chain = Chain::new(steps()[..1].to_vec());
        chain.next();
        chain.wait_for(10);

        // when
        chain.cancel();

        // then
        assert_eq!((chain.on_exit(10, 0, false), chain.waiting_on()), (None, None));
    }

    #[test]
    fn should_fail_when_the_awaited_step_is_removed() {
        // given
        let mut chain = Chain::new(steps());
        chain.next();
        chain.wait_for(10);

        // when
        let removed = [chain.on_removed(7), chain.on_removed(10)];

        // then
        assert_eq!(removed, [None, Some(Next::Fail("before: Build misc was removed".to_owned()))]);
    }

    #[test]
    fn should_launch_at_once_without_steps() {
        // given
        let mut chain = Chain::new(Vec::new());

        // when
        let next = chain.next();

        // then
        assert_eq!(next, Next::Launch);
    }
}
