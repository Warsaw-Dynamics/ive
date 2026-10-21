//! Puts a host in a systemd scope of its own, so its program and everything that program starts are one cgroup:
//! under memory pressure systemd-oomd then ends that terminal alone, not whoever started it with all its other
//! terminals.

use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// How long systemd gets to run the job that moves the host; the program must start inside the scope.
const MOVE_TIMEOUT: Duration = Duration::from_secs(1);
const MOVE_POLL: Duration = Duration::from_millis(5);

/// Moves this process into `ptyhost-<id>.scope` of the user's systemd, as `systemd-run --user --scope` does; `Err`
/// says why it stays in its starter's cgroup.
pub fn move_into_own_scope(id: &str, cwd: &Path) -> Result<(), String> {
    let unit = unit_name(id);
    let output = Command::new("busctl")
        .args(start_args(&unit, std::process::id(), &format!("terminal {id} in {}", cwd.display())))
        .stdin(Stdio::null())
        .output()
        .map_err(|error| format!("cannot run busctl for {unit}: {error}"))?;
    if !output.status.success() {
        return Err(format!("systemd did not start {unit}: {}", String::from_utf8_lossy(&output.stderr).trim()));
    }
    wait_until_moved(&unit)
}

fn unit_name(id: &str) -> String {
    format!("ptyhost-{id}.scope")
}

/// `busctl` arguments for StartTransientUnit(name, mode, properties, aux); an emptied scope is collected even
/// after it failed, so an oomd kill leaves no unit behind.
fn start_args(unit: &str, pid: u32, description: &str) -> Vec<String> {
    let fixed = ["--user", "call", "org.freedesktop.systemd1", "/org/freedesktop/systemd1", "org.freedesktop.systemd1.Manager"];
    let call = ["StartTransientUnit", "ssa(sv)a(sa(sv))", unit, "fail", "3"];
    let properties = ["PIDs", "au", "1", &pid.to_string(), "Description", "s", description, "CollectMode", "s", "inactive-or-failed", "0"];
    fixed.iter().chain(&call).chain(&properties).map(|arg| arg.to_string()).collect()
}

/// The job runs after StartTransientUnit returns: waits until `/proc/self/cgroup` names the unit.
fn wait_until_moved(unit: &str) -> Result<(), String> {
    let deadline = Instant::now() + MOVE_TIMEOUT;
    while Instant::now() < deadline {
        if std::fs::read_to_string("/proc/self/cgroup").is_ok_and(|listing| is_in_unit(&listing, unit)) {
            return Ok(());
        }
        std::thread::sleep(MOVE_POLL);
    }
    Err(format!("systemd did not move the host into {unit} within {} ms", MOVE_TIMEOUT.as_millis()))
}

fn is_in_unit(cgroup_listing: &str, unit: &str) -> bool {
    cgroup_listing.lines().filter_map(|line| line.strip_prefix("0::")).any(|path| path.ends_with(&format!("/{unit}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_ask_systemd_for_a_collected_scope_holding_the_host() {
        // given
        let unit = unit_name("t7daeab592245");

        // when
        let args = start_args(&unit, 4242, "terminal t7daeab592245 in /repo");

        // then
        assert_eq!(
            args,
            [
                "--user", "call", "org.freedesktop.systemd1", "/org/freedesktop/systemd1", "org.freedesktop.systemd1.Manager",
                "StartTransientUnit", "ssa(sv)a(sa(sv))", "ptyhost-t7daeab592245.scope", "fail", "3",
                "PIDs", "au", "1", "4242", "Description", "s", "terminal t7daeab592245 in /repo",
                "CollectMode", "s", "inactive-or-failed", "0",
            ]
        );
    }

    #[test]
    fn should_tell_the_scope_from_the_starters_cgroup() {
        // given
        let starter = "0::/user.slice/user-1000.slice/user@1000.service/app.slice/app-ive\\x2dkb-6bb5.scope\n";
        let moved = "0::/user.slice/user-1000.slice/user@1000.service/app.slice/ptyhost-t1.scope\n";

        // when
        let found = (is_in_unit(starter, "ptyhost-t1.scope"), is_in_unit(moved, "ptyhost-t1.scope"), is_in_unit(moved, "host-t1.scope"));

        // then
        assert_eq!(found, (false, true, false));
    }
}
