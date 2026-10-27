//! Ending a process for good on Windows: the job holding a terminal's program and everything it started.

use std::sync::Arc;

use crate::job::Job;

/// What a process's Stop ends: its terminal's job.
#[derive(Clone)]
pub struct Targets {
    pid: u32,
    job: Arc<Job>,
}

impl Targets {
    /// `None` without a program pid or a job: there is nothing to end or wait for beyond the hang-up.
    pub fn new(program: Option<u32>, job: Option<Arc<Job>>) -> Option<Self> {
        Some(Self { pid: program?, job: job? })
    }

    /// The program's pid, as the user knows it.
    pub fn pid(&self) -> u32 {
        self.pid
    }

    /// Nothing beyond the hang-up: closing the pseudoconsole already sent its programs `CTRL_CLOSE_EVENT`.
    pub fn terminate(&self) {}

    pub fn kill(&self) {
        self.job.terminate();
    }

    pub fn alive(&self) -> bool {
        self.job.is_running()
    }
}
