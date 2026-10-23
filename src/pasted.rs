//! What a drop or an image paste types into a terminal: file paths, quoted for the shell. A pasted image is first
//! saved to a file of its own, which a program like Claude Code attaches when it reads the path.

use std::io::{self, Write as _};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use gpui::Image;

use crate::clipboard::Contents;
use crate::profile;
use crate::script::keymap::PasteKind;

/// Saved images older than this go at the next start; a session can still refer to one a few days later.
const KEPT_FOR: Duration = Duration::from_secs(7 * 24 * 60 * 60);

/// `paths` as shell words separated by spaces, e.g. `/tmp/a.png '/tmp/my notes.md'`.
pub fn shell_words(paths: &[PathBuf]) -> String {
    paths.iter().map(|path| shell_word(path)).collect::<Vec<_>>().join(" ")
}

/// `path` as is when the shell reads it as one plain word, else in single quotes; a path holding control characters
/// or bytes that are not UTF-8 in `$'…'` with those as `\xNN`, so that typed into a terminal they cannot act as keys
/// (Ctrl+U, Enter) and name the file as it is.
#[cfg(unix)]
pub fn shell_word(path: &Path) -> String {
    use std::os::unix::ffi::OsStrExt as _;
    let bytes = path.as_os_str().as_bytes();
    if std::str::from_utf8(bytes).map_or(true, |text| text.chars().any(char::is_control)) {
        return ansi_c_quoted(bytes);
    }
    let text = path.to_string_lossy();
    if !text.is_empty() && text.chars().all(|c| c.is_ascii_alphanumeric() || PLAIN.contains(c)) {
        return text.into_owned();
    }
    single_quoted(&text)
}

/// `path` as is when PowerShell reads it as one plain word, else in single quotes; Windows file names hold no
/// control characters that could act as keys.
#[cfg(windows)]
pub fn shell_word(path: &Path) -> String {
    let text = path.to_string_lossy();
    if !text.is_empty() && text.chars().all(|c| c.is_ascii_alphanumeric() || PLAIN.contains(c)) {
        return text.into_owned();
    }
    single_quoted(&text)
}

/// Besides letters and digits, what the shell reads as part of a plain word. No `~`: at the start of a word the
/// shell would expand it.
#[cfg(unix)]
const PLAIN: &str = "/._-+,:@%=";
/// PowerShell, the terminals' shell on Windows, splits words at `,` and splats an `@` at the start.
#[cfg(windows)]
const PLAIN: &str = r"\/._-:";

/// `text` in single quotes for a POSIX shell: a quote in it becomes `'\''`.
#[cfg(unix)]
pub fn single_quoted(text: &str) -> String {
    format!("'{}'", text.replace('\'', r"'\''"))
}

/// `bytes` in bash/zsh/dash's `$'…'`: printable ASCII as is (`'` and `\` escaped), everything else as `\xNN`, UTF-8
/// text bytes included only when it is not text.
#[cfg(unix)]
fn ansi_c_quoted(bytes: &[u8]) -> String {
    let text = std::str::from_utf8(bytes).ok();
    let mut quoted = String::from("$'");
    let push_byte = |byte: u8, quoted: &mut String| quoted.push_str(&format!("\\x{byte:02x}"));
    match text {
        Some(text) => {
            for c in text.chars() {
                match c {
                    '\'' => quoted.push_str("\\'"),
                    '\\' => quoted.push_str("\\\\"),
                    c if c.is_control() => c.to_string().bytes().for_each(|byte| push_byte(byte, &mut quoted)),
                    c => quoted.push(c),
                }
            }
        }
        None => {
            for &byte in bytes {
                match byte {
                    b'\'' => quoted.push_str("\\'"),
                    b'\\' => quoted.push_str("\\\\"),
                    0x20..=0x7e => quoted.push(char::from(byte)),
                    other => push_byte(other, &mut quoted),
                }
            }
        }
    }
    quoted.push('\'');
    quoted
}

/// `text` in single quotes for PowerShell, the terminals' shell on Windows: a quote in it becomes `''`.
#[cfg(windows)]
pub fn single_quoted(text: &str) -> String {
    format!("'{}'", text.replace('\'', "''"))
}

