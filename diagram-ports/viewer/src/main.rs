use diagram_viewer::{
    cli::{self, Mode},
    worker,
};
fn main() {
    if let Err(error) = run() {
        eprintln!("diagram-viewer: {error}");
        std::process::exit(1);
    }
}
fn run() -> Result<(), String> {
    let cli = cli::parse(std::env::args_os().skip(1))?;
    match cli.mode {
        Mode::Worker => worker::main(),
        Mode::Help => {
            println!("{}", cli::HELP);
            Ok(())
        }
        Mode::Export {
            source,
            output,
            comparison,
        } => {
            let mut job = worker::Job::headless(worker::read_source(&source)?);
            job.language = cli.language;
            let rendered = worker::run(
                &std::env::current_exe()
                    .map_err(|error| format!("cannot locate diagram-viewer executable: {error}"))?,
                job,
            )?;
            diagram_viewer::export::write(&rendered.svg, &output, comparison)
        }
        Mode::View {
            source,
            background,
            capture,
        } => diagram_viewer::app::run(
            source,
            diagram_viewer::app::Settings {
                language: cli.language,
                background,
                capture,
            },
        ),
    }
}
