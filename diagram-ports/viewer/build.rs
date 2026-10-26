use sha2::{Digest, Sha256};
use std::{env, fs, path::PathBuf, process::Command};

const SOURCE_SHA256: &str = "0f661718f3e5268dc3bad0fb53ab646605d26db70f4080799aff1dc2f61783c3";

struct Paths {
    archive: PathBuf,
    source: PathBuf,
    build: PathBuf,
    helper: PathBuf,
}

fn main() {
    let target = env::var("TARGET").expect("Cargo target is missing");
    let host = env::var("HOST").expect("Cargo host is missing");
    assert_eq!(
        target, host,
        "bundled Graphviz needs a native target toolchain: target {target}, host {host}"
    );
    assert!(
        target.contains("linux") || target.contains("darwin"),
        "bundled Graphviz is supported on Linux and macOS: {target}"
    );
    let out = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo OUT_DIR is missing"));
    let paths = Paths {
        archive: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../vendor/graphviz/graphviz-16.1.0.tar.xz"),
        source: out.join("graphviz-16.1.0"),
        build: out.join("graphviz-build"),
        helper: out.join("ive-dot-helper"),
    };
    println!("cargo:rerun-if-changed={}", paths.archive.display());
    verify_archive(&paths);
    extract_source(&paths);
    configure(&paths);
    build(&paths);
    fs::copy(paths.build.join("cmd/dot/dot_static"), &paths.helper).unwrap_or_else(|error| {
        panic!(
            "cannot copy bundled DOT helper {}: {error}",
            paths.helper.display()
        )
    });
}

fn verify_archive(paths: &Paths) {
    let bytes = fs::read(&paths.archive).unwrap_or_else(|error| {
        panic!(
            "cannot read Graphviz source {}: {error}",
            paths.archive.display()
        )
    });
    let hash = format!("{:x}", Sha256::digest(bytes));
    assert_eq!(
        hash,
        SOURCE_SHA256,
        "Graphviz source archive {} has unexpected SHA-256 {hash}",
        paths.archive.display()
    );
}

fn extract_source(paths: &Paths) {
    let parent = paths
        .source
        .parent()
        .expect("Graphviz source has no parent");
    run(
        Command::new("tar")
            .arg("-xJf")
            .arg(&paths.archive)
            .arg("-C")
            .arg(parent),
        "extract Graphviz source",
    );
    fs::create_dir_all(&paths.build).unwrap_or_else(|error| {
        panic!(
            "cannot create Graphviz build directory {}: {error}",
            paths.build.display()
        )
    });
}

fn configure(paths: &Paths) {
    let mut command = Command::new(paths.source.join("configure"));
    command
        .current_dir(&paths.build)
        .env("CFLAGS", "-O2")
        .env("CXXFLAGS", "-O2");
    command.args([
        "--disable-shared",
        "--enable-static",
        "--disable-ltdl",
        "--disable-swig",
        "--disable-tcl",
        "--disable-man-pdfs",
        "--without-pangocairo",
        "--without-freetype2",
        "--without-expat",
        "--without-libgd",
        "--without-gts",
        "--without-webp",
        "--without-gtk",
        "--without-x",
    ]);
    run(&mut command, "configure bundled Graphviz");
}

fn build(paths: &Paths) {
    let jobs = env::var("NUM_JOBS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(2)
        .clamp(1, 4);
    let mut command = Command::new("make");
    command.current_dir(&paths.build).arg(format!("-j{jobs}"));
    run(&mut command, "compile bundled Graphviz");
}

fn run(command: &mut Command, action: &str) {
    let output = command
        .output()
        .unwrap_or_else(|error| panic!("cannot {action}: {error}"));
    if output.status.success() {
        return;
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    panic!(
        "cannot {action}: {}\n{}\n{}",
        output.status,
        tail(&stdout),
        tail(&stderr)
    );
}

fn tail(value: &str) -> String {
    value
        .chars()
        .rev()
        .take(8_000)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect()
}
