//! Logs the memory of IVE's cgroup as it grows or stalls, so that when systemd-oomd kills it, the journal already
//! names what filled it. Terminals have scopes of their own (ptyhost), unless systemd was not there to make them.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const SAMPLE_INTERVAL: Duration = Duration::from_secs(10);
/// Usage of the first report, and how far usage must pass the last report for the next. Usage is anon plus swap:
/// the page cache of what terminals build is charged here too, but the kernel takes it back when needed.
const REPORT_STEP: u64 = 4 << 30;
/// Percent of the last 10 s in which some task waited for memory; oomd acts on this.
const PRESSURE_REPORT: f64 = 10.0;
const PRESSURE_REPEAT: Duration = Duration::from_secs(60);
const TOP_PROCESSES: usize = 6;
const COMMAND_CHARS: usize = 120;

/// Watches on a thread of its own; without cgroup v2 nothing is watched and stderr says why.
pub fn start() {
    let Some(cgroup) = own_cgroup() else {
        eprintln!("ive: memory not watched: no cgroup v2 in /proc/self/cgroup");
        return;
    };
    let spawned = std::thread::Builder::new().name("ive-memory".to_owned()).spawn(move || watch(&cgroup));
    if let Err(error) = spawned {
        eprintln!("ive: memory not watched: {error}");
    }
}

fn own_cgroup() -> Option<PathBuf> {
    let listing = fs::read_to_string("/proc/self/cgroup").ok()?;
    let relative = listing.lines().find_map(|line| line.strip_prefix("0::"))?;
    Some(Path::new("/sys/fs/cgroup").join(relative.trim_start_matches('/')))
}

