//! tmux-style sessions from the command line, for the `ptyhost` binary and programs embedding it.
//!
//! ```text
//! ptyhost new [--id ID] [--cwd DIR] [--attach] [-- COMMAND...]   start a session (default: $SHELL -l)
//! ptyhost attach ID                                              attach this terminal; Ctrl-] detaches
//! ptyhost ls                                                     list live sessions
//! ptyhost kill ID                                                hang up the session's program, kill it after 2 s
//! ```

use std::io;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::client::{self, Client, Ending};
use crate::host::{self, HostConfig, Launcher};
use crate::protocol::Size;
use crate::registry::Registry;

/// The host's hang-up grace plus time to clean up.
const KILL_WAIT: Duration = Duration::from_secs(4);

/// Runs one verb (`new | attach ID | ls | kill ID | serve …`); `launcher` starts hosts for `new`.
pub fn main(args: &[String], launcher: &Launcher) -> ExitCode {
    match run(args, launcher) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("ptyhost: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: &[String], launcher: &Launcher) -> io::Result<ExitCode> {
    let registry = Registry::from_env();
    match args.split_first() {
        Some((verb, rest)) if verb == "new" => new(&registry, launcher, Options::parse(rest)?),
        Some((verb, [id])) if verb == "attach" => attach(&registry, id),
        Some((verb, [])) if verb == "ls" => list(&registry),
        Some((verb, [id])) if verb == "kill" => kill(&registry, id),
        Some((verb, rest)) if verb == "serve" => serve(&registry, Options::parse(rest)?),
        _ => Err(io::Error::new(io::ErrorKind::InvalidInput, format!("usage: new | attach ID | ls | kill ID, got {args:?}"))),
    }
}

fn new(registry: &Registry, launcher: &Launcher, options: Options) -> io::Result<ExitCode> {
    let attach_after = options.attach;
    let config = options.into_config(generated_id())?;
    host::start(launcher, registry, &config)?;
    if attach_after {
        return attach(registry, &config.id);
    }
    println!("{}", config.id);
    Ok(ExitCode::SUCCESS)
}

fn attach(registry: &Registry, id: &str) -> io::Result<ExitCode> {
    let message = match client::attach_terminal(registry, id)? {
        Ending::Detached => format!("[detached from {id}, still running]"),
        Ending::Exited(code) => format!("[{id} exited with {code}]"),
        Ending::Lost => format!("[lost connection to {id}]"),
    };
    eprintln!("\r\n{message}");
    Ok(ExitCode::SUCCESS)
}

fn list(registry: &Registry) -> io::Result<ExitCode> {
    for session in registry.sessions()? {
        println!("{}\t{}\t{}\t{}", session.id, session.program_pid, session.cwd.display(), session.command.join(" "));
    }
    Ok(ExitCode::SUCCESS)
}

/// Hangs up the session's program as closing its terminal would; the host kills it if it outlives the grace.
fn kill(registry: &Registry, id: &str) -> io::Result<ExitCode> {
    let mut client = Client::connect(registry, id)?;
    client.hang_up()?;
    let deadline = Instant::now() + KILL_WAIT;
    while Instant::now() < deadline {
        if registry.session(id)?.is_none() {
            return Ok(ExitCode::SUCCESS);
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    Err(io::Error::new(io::ErrorKind::TimedOut, format!("session {id} still runs {KILL_WAIT:?} after the hang-up")))
}

fn serve(registry: &Registry, options: Options) -> io::Result<ExitCode> {
    let config = options.into_config(String::new())?;
    let code = host::serve(registry, &config)?;
    Ok(ExitCode::from(code.clamp(0, 255) as u8))
}

fn generated_id() -> String {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |elapsed| elapsed.subsec_nanos());
    format!("s{:x}{:x}", std::process::id(), nanos)
}

#[derive(Default)]
struct Options {
    id: Option<String>,
    cwd: Option<PathBuf>,
    size: Option<Size>,
    attach: bool,
    env: Vec<(String, String)>,
    labels: Vec<(String, String)>,
    command: Vec<String>,
}

impl Options {
    fn parse(args: &[String]) -> io::Result<Options> {
        let mut options = Options::default();
        let mut rest = args.iter();
        while let Some(arg) = rest.next() {
            let mut value = || rest.next().cloned().ok_or_else(|| invalid(format!("{arg} needs a value")));
            match arg.as_str() {
                "--id" => options.id = Some(value()?),
                "--cwd" => options.cwd = Some(PathBuf::from(value()?)),
                "--size" => options.size = Some(Size::parse(&value()?)?),
                "--attach" => options.attach = true,
                "--env" => options.env.push(key_value(&value()?)?),
                "--label" => options.labels.push(key_value(&value()?)?),
                "--" => {
                    options.command = rest.cloned().collect();
                    break;
                }
                other => return Err(invalid(format!("unknown option {other:?}"))),
            }
        }
        Ok(options)
    }

    fn into_config(self, default_id: String) -> io::Result<HostConfig> {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into());
        let command = if self.command.is_empty() { vec![shell, "-l".into()] } else { self.command };
        Ok(HostConfig {
            id: self.id.unwrap_or(default_id),
            command,
            cwd: match self.cwd {
                Some(cwd) => cwd,
                None => std::env::current_dir()?,
            },
            size: self.size.unwrap_or_else(client::terminal_size),
            env: self.env,
            labels: self.labels,
        })
    }
}

fn key_value(text: &str) -> io::Result<(String, String)> {
    text.split_once('=')
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .ok_or_else(|| invalid(format!("expected KEY=VALUE, got {text:?}")))
}

fn invalid(message: String) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}