/// The text a paste of `kind` types, with `save` turning the image into a file; `None` when `contents` has
/// nothing of that kind.
pub fn paste_text(kind: PasteKind, contents: Contents, save: impl FnOnce(&Image) -> io::Result<PathBuf>) -> io::Result<Option<String>> {
    let Contents { text, image } = contents;
    let (text, image) = match kind {
        PasteKind::TextElseImage if text.is_some() => (text, None),
        PasteKind::TextElseImage | PasteKind::TextAndImage => (text, image),
        PasteKind::Text => (text, None),
        PasteKind::Image => (None, image),
    };
    let path = image.as_ref().map(save).transpose()?.map(|path| shell_words(&[path]));
    Ok(match (text, path) {
        (Some(text), Some(path)) => Some(format!("{text} {path}")),
        (text, path) => text.or(path),
    })
}

/// Saves `image` to this profile's paste folder; the path of the new file.
pub fn save_image(image: &Image) -> io::Result<PathBuf> {
    let dir = profile::current().pastes_dir().ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "the OS reports no cache folder"))?;
    save_image_in(&dir, image, SystemTime::now())
}

/// Deletes saved images older than a week from this profile's paste folder; nothing when it has none.
pub fn delete_old_images() -> io::Result<()> {
    match profile::current().pastes_dir() {
        Some(dir) => delete_older_than(&dir, SystemTime::now() - KEPT_FOR),
        None => Ok(()),
    }
}

/// The folder and the images are the user's alone: a screenshot can show secrets.
fn save_image_in(dir: &Path, image: &Image, now: SystemTime) -> io::Result<PathBuf> {
    private_dir(dir)?;
    let millis = now.duration_since(UNIX_EPOCH).unwrap_or_default().as_millis();
    let (bytes, extension) = saved_form(image);
    let path = dir.join(format!("paste-{millis}-{:x}.{extension}", image.id));
    private_file(&path)?.write_all(&bytes)?;
    Ok(path)
}

#[cfg(unix)]
fn saved_form(image: &Image) -> (std::borrow::Cow<'_, [u8]>, &'static str) {
    (std::borrow::Cow::Borrowed(&image.bytes), image.format.extension())
}

/// A bitmap, what Print Screen puts on the Windows clipboard, is saved as PNG, which Claude Code reads.
#[cfg(windows)]
fn saved_form(image: &Image) -> (std::borrow::Cow<'_, [u8]>, &'static str) {
    let png = matches!(image.format, gpui::ImageFormat::Bmp).then(|| png_of_bmp(&image.bytes)).flatten();
    match png {
        Some(png) => (std::borrow::Cow::Owned(png), "png"),
        None => (std::borrow::Cow::Borrowed(&image.bytes), image.format.extension()),
    }
}

#[cfg(windows)]
fn png_of_bmp(bmp: &[u8]) -> Option<Vec<u8>> {
    let decoded = image::load_from_memory_with_format(bmp, image::ImageFormat::Bmp).ok()?;
    let mut png = io::Cursor::new(Vec::new());
    decoded.write_to(&mut png, image::ImageFormat::Png).ok()?;
    Some(png.into_inner())
}

#[cfg(unix)]
fn private_dir(dir: &Path) -> io::Result<()> {
    use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
    std::fs::DirBuilder::new().recursive(true).mode(0o700).create(dir)?;
    // a folder made before IVE kept it private
    std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700))
}

#[cfg(not(unix))]
fn private_dir(dir: &Path) -> io::Result<()> {
    std::fs::create_dir_all(dir)
}

fn private_file(path: &Path) -> io::Result<std::fs::File> {
    let mut options = std::fs::File::options();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    options.open(path)
}

