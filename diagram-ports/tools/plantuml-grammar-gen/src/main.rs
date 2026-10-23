use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    error::Error,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
const GRAMMARS: &str = "engine/src/plantuml/parser/grammar";
const GENERATED: &str = "engine/src/plantuml/generated";

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    let result = match arguments.as_slice() {
        [mode] => run(root, mode),
        _ => Err("usage: plantuml-grammar-gen generate|check".into()),
    };
    if let Err(error) = result {
        eprintln!("PlantUML generation failed: {error}");
        std::process::exit(1);
    }
}

fn settings() -> Value {
    json!({"crate": "lalrpop", "version": "0.23.2", "defaultFeatures": false,
        "forceBuild": true, "emitComments": false, "emitWhitespace": true,
        "emitReport": false, "emitRerunDirectives": false, "features": []})
}

fn run(root: &Path, mode: &str) -> Result<()> {
    match mode {
        "generate" | "check" => {}
        _ => {
            return Err(
                format!("unknown generation command {mode:?}; expected generate|check").into(),
            );
        }
    }
    let inputs = list_inputs(root)?;
    check_conflicts(root, &inputs)?;
    let manifest = root.join(GENERATED).join("manifest.json");
    if manifest.exists() {
        let previous: Value = serde_json::from_slice(&read_bounded(&manifest, 1048576)?)?;
        if previous["generator"] != settings() {
            return Err("generator version/options mismatch".into());
        }
    }
    let mirror = Mirror::new()?;
    let inventory = regenerate(root, &mirror.0, &inputs)?;
    if mode == "check" {
        return check(root, &mirror.0, &inputs, &inventory);
    }
    write(root, &mirror.0, &inputs, &inventory)
}

fn list_inputs(root: &Path) -> Result<Vec<String>> {
    let mut inputs = Vec::new();
    for entry in fs::read_dir(root.join(GRAMMARS))? {
        let entry = entry?;
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| "grammar filename is not UTF-8")?;
        let stem = name
            .strip_suffix(".lalrpop")
            .ok_or("unexpected file in grammar directory")?;
        validate_stem(stem)?;
        match entry.file_type()?.is_file() {
            true => {}
            false => return Err(format!("grammar entry is not a regular file: {name:?}").into()),
        }
        if inputs.len() == 64 {
            return Err("grammar count exceeds limit 64".into());
        }
        inputs.push(name);
    }
    inputs.sort();
    if inputs.is_empty() {
        return Err("no PlantUML grammars found".into());
    }
    Ok(inputs)
}

fn output_name(input: &str) -> String {
    input.replace(".lalrpop", ".rs")
}

fn validate_stem(stem: &str) -> Result<()> {
    if (1..=64).contains(&stem.len())
        && stem
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte == b'_')
    {
        return Ok(());
    }
    Err(format!(
        "invalid grammar filename stem {stem:?}; expected 1..64 lowercase ASCII letters/underscores"
    )
    .into())
}

fn check_conflicts(root: &Path, inputs: &[String]) -> Result<()> {
    let directory = root.join(GENERATED);
    let entries = match fs::read_dir(&directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.into()),
    };
    let expected: BTreeSet<_> = inputs
        .iter()
        .map(|name| output_name(name))
        .chain(["manifest.json".to_owned()])
        .collect();
    for entry in entries {
        let entry = entry?;
        if entry.file_type()?.is_file()
            && expected.contains(&entry.file_name().to_string_lossy().into_owned())
        {
            continue;
        }
        return Err(format!("generated output conflict: {:?}", entry.file_name()).into());
    }
    Ok(())
}

fn read_bounded(path: &Path, maximum: u64) -> Result<Vec<u8>> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.is_file() && metadata.len() <= maximum {
        return Ok(fs::read(path)?);
    }
    Err(format!(
        "generation input/output {:?} exceeds regular-file bound {maximum}",
        path
    )
    .into())
}

