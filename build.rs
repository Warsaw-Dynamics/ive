//! Lists the files under `assets/` for `src/assets.rs` to compile in; on Windows also links IVE's icon, stack size
//! and ConPTY.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

/// Windows gives the main thread 1 MiB of stack, Linux 8 MiB; Zed links with 8 MiB on Windows too.
const WINDOWS_STACK_BYTES: u32 = 8 << 20;

fn main() {
    write_assets();
    link_windows_stack();
    copy_conpty();
    embed_icon();
}

/// `desktop/icons/ive.ico` as resource `1 ICON`, which gpui loads for IVE's windows and Explorer shows for the exe.
/// Only the icon: gpui embeds its own manifest.
fn embed_icon() {
    let rc = Path::new("desktop/icons/ive.rc");
    println!("cargo::rerun-if-changed={}", rc.display());
    println!("cargo::rerun-if-changed=desktop/icons/ive.ico");
    if let Err(error) = embed_resource::compile_for(rc, ["ive", "ive-cli"], embed_resource::NONE).manifest_optional() {
        panic!("cannot embed {}: {error}", rc.display());
    }
}

/// Puts `vendor/conpty`'s `conpty.dll` and `OpenConsole.exe` next to the built exe, where alacritty's DLL search
/// finds them first: Windows 10's own console host swallows `OSC 7`, `OSC 9;4` and bracketed paste. A copy already in
/// place is kept, as a running IVE holds it.
fn copy_conpty() {
    let os = std::env::var("CARGO_CFG_TARGET_OS");
    let arch = std::env::var("CARGO_CFG_TARGET_ARCH");
    if os.as_deref() != Ok("windows") || arch.as_deref() != Ok("x86_64") {
        return;
    }
    let source = Path::new("vendor/conpty/x64");
    println!("cargo::rerun-if-changed={}", source.display());
    // OUT_DIR is <profile dir>/build/<crate>-<hash>/out
    let out = PathBuf::from(std::env::var("OUT_DIR").expect("cargo sets OUT_DIR"));
    let Some(profile_dir) = out.ancestors().nth(3) else { return };
    for name in ["conpty.dll", "OpenConsole.exe"] {
        let (from, to) = (source.join(name), profile_dir.join(name));
        let bytes = std::fs::read(&from).unwrap_or_else(|error| panic!("cannot read {}: {error}", from.display()));
        if std::fs::read(&to).is_ok_and(|current| current == bytes) {
            continue;
        }
        if let Err(error) = std::fs::write(&to, &bytes) {
            println!("cargo::warning=cannot copy {} to {}: {error}", from.display(), to.display());
        }
    }
}

fn link_windows_stack() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let flag = match std::env::var("CARGO_CFG_TARGET_ENV").as_deref() {
        Ok("msvc") => format!("/STACK:{WINDOWS_STACK_BYTES}"),
        _ => format!("-Wl,--stack,{WINDOWS_STACK_BYTES}"),
    };
    println!("cargo::rustc-link-arg-bins={flag}");
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
