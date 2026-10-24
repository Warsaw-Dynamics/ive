use std::{env, error::Error, fs, path::Path};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() {
    if let Err(error) = run() {
        eprintln!("DOT grammar generation failed: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args: Vec<_> = env::args().skip(1).collect();
    let [mode, flag, root] = args.as_slice() else {
        return Err("usage: dot-grammar-gen generate|check --root ABSOLUTE_DIAGRAM_PORTS_ROOT".into());
    };
    if flag != "--root" || !matches!(mode.as_str(), "generate" | "check") {
        return Err(format!("invalid DOT grammar generator arguments: {args:?}").into());
    }
    let root = Path::new(root);
    if !root.is_absolute() || fs::canonicalize(root)? != root {
        return Err(format!("DOT grammar root is not absolute and canonical: {root:?}").into());
    }
    let source = root.join("engine/src/dot/grammar.lalrpop");
    let destination = root.join("engine/src/dot/generated.rs");
    let workspace = env::temp_dir().join(format!("dot-grammar-gen-{}", std::process::id()));
    if workspace.exists() { fs::remove_dir_all(&workspace)?; }
    let input = workspace.join("input");
    let output = workspace.join("output");
    fs::create_dir_all(&input)?;
    fs::create_dir_all(&output)?;
    fs::copy(&source, input.join("grammar.lalrpop"))?;
    let generated = generate(&input, &output)?;
    if mode == "check" {
        if fs::read(&destination)? != generated {
            return Err(format!("generated DOT parser differs from {destination:?}").into());
        }
    } else {
        fs::write(&destination, &generated)?;
    }
    fs::remove_dir_all(workspace)?;
    println!("DOT parser {mode} passed: {} bytes", generated.len());
    Ok(())
}

fn generate(input: &Path, output: &Path) -> Result<Vec<u8>> {
    lalrpop::Configuration::new()
        .set_in_dir(input)
        .set_out_dir(output)
        .force_build(true)
        .emit_comments(false)
        .emit_whitespace(true)
        .emit_report(false)
        .emit_rerun_directives(false)
        .log_quiet()
        .process()?;
    let generated = output.join("grammar.rs");
    let bytes = fs::read(&generated)?;
    if bytes.len() > 4 * 1024 * 1024 || !bytes.starts_with(b"// auto-generated: \"lalrpop 0.23.2\"\n") {
        return Err(format!("DOT parser output is invalid or too large: {generated:?}").into());
    }
    Ok(bytes)
}
