use std::{ffi::OsString, path::PathBuf};

#[derive(Debug, PartialEq, Eq)]
pub enum Mode {
    Help,
    Worker,
    Export {
        source: PathBuf,
        output: PathBuf,
        comparison: Comparison,
    },
    View {
        source: Input,
        background: bool,
        capture: Capture,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Input {
    Example(String),
    File(PathBuf),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Capture {
    Disabled,
    Png(PathBuf),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Comparison {
    Disabled,
    Png(PathBuf),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Cli {
    pub mode: Mode,
    pub language: String,
}

pub const HELP: &str = "diagram-viewer [FILE] [--language mermaid|plantuml|dot] [--background]\n  --example NAME       Show a built-in corpus example (default: flowchart)\n  --render FILE --output FILE.svg|FILE.png|FILE.json  Export without opening a window\n  --compare-png FILE.png  Compare exported PNG pixels with a reference\n  --capture FILE.png   Capture the rendered app frame and quit (also captures errors)\n  --background         Open without taking focus\n  --help               Show this help";

pub fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Cli, String> {
    let args: Vec<_> = args.into_iter().collect();
    if args == [OsString::from("--worker")] {
        return Ok(Cli {
            mode: Mode::Worker,
            language: "mermaid".into(),
        });
    }
    if args == [OsString::from("--help")] {
        return Ok(Cli {
            mode: Mode::Help,
            language: "mermaid".into(),
        });
    }
    parse_view(args)
}

fn parse_view(args: Vec<OsString>) -> Result<Cli, String> {
    let mut values = std::collections::HashMap::new();
    let mut flags = std::collections::HashSet::new();
    let mut source = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.to_str() {
            Some("--background" | "--render") => {
                if !flags.insert(arg.clone()) {
                    return Err(format!("duplicate argument {arg:?}"));
                }
            }
            Some("--output" | "--capture" | "--language" | "--example" | "--compare-png") => {
                let value = args
                    .next()
                    .filter(|value| !value.is_empty())
                    .ok_or_else(|| format!("argument {arg:?} requires a value"))?;
                if values.insert(arg.clone(), value).is_some() {
                    return Err(format!("duplicate argument {arg:?}"));
                }
            }
            Some(value) if value.starts_with('-') => {
                return Err(format!("unknown argument {arg:?}"));
            }
            _ => {
                if source.replace(PathBuf::from(arg)).is_some() {
                    return Err("only one source file is allowed".into());
                }
            }
        }
    }
    finish(values, flags, source)
}

fn finish(
    mut values: std::collections::HashMap<OsString, OsString>,
    flags: std::collections::HashSet<OsString>,
    source: Option<PathBuf>,
) -> Result<Cli, String> {
    let language = values
        .remove(&OsString::from("--language"))
        .unwrap_or_else(|| {
            match source
                .as_ref()
                .and_then(|path| path.extension())
                .and_then(|ext| ext.to_str())
                .map(str::to_ascii_lowercase)
                .as_deref()
            {
                Some("puml" | "plantuml") => "plantuml".into(),
                Some("dot") => "dot".into(),
                _ => "mermaid".into(),
            }
        })
        .into_string()
        .map_err(|value| format!("invalid language {value:?}"))?;
    if !matches!(language.as_str(), "mermaid" | "plantuml" | "dot") {
        return Err(format!(
            "unknown diagram language {language:?}; expected mermaid, plantuml or dot"
        ));
    }
    let example = values.remove(&OsString::from("--example"));
    let capture = values.remove(&OsString::from("--capture"));
    let output = values.remove(&OsString::from("--output"));
    let compare = values.remove(&OsString::from("--compare-png"));
    let background = flags.contains(&OsString::from("--background"));
    let mode = if flags.contains(&OsString::from("--render")) {
        if example.is_some() || capture.is_some() || background {
            return Err(
                "--render cannot be combined with --example, --capture or --background".into(),
            );
        }
        let output = output
            .map(PathBuf::from)
            .ok_or("--render requires --output FILE.svg, FILE.png or DOT FILE.json")?;
        let is_png = output
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("png"));
        let is_svg = output
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("svg"));
        let is_json = language == "dot"
            && output
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("json"));
        if !is_png && !is_svg && !is_json {
            return Err(format!(
                "render output {} must end in .svg, .png or DOT .json",
                output.display()
            ));
        }
        if compare.is_some() && !is_png {
            return Err("--compare-png requires a .png output".into());
        }
        Mode::Export {
            source: source.ok_or("--render requires a source file")?,
            output,
            comparison: compare.map_or(Comparison::Disabled, |path| Comparison::Png(path.into())),
        }
    } else {
        if output.is_some() || compare.is_some() {
            return Err("--output and --compare-png require --render".into());
        }
        if source.is_some() && example.is_some() {
            return Err("choose a source file or --example, not both".into());
        }
        let source = match source {
            Some(path) => Input::File(path),
            None => Input::Example(
                example
                    .unwrap_or_else(|| {
                        if language == "plantuml" {
                            "plantuml-sequence".into()
                        } else if language == "dot" {
                            "dot-basic".into()
                        } else {
                            "flowchart".into()
                        }
                    })
                    .into_string()
                    .map_err(|value| format!("invalid example name {value:?}"))?,
            ),
        };
        Mode::View {
            source,
            background,
            capture: capture.map_or(Capture::Disabled, |path| Capture::Png(path.into())),
        }
    };
    Ok(Cli { mode, language })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn arguments(args: &[&str]) -> Result<Cli, String> {
        parse(args.iter().map(OsString::from))
    }

    #[test]
    fn should_select_plantuml_for_its_file_extensions_and_default_example() {
        // given
        let cases = [
            ["--render", "input.puml", "--output", "out.svg"],
            ["--render", "input.PLANTUML", "--output", "out.svg"],
        ];
        // when
        for args in cases {
            let parsed = arguments(&args).unwrap();
            // then
            assert_eq!(parsed.language, "plantuml");
        }
        // when
        let explicit = arguments(&["--language", "mermaid", "input.puml"]).unwrap();
        let example = arguments(&["--language", "plantuml"]).unwrap();
        // then
        assert_eq!(explicit.language, "mermaid");
        assert!(
            matches!(example.mode,Mode::View{source:Input::Example(name),..} if name=="plantuml-sequence")
        );
    }

    #[test]
    fn should_select_dot_for_its_file_extension_and_explicit_language() {
        // given
        let inferred = ["--render", "graph.DOT", "--output", "graph.svg"];
        let explicit = ["--language", "dot", "graph.txt"];

        // when
        let inferred = arguments(&inferred).unwrap();
        let explicit = arguments(&explicit).unwrap();
        let missing = arguments(&["--language", "dot"]);

        // then
        assert_eq!(inferred.language, "dot");
        assert_eq!(explicit.language, "dot");
        assert!(
            matches!(missing.unwrap().mode, Mode::View { source: Input::Example(name), .. } if name == "dot-basic")
        );
    }

    #[test]
    fn should_allow_json_geometry_only_for_dot_exports() {
        // given
        let dot = ["--render", "graph.dot", "--output", "graph.json"];
        let plantuml = ["--render", "graph.puml", "--output", "graph.json"];
        // when
        let accepted = arguments(&dot);
        let rejected = arguments(&plantuml);
        // then
        assert!(matches!(accepted.unwrap().mode, Mode::Export { .. }));
        assert!(rejected.is_err());
    }

    #[test]
    fn should_open_default_example_without_a_file() {
        // given
        let args = [];
        // when
        let actual = arguments(&args).unwrap();
        // then
        assert_eq!(
            actual.mode,
            Mode::View {
                source: Input::Example("flowchart".into()),
                background: false,
                capture: Capture::Disabled
            }
        );
    }

    #[test]
    fn should_parse_explicit_svg_export() {
        // given
        let args = ["--render", "input.mmd", "--output", "result.svg"];
        // when
        let actual = arguments(&args).unwrap();
        // then
        assert_eq!(
            actual.mode,
            Mode::Export {
                source: "input.mmd".into(),
                output: "result.svg".into(),
                comparison: Comparison::Disabled,
            }
        );
    }

    #[test]
    fn should_parse_headless_png_comparison() {
        // given
        let args = [
            "--render",
            "input.puml",
            "--output",
            "actual.png",
            "--compare-png",
            "reference.png",
        ];
        // when
        let actual = arguments(&args).unwrap();
        // then
        assert_eq!(actual.language, "plantuml");
        assert_eq!(
            actual.mode,
            Mode::Export {
                source: "input.puml".into(),
                output: "actual.png".into(),
                comparison: Comparison::Png("reference.png".into()),
            }
        );
    }

    #[test]
    fn should_parse_background_capture_and_language() {
        // given
        let args = [
            "--background",
            "--capture",
            "frame.png",
            "--language",
            "plantuml",
            "input.puml",
        ];
        // when
        let actual = arguments(&args).unwrap();
        // then
        assert_eq!(actual.language, "plantuml");
        assert_eq!(
            actual.mode,
            Mode::View {
                source: Input::File("input.puml".into()),
                background: true,
                capture: Capture::Png("frame.png".into())
            }
        );
    }

    #[test]
    fn should_reject_ambiguous_or_incomplete_arguments() {
        // given
        let cases: &[&[&str]] = &[
            &["--render", "in.mmd"],
            &["--output", "out.svg"],
            &["--render", "in.puml", "--output", "out.txt"],
            &[
                "--render",
                "in.puml",
                "--output",
                "out.svg",
                "--compare-png",
                "reference.png",
            ],
            &["a", "b"],
            &["a", "--example", "pie"],
            &["--render", "a", "--output", "b", "--capture", "c"],
            &["--worker", "a"],
            &["--language", "unknown"],
            &["--capture"],
            &["--language", "mermaid", "--language", "plantuml"],
            &["--unknown"],
        ];
        // when
        let actual: Vec<_> = cases.iter().map(|args| arguments(args)).collect();
        // then
        assert!(actual.iter().all(Result::is_err));
    }
}
