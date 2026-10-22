//! Lists the files under `assets/` for `src/assets.rs` to compile in.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

fn main() {
    write_assets();
}

/// `$OUT_DIR/assets.rs`: `&[("fonts/Inter.ttf", include_bytes!(…)), …]`, sorted by path so lookups can bisect.
fn write_assets() {
    let root = Path::new(&std::env::var("CARGO_MANIFEST_DIR").expect("cargo sets CARGO_MANIFEST_DIR")).join("assets");
    let mut files = Vec::new();
    collect(&root, &mut files);
    let mut entries: Vec<(String, PathBuf)> = files.into_iter().map(|file| (asset_path(&root, &file), file)).collect();
    entries.sort();
    let mut table = String::from("&[\n");
    for (name, file) in &entries {
        writeln!(table, "    ({name:?}, include_bytes!({:?})),", file.display().to_string()).expect("writing to a String");
    }
    table.push(']');
    let out = Path::new(&std::env::var("OUT_DIR").expect("cargo sets OUT_DIR")).join("assets.rs");
    std::fs::write(&out, table).unwrap_or_else(|error| panic!("cannot write {}: {error}", out.display()));
    // a folder makes cargo watch everything under it
    println!("cargo::rerun-if-changed={}", root.display());
}

fn collect(dir: &Path, files: &mut Vec<PathBuf>) {
    let entries = std::fs::read_dir(dir).unwrap_or_else(|error| panic!("cannot list {}: {error}", dir.display()));
    for entry in entries {
        let path = entry.unwrap_or_else(|error| panic!("cannot list {}: {error}", dir.display())).path();
        if path.is_dir() {
            collect(&path, files);
        } else {
            files.push(path);
        }
    }
}

/// `/…/assets/fonts/Inter.ttf` → `fonts/Inter.ttf`, with `/` on every OS.
fn asset_path(root: &Path, file: &Path) -> String {
    let relative = file.strip_prefix(root).expect("collected under the assets folder");
    relative.components().map(|part| part.as_os_str().to_string_lossy()).collect::<Vec<_>>().join("/")
}
