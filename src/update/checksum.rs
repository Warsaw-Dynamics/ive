//! Verifying a download against the release's `SHA256SUMS`.

use std::fmt::Write as _;
use std::fs::File;
use std::io;
use std::path::Path;

use sha2::{Digest, Sha256};

/// `(name, lowercase hex)` per line of `sha256sum` output (`<hash>  <name>`, or `<hash> *<name>` in binary mode);
/// other lines are skipped.
pub fn parse_sums(text: &str) -> Vec<(String, String)> {
    text.lines().filter_map(parse_line).collect()
}

fn parse_line(line: &str) -> Option<(String, String)> {
    let (hash, name) = line.split_once(' ')?;
    let name = name.trim_start_matches([' ', '*']);
    let is_hash = hash.len() == 64 && hash.chars().all(|char| char.is_ascii_hexdigit());
    (is_hash && !name.is_empty()).then(|| (name.to_owned(), hash.to_ascii_lowercase()))
}

/// Checks that `file`, published as `name`, has the hash `sums` gives it.
pub fn verify(file: &Path, name: &str, sums: &[(String, String)]) -> Result<(), String> {
    let expected = sums.iter().find(|(listed, _)| listed == name).map(|(_, hash)| hash).ok_or_else(|| format!("{name} is not in the release's checksums"))?;
    let actual = sha256(file).map_err(|error| format!("cannot read {} to verify it: {error}", file.display()))?;
    if &actual == expected {
        return Ok(());
    }
    Err(format!("{name} downloaded to {} has SHA-256 {actual}, the release says {expected}", file.display()))
}

fn sha256(file: &Path) -> io::Result<String> {
    let mut hasher = Sha256::new();
    io::copy(&mut File::open(file)?, &mut hasher)?;
    Ok(hasher.finalize().iter().fold(String::with_capacity(64), |mut hex, byte| {
        let _ = write!(hex, "{byte:02x}");
        hex
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    const HELLO_SHA256: &str = "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824";

    #[test]
    fn should_parse_sums_in_both_formats() {
        // given
        let text = format!("{HELLO_SHA256}  a.tar.gz\n{}  *b.zip\n\nnot a line\n", HELLO_SHA256.to_uppercase());

        // when
        let sums = parse_sums(&text);

        // then
        assert_eq!(sums, vec![("a.tar.gz".to_owned(), HELLO_SHA256.to_owned()), ("b.zip".to_owned(), HELLO_SHA256.to_owned())]);
    }

    #[test]
    fn should_accept_a_file_matching_its_checksum() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("a.tar.gz");
        std::fs::write(&file, "hello").unwrap();
        let sums = vec![("a.tar.gz".to_owned(), HELLO_SHA256.to_owned())];

        // when
        let verified = verify(&file, "a.tar.gz", &sums);

        // then
        assert_eq!(verified, Ok(()));
    }

    #[test]
    fn should_reject_a_download_whose_checksum_differs() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("a.tar.gz");
        std::fs::write(&file, "hell").unwrap();
        let sums = vec![("a.tar.gz".to_owned(), HELLO_SHA256.to_owned())];

        // when
        let verified = verify(&file, "a.tar.gz", &sums);

        // then
        assert!(verified.unwrap_err().contains(HELLO_SHA256));
    }

    #[test]
    fn should_reject_a_file_missing_from_the_sums() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("a.tar.gz");
        std::fs::write(&file, "hello").unwrap();

        // when
        let verified = verify(&file, "a.tar.gz", &[]);

        // then
        assert!(verified.unwrap_err().contains("a.tar.gz"));
    }
}