fn watch(cgroup: &Path) {
    let mut trigger = Trigger { next_report: REPORT_STEP, pressure_quiet_until: None };
    loop {
        if let Some(sample) = Sample::read(cgroup)
            && let Some(reason) = trigger.reason(&sample, Instant::now())
        {
            report(cgroup, &sample, &reason);
        }
        std::thread::sleep(SAMPLE_INTERVAL);
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Sample {
    current: u64,
    anon: u64,
    file: u64,
    swap: u64,
    pressure: f64,
}

impl Sample {
    fn read(cgroup: &Path) -> Option<Self> {
        let read = |name: &str| fs::read_to_string(cgroup.join(name)).ok();
        Some(Sample {
            current: read("memory.current")?.trim().parse().ok()?,
            anon: read("memory.stat").as_deref().and_then(|stat| stat_field(stat, "anon")).unwrap_or(0),
            file: read("memory.stat").as_deref().and_then(|stat| stat_field(stat, "file")).unwrap_or(0),
            swap: read("memory.swap.current").and_then(|swap| swap.trim().parse().ok()).unwrap_or(0),
            pressure: read("memory.pressure").as_deref().and_then(some_avg10).unwrap_or(0.0),
        })
    }
}

/// When to report: each time usage passes a new step, and while memory stalls, once a minute.
struct Trigger {
    next_report: u64,
    pressure_quiet_until: Option<Instant>,
}

impl Trigger {
    fn reason(&mut self, sample: &Sample, now: Instant) -> Option<String> {
        let used = sample.anon + sample.swap;
        if used >= self.next_report {
            self.next_report = (used / REPORT_STEP + 1) * REPORT_STEP;
            return Some(format!("anon and swap passed {}", gib(used / REPORT_STEP * REPORT_STEP)));
        }
        let quiet = self.pressure_quiet_until.is_some_and(|until| now < until);
        if sample.pressure >= PRESSURE_REPORT && !quiet {
            self.pressure_quiet_until = Some(now + PRESSURE_REPEAT);
            return Some(format!("memory pressure {:.1}%", sample.pressure));
        }
        None
    }
}

fn report(cgroup: &Path, sample: &Sample, reason: &str) {
    eprintln!(
        "ive: memory of IVE's cgroup {} (anon {}, swap {}, file cache {}, pressure {:.1}%): {reason}; largest:",
        gib(sample.current),
        gib(sample.anon),
        gib(sample.swap),
        gib(sample.file),
        sample.pressure
    );
    for process in largest_processes(cgroup) {
        eprintln!("ive:   {:>8} {:>7} {}", process.pid, gib(process.rss), process.command);
    }
}

struct Process {
    pid: u32,
    rss: u64,
    command: String,
}

fn largest_processes(cgroup: &Path) -> Vec<Process> {
    let pids = fs::read_to_string(cgroup.join("cgroup.procs")).unwrap_or_default();
    let mut processes: Vec<Process> = pids.lines().filter_map(|pid| pid.parse().ok()).filter_map(read_process).collect();
    processes.sort_by_key(|process| std::cmp::Reverse(process.rss));
    processes.truncate(TOP_PROCESSES);
    processes
}

/// `None` for a process that ended since the cgroup listed it.
fn read_process(pid: u32) -> Option<Process> {
    let status = fs::read_to_string(format!("/proc/{pid}/status")).ok()?;
    let cmdline = fs::read(format!("/proc/{pid}/cmdline")).ok()?;
    Some(Process { pid, rss: status_rss(&status)?, command: command_line(&cmdline) })
}

fn status_rss(status: &str) -> Option<u64> {
    let kib = status.lines().find_map(|line| line.strip_prefix("VmRSS:"))?;
    Some(kib.trim().trim_end_matches("kB").trim().parse::<u64>().ok()? * 1024)
}

fn command_line(cmdline: &[u8]) -> String {
    let joined = String::from_utf8_lossy(cmdline).replace('\0', " ");
    joined.trim().chars().take(COMMAND_CHARS).collect()
}

fn stat_field(stat: &str, name: &str) -> Option<u64> {
    stat.lines().find_map(|line| line.strip_prefix(name)?.strip_prefix(' '))?.trim().parse().ok()
}

fn some_avg10(pressure: &str) -> Option<f64> {
    let some = pressure.lines().find_map(|line| line.strip_prefix("some "))?;
    some.split_whitespace().find_map(|field| field.strip_prefix("avg10="))?.parse().ok()
}

fn gib(bytes: u64) -> String {
    format!("{:.1} GiB", bytes as f64 / (1u64 << 30) as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(anon: u64, pressure: f64) -> Sample {
        Sample { current: anon, anon, file: 0, swap: 0, pressure }
    }

    #[test]
    fn should_report_once_per_step_passed() {
        // given
        let mut trigger = Trigger { next_report: REPORT_STEP, pressure_quiet_until: None };
        let now = Instant::now();

        // when
        let reasons: Vec<Option<String>> =
            [3 << 30, 5 << 30, 6 << 30, 13 << 30].iter().map(|&anon| trigger.reason(&sample(anon, 0.0), now)).collect();

        // then
        let passed = |gib: &str| Some(format!("anon and swap passed {gib} GiB"));
        assert_eq!(reasons, [None, passed("4.0"), None, passed("12.0")]);
    }

    #[test]
    fn should_report_pressure_at_most_once_a_minute() {
        // given
        let mut trigger = Trigger { next_report: u64::MAX, pressure_quiet_until: None };
        let start = Instant::now();

        // when
        let first = trigger.reason(&sample(0, 57.5), start);
        let soon = trigger.reason(&sample(0, 57.5), start + Duration::from_secs(10));
        let later = trigger.reason(&sample(0, 57.5), start + PRESSURE_REPEAT);

        // then
        assert_eq!(first, Some("memory pressure 57.5%".to_owned()));
        assert_eq!(soon, None);
        assert_eq!(later, Some("memory pressure 57.5%".to_owned()));
    }

    #[test]
    fn should_parse_cgroup_and_proc_files() {
        // given
        let pressure = "some avg10=57.48 avg60=28.16 avg300=7.34 total=65000000\nfull avg10=50.00 avg60=1.00 avg300=0.00 total=0\n";
        let stat = "anon 1073741824\nfile 42\nfile_mapped 7\n";
        let status = "Name:\tjava\nVmPeak:\t 9 kB\nVmRSS:\t  2048 kB\n";

        // when
        let parsed = (some_avg10(pressure), stat_field(stat, "anon"), stat_field(stat, "file"), status_rss(status), command_line(b"java\0-jar\0app.jar\0"));

        // then
        assert_eq!(parsed, (Some(57.48), Some(1 << 30), Some(42), Some(2 << 20), "java -jar app.jar".to_owned()));
    }
}
