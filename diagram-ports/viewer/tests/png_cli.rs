use image::ImageFormat;
use std::{path::Path, process::Command};

fn render(source: &Path, output: &Path, reference: Option<&Path>) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_diagram-viewer"));
    command
        .arg("--render")
        .arg(source)
        .arg("--output")
        .arg(output);
    if let Some(reference) = reference {
        command.arg("--compare-png").arg(reference);
    }
    command.output().unwrap()
}

#[test]
fn should_compare_headless_dot_png_pixel_by_pixel() {
    // given
    let folder = tempfile::tempdir().unwrap();
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/plantuml/class.puml");
    let reference = folder.path().join("reference.png");
    let actual = folder.path().join("actual.png");
    let first = render(&source, &reference, None);
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    let same_path = render(&source, &reference, Some(&reference));

    // when
    let identical = render(&source, &actual, Some(&reference));
    let bytes = std::fs::read(&reference).unwrap();
    let mut changed = image::load_from_memory_with_format(&bytes, ImageFormat::Png)
        .unwrap()
        .to_rgba8();
    changed.get_pixel_mut(0, 0).0[0] ^= 1;
    changed
        .save_with_format(&reference, ImageFormat::Png)
        .unwrap();
    let different = render(&source, &actual, Some(&reference));

    // then
    assert!(
        identical.status.success(),
        "{}",
        String::from_utf8_lossy(&identical.stderr)
    );
    assert!(!same_path.status.success());
    assert!(String::from_utf8_lossy(&same_path.stderr).contains("also its comparison reference"));
    assert!(!different.status.success());
    assert!(String::from_utf8_lossy(&different.stderr).contains("PNG pixels differ: 1 of"));
}

#[test]
fn should_render_dot_state_scope_without_internal_edges() {
    // given
    let folder = tempfile::tempdir().unwrap();
    let source =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/plantuml/statehistory.puml");
    let output = folder.path().join("state.png");

    // when
    let result = render(&source, &output, None);

    // then
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(output.metadata().unwrap().len() > 100);
}

#[test]
fn should_use_bundled_dot_without_a_system_path() {
    // given
    let folder = tempfile::tempdir().unwrap();
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/plantuml/class.puml");
    let output = folder.path().join("actual.png");
    // when
    let result = Command::new(env!("CARGO_BIN_EXE_diagram-viewer"))
        .arg("--render")
        .arg(&source)
        .arg("--output")
        .arg(&output)
        .env("PATH", folder.path())
        .output()
        .unwrap();

    // then
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(output.metadata().unwrap().len() > 100);
}