fn delete_older_than(dir: &Path, cutoff: SystemTime) -> io::Result<()> {
    let entries = match std::fs::read_dir(dir) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        entries => entries?,
    };
    for entry in entries {
        let path = entry?.path();
        let modified = std::fs::metadata(&path)?.modified()?;
        if path.is_file() && modified < cutoff {
            std::fs::remove_file(&path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::ImageFormat;

    fn png(bytes: &[u8]) -> Image {
        Image::from_bytes(ImageFormat::Png, bytes.to_vec())
    }

    #[cfg(unix)]
    #[test]
    fn should_leave_plain_paths_unquoted() {
        // given
        let paths = [PathBuf::from("/mnt/data/Projects/wezterm"), PathBuf::from("/tmp/a_b-c.1+2,x:y@z%=.png")];

        // when
        let words = shell_words(&paths);

        // then
        assert_eq!(words, "/mnt/data/Projects/wezterm /tmp/a_b-c.1+2,x:y@z%=.png");
    }

    #[cfg(unix)]
    #[test]
    fn should_quote_paths_the_shell_would_split_or_expand() {
        // given
        let paths = [PathBuf::from("/tmp/my notes.md"), PathBuf::from("/tmp/it's"), PathBuf::from("~/x"), PathBuf::from("/tmp/$HOME")];

        // when
        let words = shell_words(&paths);

        // then
        assert_eq!(words, r"'/tmp/my notes.md' '/tmp/it'\''s' '~/x' '/tmp/$HOME'");
    }

    #[cfg(windows)]
    #[test]
    fn should_quote_paths_for_powershell() {
        // given
        let paths = [PathBuf::from(r"C:\Users\kb\a_b-c.1.png"), PathBuf::from(r"C:\my notes\it's"), PathBuf::from(r"C:\a,b"), PathBuf::from(r"C:\$HOME")];

        // when
        let words = shell_words(&paths);

        // then
        assert_eq!(words, r"C:\Users\kb\a_b-c.1.png 'C:\my notes\it''s' 'C:\a,b' 'C:\$HOME'");
    }

    // Windows file names cannot hold control characters.
    #[cfg(unix)]
    #[test]
    fn should_write_a_newline_as_an_escape_not_a_key() {
        // given
        let paths = [PathBuf::from("/tmp/a\nb")];

        // when
        let words = shell_words(&paths);

        // then
        assert_eq!(words, r"$'/tmp/a\x0ab'");
    }

    // Windows file names cannot hold control characters.
    #[cfg(unix)]
    #[test]
    fn should_write_control_characters_as_escapes_so_they_cannot_act_as_keys() {
        // given
        let paths = [PathBuf::from("/r/x\u{15}touch /tmp/pwned\n"), PathBuf::from("/r/\u{3}\u{1b}[2~it's\u{7f}")];

        // when
        let words = shell_words(&paths);

        // then
        assert_eq!(words, r"$'/r/x\x15touch /tmp/pwned\x0a' $'/r/\x03\x1b[2~it\'s\x7f'");
        assert!(!words.chars().any(|c| c.is_control()), "{words:?}");
    }

    #[cfg(unix)]
    #[test]
    fn should_write_bytes_that_are_not_utf8_as_escapes() {
        // given
        use std::os::unix::ffi::OsStrExt as _;
        let path = PathBuf::from(std::ffi::OsStr::from_bytes(b"/r/caf\xe9 \\x"));

        // when
        let words = shell_words(&[path]);

        // then
        assert_eq!(words, r"$'/r/caf\xe9 \\x'");
    }

    #[test]
    fn should_quote_an_empty_path() {
        // when
        let words = shell_words(&[PathBuf::new()]);

        // then
        assert_eq!(words, "''");
    }

    fn contents(text: Option<&str>, image: bool) -> Contents {
        Contents { text: text.map(str::to_owned), image: image.then(|| png(b"x")) }
    }

    fn saved(_image: &Image) -> io::Result<PathBuf> {
        Ok(PathBuf::from("/p/x.png"))
    }

    fn not_saved(_image: &Image) -> io::Result<PathBuf> {
        panic!("the image must not be saved")
    }

    #[test]
    fn should_paste_text_when_there_is_text() {
        // when
        let text = paste_text(PasteKind::TextElseImage, contents(Some("hello"), true), not_saved).unwrap();

        // then
        assert_eq!(text.as_deref(), Some("hello"));
    }

    #[test]
    fn should_paste_image_path_when_there_is_no_text() {
        // when
        let text = paste_text(PasteKind::TextElseImage, contents(None, true), saved).unwrap();

        // then
        assert_eq!(text.as_deref(), Some("/p/x.png"));
    }

    #[test]
    fn should_paste_only_text() {
        // when
        let pasted = [contents(Some("hello"), true), contents(None, true)].map(|both| paste_text(PasteKind::Text, both, not_saved).unwrap());

        // then
        assert_eq!(pasted, [Some("hello".to_owned()), None]);
    }

    #[test]
    fn should_paste_only_image() {
        // when
        let pasted = [contents(Some("hello"), true), contents(Some("hello"), false)].map(|both| paste_text(PasteKind::Image, both, saved).unwrap());

        // then
        assert_eq!(pasted, [Some("/p/x.png".to_owned()), None]);
    }

    #[test]
    fn should_paste_text_then_image_path() {
        // when
        let text = paste_text(PasteKind::TextAndImage, contents(Some("hello"), true), saved).unwrap();

        // then
        assert_eq!(text.as_deref(), Some("hello /p/x.png"));
    }

    #[test]
    fn should_paste_whichever_exists_for_text_and_image() {
        // when
        let pasted = [contents(Some("hello"), false), contents(None, true), contents(None, false)]
            .map(|one| paste_text(PasteKind::TextAndImage, one, saved).unwrap());

        // then
        assert_eq!(pasted, [Some("hello".to_owned()), Some("/p/x.png".to_owned()), None]);
    }

    #[test]
    fn should_save_image_with_its_format_extension() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let image = png(b"not really a png");

        // when
        let path = save_image_in(&dir.path().join("pastes"), &image, UNIX_EPOCH + Duration::from_millis(1234)).unwrap();

        // then
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        assert!(name.starts_with("paste-1234-") && name.ends_with(".png"), "{name}");
        assert_eq!(std::fs::read(&path).unwrap(), b"not really a png");
    }

    #[cfg(windows)]
    #[test]
    fn should_save_a_bitmap_as_png() {
        // given
        let mut bmp = io::Cursor::new(Vec::new());
        image::RgbImage::new(2, 1).write_to(&mut bmp, image::ImageFormat::Bmp).unwrap();
        let image = Image::from_bytes(ImageFormat::Bmp, bmp.into_inner());
        let dir = tempfile::tempdir().unwrap();

        // when
        let path = save_image_in(&dir.path().join("pastes"), &image, UNIX_EPOCH).unwrap();

        // then
        let saved = image::open(&path).unwrap();
        assert_eq!((path.extension().unwrap().to_str(), saved.width(), saved.height()), (Some("png"), 2, 1));
    }

    #[cfg(unix)]
    #[test]
    fn should_keep_pasted_images_private() {
        // given
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let pastes = dir.path().join("pastes");

        // when
        let path = save_image_in(&pastes, &png(b"secret"), UNIX_EPOCH).unwrap();

        // then
        let mode = |path: &Path| std::fs::metadata(path).unwrap().permissions().mode() & 0o777;
        assert_eq!((mode(&pastes), mode(&path)), (0o700, 0o600));
    }

    #[test]
    fn should_delete_only_images_older_than_cutoff() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let old = save_image_in(dir.path(), &png(b"old"), UNIX_EPOCH).unwrap();
        let cutoff = SystemTime::now() - Duration::from_secs(60);
        std::fs::File::options().write(true).open(&old).unwrap().set_modified(cutoff - Duration::from_secs(1)).unwrap();
        let new = save_image_in(dir.path(), &png(b"new"), UNIX_EPOCH).unwrap();

        // when
        delete_older_than(dir.path(), cutoff).unwrap();

        // then
        assert_eq!((old.exists(), new.exists()), (false, true));
    }

    #[test]
    fn should_accept_a_missing_paste_folder() {
        // given
        let dir = tempfile::tempdir().unwrap();

        // when
        let deleted = delete_older_than(&dir.path().join("pastes"), SystemTime::now());

        // then
        assert!(deleted.is_ok());
    }
}
