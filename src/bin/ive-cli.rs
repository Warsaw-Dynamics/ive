//! `ive-cli`: IVE as a console program, for roles that print into a terminal (`--serve`); Windows needs it because
//! `ive.exe` has no console of its own. Elsewhere `ive` does both.
use std::process::ExitCode;

fn main() -> ExitCode {
    ive::main()
}
