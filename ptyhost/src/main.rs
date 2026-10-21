//! `ptyhost`: tmux-style sessions from the command line; see [`ptyhost::cli`].

use std::process::ExitCode;

use ptyhost::host::Launcher;

fn main() -> ExitCode {
    let args: Result<Vec<String>, _> = std::env::args_os().skip(1).map(|arg| arg.into_string()).collect();
    let Ok(args) = args else {
        eprintln!("ptyhost: arguments must be UTF-8");
        return ExitCode::from(2);
    };
    let exe = match std::env::current_exe() {
        Ok(exe) => exe,
        Err(error) => {
            eprintln!("ptyhost: cannot find its own executable: {error}");
            return ExitCode::FAILURE;
        }
    };
    ptyhost::cli::main(&args, &Launcher { exe, args: Vec::new() })
}
