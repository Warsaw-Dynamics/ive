//! A Windows job object holding a terminal's program and everything it starts, so Kill reaches programs that left
//! the console, as a process group's SIGKILL does on Unix. A program may still leave it on purpose, starting a child
//! with `CREATE_BREAKAWAY_FROM_JOB`, as `setsid` leaves a session: so does the new IVE a restart starts.

use std::io;

use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
use windows_sys::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT, JOB_OBJECT_LIMIT_BREAKAWAY_OK, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    JOBOBJECT_BASIC_ACCOUNTING_INFORMATION, JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectBasicAccountingInformation,
    JobObjectExtendedLimitInformation, OpenJobObjectW, QueryInformationJobObject, SetInformationJobObject, TerminateJobObject,
};
use windows_sys::Win32::System::SystemServices::{JOB_OBJECT_QUERY, JOB_OBJECT_TERMINATE};

/// What a program ended by [`Job::terminate`] exits with: the exit code of a Ctrl+C.
pub const TERMINATED: u32 = 0xC000_013A;

pub struct Job(HANDLE);

// SAFETY: a job handle may be used from any thread
unsafe impl Send for Job {}
// SAFETY: as above; every call on it is thread-safe
unsafe impl Sync for Job {}

impl Job {
    /// A new job holding `process`; what it starts from now on lands in the job too.
    // a HANDLE is never dereferenced here: Windows checks it and fails the call when it is not an open process
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn holding(process: HANDLE) -> io::Result<Self> {
        Self::named_holding(None, process)
    }

    /// As [`holding`](Self::holding), under `name` when given, so another process can [`open`](Self::open) it.
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn named_holding(name: Option<&str>, process: HANDLE) -> io::Result<Self> {
        let name = name.map(wide);
        // SAFETY: a NUL-terminated name or none, and no security attributes
        let handle = unsafe { CreateJobObjectW(std::ptr::null(), name.as_ref().map_or(std::ptr::null(), |name| name.as_ptr())) };
        if handle.is_null() {
            return Err(io::Error::last_os_error());
        }
        let job = Self(handle);
        job.set_limits(JOB_OBJECT_LIMIT_BREAKAWAY_OK)?;
        // SAFETY: both handles are open
        if unsafe { AssignProcessToJobObject(job.0, process) } == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(job)
    }

    /// The job another process created under `name`, to end its processes or ask whether they run.
    pub fn open(name: &str) -> io::Result<Self> {
        let name = wide(name);
        // SAFETY: a NUL-terminated name
        let handle = unsafe { OpenJobObjectW(JOB_OBJECT_TERMINATE | JOB_OBJECT_QUERY, 0, name.as_ptr()) };
        if handle.is_null() {
            return Err(io::Error::last_os_error());
        }
        Ok(Self(handle))
    }

    /// The name a host gives the job of its program: by the host's pid, which no other live host has.
    pub fn host_job_name(host_pid: u32) -> String {
        format!("Local\\ptyhost-{host_pid}")
    }

    /// Ends the job's processes once its last handle closes, so they cannot outlive a holder that crashed.
    pub fn kill_on_close(&self) -> io::Result<()> {
        self.set_limits(JOB_OBJECT_LIMIT_BREAKAWAY_OK | JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE)
    }

    /// Replaces the job's limits with `flags`.
    fn set_limits(&self, flags: JOB_OBJECT_LIMIT) -> io::Result<()> {
        // SAFETY: plain data, for which all zeros means no limits
        let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
        limits.BasicLimitInformation.LimitFlags = flags;
        let size = std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32;
        // SAFETY: the handle is open and `limits` is `size` bytes of a zeroed, valid struct
        if unsafe { SetInformationJobObject(self.0, JobObjectExtendedLimitInformation, (&raw const limits).cast(), size) } == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    /// Ends every process in the job at once.
    pub fn terminate(&self) {
        // SAFETY: the handle is open; an empty job makes it succeed with nothing to do
        unsafe { TerminateJobObject(self.0, TERMINATED) };
    }

    /// Whether a process in the job still runs.
    pub fn is_running(&self) -> bool {
        self.active_processes() > 0
    }

    fn active_processes(&self) -> u32 {
        let mut info = std::mem::MaybeUninit::<JOBOBJECT_BASIC_ACCOUNTING_INFORMATION>::zeroed();
        let size = std::mem::size_of::<JOBOBJECT_BASIC_ACCOUNTING_INFORMATION>() as u32;
        // SAFETY: the call writes at most `size` bytes into `info`, which is that large
        let read = unsafe { QueryInformationJobObject(self.0, JobObjectBasicAccountingInformation, info.as_mut_ptr().cast(), size, std::ptr::null_mut()) };
        if read == 0 {
            return 0;
        }
        // SAFETY: zeroed is a valid value, and the successful read filled it
        unsafe { info.assume_init() }.ActiveProcesses
    }
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain([0]).collect()
}

impl Drop for Job {
    fn drop(&mut self) {
        // SAFETY: the handle is open and owned; without `kill_on_close` closing it leaves the processes running
        unsafe { CloseHandle(self.0) };
    }
}

#[cfg(test)]
mod tests {
    use std::os::windows::io::AsRawHandle;
    use std::process::{Child, Command, Stdio};
    use std::time::{Duration, Instant};

    use super::*;

    fn pinging() -> Child {
        Command::new("cmd").args(["/D", "/C", "ping -n 30 127.0.0.1"]).stdout(Stdio::null()).spawn().unwrap()
    }

    fn wait_for(condition: impl Fn() -> bool) {
        let deadline = Instant::now() + Duration::from_secs(10);
        while !condition() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    #[test]
    fn should_end_a_program_and_what_it_started() {
        // given
        let mut shell = pinging();
        let job = Job::holding(shell.as_raw_handle()).unwrap();
        wait_for(|| job.active_processes() >= 2);
        let running = job.is_running();

        // when
        job.terminate();

        // then
        assert_eq!(shell.wait().unwrap().code(), Some(TERMINATED as i32));
        wait_for(|| !job.is_running());
        assert_eq!((running, job.is_running()), (true, false));
    }

    #[test]
    fn should_end_a_named_job_opened_by_name() {
        // given
        let mut shell = pinging();
        let name = format!("Local\\ptyhost-test-{}", shell.id());
        let _created = Job::named_holding(Some(&name), shell.as_raw_handle()).unwrap();

        // when
        Job::open(&name).unwrap().terminate();

        // then
        assert_eq!(shell.wait().unwrap().code(), Some(TERMINATED as i32));
    }

    #[test]
    fn should_end_its_programs_when_closed_after_kill_on_close() {
        // given
        let mut shell = pinging();
        let job = Job::holding(shell.as_raw_handle()).unwrap();
        job.kill_on_close().unwrap();

        // when
        drop(job);

        // then
        let deadline = Instant::now() + Duration::from_secs(5);
        while shell.try_wait().unwrap().is_none() {
            assert!(Instant::now() < deadline, "the program outlived its job");
            std::thread::sleep(Duration::from_millis(20));
        }
    }
}
