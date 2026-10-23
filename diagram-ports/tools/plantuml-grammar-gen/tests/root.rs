use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

struct Fixture(PathBuf);
impl Fixture {
    fn new(word: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "plantuml-root-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("engine/src/plantuml/parser/grammar")).unwrap();
        fs::create_dir_all(root.join("tools/plantuml-grammar-gen")).unwrap();
        fs::write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"engine\"]\n",
        )
        .unwrap();
        fs::write(
            root.join("engine/Cargo.toml"),
            "[package]\nname = \"diagram-engine\"\n",
        )
        .unwrap();
        fs::write(
            root.join("tools/plantuml-grammar-gen/Cargo.toml"),
            "[package]\nname = \"plantuml-grammar-gen\"\n",
        )
        .unwrap();
        fs::write(root.join("engine/src/plantuml/parser/grammar/definition.lalrpop"), format!("grammar; extern {{ enum Token {{ \"{word}\" => Token::Ok, }} }} pub Value: () = \"{word}\" => ();\n")).unwrap();
        Self(fs::canonicalize(root).unwrap())
    }
    fn generated(&self) -> PathBuf {
        self.0.join("engine/src/plantuml/generated/definition.rs")
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn invoke(cwd: &Path, root: &Path, mode: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_plantuml-grammar-gen"))
        .current_dir(cwd)
        .args([mode, "--root"])
        .arg(root)
        .output()
        .unwrap()
}

#[test]
fn should_target_two_explicit_checkout_roots_with_one_compiled_binary() {
    // given
    let first = Fixture::new("first");
    let second = Fixture::new("second");
    // when
    let a = invoke(&second.0, &first.0, "generate");
    let first_bytes = fs::read(first.generated()).unwrap_or_default();
    let b = invoke(&first.0, &second.0, "generate");
    // then
    assert!(a.status.success(), "{}", String::from_utf8_lossy(&a.stderr));
    assert!(b.status.success(), "{}", String::from_utf8_lossy(&b.stderr));
    assert_ne!(first_bytes, fs::read(second.generated()).unwrap());
    assert_eq!(first_bytes, fs::read(first.generated()).unwrap());
    assert!(invoke(&first.0, &second.0, "check").status.success());
    assert_eq!(first_bytes, fs::read(first.generated()).unwrap());
}

#[test]
fn should_reject_missing_or_wrong_roots_without_touching_the_current_checkout() {
    // given
    let first = Fixture::new("first");
    let wrong = Fixture::new("wrong");
    fs::remove_file(wrong.0.join("engine/Cargo.toml")).unwrap();
    // when
    let invalid = invoke(&first.0, &wrong.0, "generate");
    let missing = Command::new(env!("CARGO_BIN_EXE_plantuml-grammar-gen"))
        .current_dir(&first.0)
        .arg("generate")
        .output()
        .unwrap();
    let relative = invoke(&first.0, Path::new("."), "generate");
    // then
    assert!(!invalid.status.success());
    assert!(!missing.status.success());
    assert!(!relative.status.success());
    assert!(!first.generated().exists());
    assert!(!wrong.generated().exists());
}