fn regenerate(root: &Path, mirror: &Path, inputs: &[String]) -> Result<Value> {
    let input_directory = mirror.join(GRAMMARS);
    fs::create_dir_all(&input_directory)?;
    let mut records = Vec::new();
    for name in inputs {
        let bytes = read_bounded(&root.join(GRAMMARS).join(name), 262144)?;
        let input = input_directory.join(name);
        fs::write(&input, &bytes)?;
    }
    lalrpop::Configuration::new()
        .set_in_dir(&input_directory)
        .set_out_dir(mirror.join(GENERATED))
        .force_build(true)
        .emit_comments(false)
        .emit_whitespace(true)
        .set_features(std::iter::empty::<String>())
        .emit_report(false)
        .emit_rerun_directives(false)
        .log_quiet()
        .process()?;
    for name in inputs {
        let bytes = read_bounded(&input_directory.join(name), 262144)?;
        let output = output_name(name);
        let generated = read_bounded(&mirror.join(GENERATED).join(&output), 4194304)?;
        match generated.strip_prefix(b"// auto-generated: \"lalrpop 0.23.2\"\n") {
            Some(_) => {}
            None => return Err(format!("actual generator version mismatch in {output:?}").into()),
        }
        records.push(
            json!({"grammar": format!("{GRAMMARS}/{name}"), "grammarSha256": digest(&bytes),
            "generated": format!("{GENERATED}/{output}"), "generatedSha256": digest(&generated)}),
        );
    }
    Ok(json!({"schema": 1, "generator": settings(), "grammars": records}))
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn check(root: &Path, mirror: &Path, inputs: &[String], inventory: &Value) -> Result<()> {
    let manifest_path = root.join(GENERATED).join("manifest.json");
    let manifest_bytes = read_bounded(&manifest_path, 1048576)
        .map_err(|error| format!("missing generated manifest: {error}"))?;
    let actual: Value = serde_json::from_slice(&manifest_bytes)?;
    if actual["generator"] != settings() {
        return Err("generator version/options mismatch".into());
    }
    for name in inputs {
        let output = output_name(name);
        let actual = read_bounded(&root.join(GENERATED).join(&output), 4194304)
            .map_err(|error| format!("missing generated output {output:?}: {error}"))?;
        if actual != read_bounded(&mirror.join(GENERATED).join(&output), 4194304)? {
            return Err(format!("generated parser drift: {output:?}").into());
        }
    }
    if actual != *inventory
        || manifest_bytes != format!("{}\n", serde_json::to_string_pretty(inventory)?).as_bytes()
    {
        return Err("generated manifest drift".into());
    }
    println!(
        "Verified {} reproducible PlantUML generated parser(s); check wrote no tracked outputs.",
        inputs.len()
    );
    Ok(())
}

fn write(root: &Path, mirror: &Path, inputs: &[String], inventory: &Value) -> Result<()> {
    fs::create_dir_all(root.join(GENERATED))?;
    for name in inputs {
        let output = output_name(name);
        fs::copy(
            mirror.join(GENERATED).join(&output),
            root.join(GENERATED).join(&output),
        )?;
    }
    fs::write(
        root.join(GENERATED).join("manifest.json"),
        format!("{}\n", serde_json::to_string_pretty(inventory)?),
    )?;
    println!(
        "Generated {} PlantUML parser(s) with LALRPOP 0.23.2.",
        inputs.len()
    );
    Ok(())
}

struct Mirror(PathBuf);
impl Mirror {
    fn new() -> Result<Self> {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        for _ in 0..16 {
            let path = std::env::temp_dir().join(format!(
                "plantuml-grammar-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self(path)),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error.into()),
            }
        }
        Err("cannot create bounded fresh generation mirror after 16 collisions".into())
    }
}
impl Drop for Mirror {
    fn drop(&mut self) {
        if let Err(error) = fs::remove_dir_all(&self.0) {
            eprintln!("cannot remove generation mirror {:?}: {error}", self.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicUsize, Ordering},
    };

    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            static NEXT: AtomicUsize = AtomicUsize::new(0);
            let root = std::env::temp_dir().join(format!(
                "plantuml-gen-test-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(root.join("engine/src/plantuml/parser/grammar")).unwrap();
            fs::write(root.join("engine/src/plantuml/parser/grammar/definition.lalrpop"), "grammar; extern { enum Token { \"ok\" => Token::Ok, } } pub Value: () = \"ok\" => ();\n").unwrap();
            Self(root)
        }
        fn output(&self) -> PathBuf {
            self.0.join("engine/src/plantuml/generated")
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }

    #[test]
    fn should_generate_reproducibly_and_check_without_writing() {
        // given
        let fixture = Fixture::new();
        // when
        run(&fixture.0, "generate").unwrap();
        let before = fs::read(fixture.output().join("definition.rs")).unwrap();
        let metadata = fs::metadata(fixture.output().join("definition.rs")).unwrap();
        run(&fixture.0, "check").unwrap();
        // then
        assert_eq!(
            fs::read(fixture.output().join("definition.rs")).unwrap(),
            before
        );
        assert_eq!(
            fs::metadata(fixture.output().join("definition.rs"))
                .unwrap()
                .modified()
                .unwrap(),
            metadata.modified().unwrap()
        );
        run(&fixture.0, "generate").unwrap();
        assert_eq!(
            fs::read(fixture.output().join("definition.rs")).unwrap(),
            before
        );
    }

    #[test]
    fn should_reject_drift_missing_outputs_and_version_mismatch_read_only() {
        // given
        let fixture = Fixture::new();
        run(&fixture.0, "generate").unwrap();
        let output = fixture.output().join("definition.rs");
        // when
        fs::write(&output, "drift").unwrap();
        let error = run(&fixture.0, "check").unwrap_err();
        // then
        assert!(error.to_string().contains("drift"), "{error}");
        assert_eq!(fs::read_to_string(&output).unwrap(), "drift");
        fs::remove_file(&output).unwrap();
        assert!(
            run(&fixture.0, "check")
                .unwrap_err()
                .to_string()
                .contains("missing")
        );
        run(&fixture.0, "generate").unwrap();
        let manifest = fixture.output().join("manifest.json");
        let changed = fs::read_to_string(&manifest)
            .unwrap()
            .replace("0.23.2", "0.0.0");
        fs::write(&manifest, &changed).unwrap();
        assert!(
            run(&fixture.0, "check")
                .unwrap_err()
                .to_string()
                .contains("version/options")
        );
        assert_eq!(fs::read_to_string(manifest).unwrap(), changed);
    }

    #[test]
    fn should_reject_unknown_output_conflicts() {
        // given
        let fixture = Fixture::new();
        fs::create_dir(fixture.output()).unwrap();
        fs::write(
            fixture.output().join("unknown.rs"),
            "owned by another generator",
        )
        .unwrap();
        // when
        let error = run(&fixture.0, "generate").unwrap_err();
        // then
        assert!(error.to_string().contains("conflict"), "{error}");
        assert_eq!(
            fs::read_to_string(fixture.output().join("unknown.rs")).unwrap(),
            "owned by another generator"
        );
    }
}
