use std::fmt;
use std::path::Path;
use std::sync::OnceLock;

use tree_sitter::Query;

use super::registry::{LANGUAGES, LanguageSpec};
use crate::colors::{ColorKey, Page};

/// A language IVE highlights: a row of the registry.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Language(u16);

/// A grammar with IVE's queries; `keys[i]` is the colour key of the highlight query's capture `i`, none for
/// captures named `@_…`, which only feed predicates.
pub struct Grammar {
    pub language: tree_sitter::Language,
    pub query: Query,
    pub keys: Vec<Option<ColorKey>>,
    pub injections: Option<Query>,
}

static GRAMMARS: [OnceLock<Result<Grammar, String>>; LANGUAGES.len()] = [const { OnceLock::new() }; LANGUAGES.len()];

/// Longest `#!` line read when looking for an interpreter.
const MAX_SHEBANG_BYTES: usize = 256;

impl Language {
    pub fn all() -> impl Iterator<Item = Self> {
        (0..LANGUAGES.len()).map(|index| Self(index as u16))
    }

    /// The registry row named exactly `name`, e.g. `"Java"`.
    #[cfg(test)]
    pub fn named(name: &str) -> Option<Self> {
        Self::find(|spec| spec.name == name)
    }

    /// The language an injection names, e.g. a Markdown fence's `py`; ignores case.
    pub fn for_alias(alias: &str) -> Option<Self> {
        let alias = alias.trim().to_ascii_lowercase();
        Self::find(|spec| spec.aliases.contains(&alias.as_str()))
    }

    /// By exact file name, then extension ignoring case, then the interpreter in the `#!` line `content` starts with.
    pub fn for_path(path: &Path, content: &[u8]) -> Option<Self> {
        let file_name = path.file_name()?.to_str()?;
        Self::find(|spec| spec.file_names.iter().any(|name| name_matches(name, file_name)))
            .or_else(|| Self::for_extension(path))
            .or_else(|| interpreter(content).and_then(|name| Self::find(|spec| spec.shebangs.contains(&name.as_str()))))
    }

    fn for_extension(path: &Path) -> Option<Self> {
        let extension = path.extension()?.to_str()?.to_ascii_lowercase();
        Self::find(|spec| spec.extensions.contains(&extension.as_str()))
    }

    fn find(matches: impl Fn(&LanguageSpec) -> bool) -> Option<Self> {
        LANGUAGES.iter().position(matches).map(|index| Self(index as u16))
    }

    pub fn spec(self) -> &'static LanguageSpec {
        &LANGUAGES[usize::from(self.0)]
    }

    pub fn name(self) -> &'static str {
        self.spec().name
    }

    /// The Settings page listing this language's keys; the registry is checked by tests, so a missing page is a bug.
    pub fn page(self) -> Page {
        let title = self.spec().page;
        Page::titled(title).unwrap_or_else(|| panic!("language {} names unknown page {title}", self.name()))
    }

    /// Compiled on first use. An error names the language and the query problem; it is a bug the tests catch.
    pub fn grammar(self) -> Result<&'static Grammar, &'static str> {
        GRAMMARS[usize::from(self.0)].get_or_init(|| Grammar::compile(self)).as_ref().map_err(String::as_str)
    }
}

impl fmt::Debug for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// `Dockerfile` matches only itself; `Dockerfile.*` also matches `Dockerfile.dev`, but not `Dockerfile.`.
fn name_matches(name: &str, file_name: &str) -> bool {
    match name.strip_suffix('*') {
        Some(prefix) => file_name.len() > prefix.len() && file_name.starts_with(prefix),
        None => name == file_name,
    }
}

/// `python` for `#!/usr/bin/env -S python3.12 -u` or `#!/usr/bin/python3`: the program's file name without a
/// version suffix.
fn interpreter(content: &[u8]) -> Option<String> {
    let head = &content[..content.len().min(MAX_SHEBANG_BYTES)];
    let line = head.split(|&byte| byte == b'\n').next()?;
    let line = std::str::from_utf8(line.strip_prefix(b"#!")?).ok()?;
    let mut words = line.split_whitespace().map(|word| word.rsplit('/').next().unwrap_or(word));
    let program = match words.next()? {
        "env" => words.find(|word| !word.starts_with('-'))?,
        program => program,
    };
    let name = program.trim_end_matches(|c: char| c.is_ascii_digit() || c == '.');
    (!name.is_empty()).then(|| name.to_owned())
}

/// The key a capture names. A capture name cannot hold a space, so it spells the spaces of IntelliJ ids such as
/// Groovy's `Groovy var` as `_`: `@Groovy_var`.
fn captured_key(capture: &str) -> Option<ColorKey> {
    ColorKey::by_id(capture).or_else(|| ColorKey::by_id(&capture.replace('_', " ")))
}

impl Grammar {
    fn compile(language: Language) -> Result<Self, String> {
        let spec = language.spec();
        let grammar = (spec.grammar)();
        let query =
            Query::new(&grammar, spec.highlights).map_err(|error| format!("{language:?} highlight query: {error}"))?;
        let keys = query
            .capture_names()
            .iter()
            .map(|name| match name.starts_with('_') {
                true => Ok(None),
                false => captured_key(name)
                    .map(Some)
                    .ok_or_else(|| format!("{language:?} highlight query captures @{name}, which is not a colour key")),
            })
            .collect::<Result<_, _>>()?;
        let injections = spec
            .injections
            .map(|source| Query::new(&grammar, source))
            .transpose()
            .map_err(|error| format!("{language:?} injection query: {error}"))?;
        Ok(Self { language: grammar, query, keys, injections })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    use super::*;

    /// Compiling checks every pattern against the grammar's parse tables, which can explode: one KDL pattern took
    /// 105 s. The slowest now, Swift, takes about 0.3 s in a debug test build; one past this limit would hang the
    /// test run and freeze a Settings page.
    const QUERY_COMPILE_LIMIT: Duration = Duration::from_secs(2);

    /// Runs `job` on its own thread; `None` if it has not finished after `limit`. A stuck thread is left behind: it
    /// cannot be stopped, and the test binary exits at the end of the run anyway.
    fn finished_within<T: Send + 'static>(limit: Duration, job: impl FnOnce() -> T + Send + 'static) -> Option<T> {
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            // the receiver is gone once the limit passed; nothing to report then
            let _ = sender.send(job());
        });
        receiver.recv_timeout(limit).ok()
    }

    /// Why `language`'s queries do not compile, or that they took over the limit. Compiles afresh on a thread of
    /// its own: the cached `grammar()` would block on a compile another test started, and a slow one cannot be
    /// stopped.
    fn compile_failure(language: Language) -> Option<String> {
        match finished_within(QUERY_COMPILE_LIMIT, move || Grammar::compile(language).err()) {
            Some(error) => error,
            None => Some(format!("{language:?} queries still compiling after {QUERY_COMPILE_LIMIT:?}")),
        }
    }

    fn language(name: &str) -> Option<Language> {
        Some(Language::named(name).unwrap())
    }

    fn for_name(path: &str) -> Option<Language> {
        Language::for_path(Path::new(path), b"")
    }

    /// Names claiming the same entry of `field` across rows.
    fn claimed_twice(field: fn(&LanguageSpec) -> &[&'static str]) -> Vec<(&'static str, Vec<&'static str>)> {
        let mut owners: HashMap<&str, Vec<&str>> = HashMap::new();
        for spec in LANGUAGES {
            for entry in field(spec) {
                owners.entry(entry).or_default().push(spec.name);
            }
        }
        owners.into_iter().filter(|(_, names)| names.len() > 1).collect()
    }

    #[test]
    fn should_give_up_waiting_for_a_job_past_its_time_limit() {
        // given
        let limit = Duration::from_millis(100);

        // when
        let results = (finished_within(limit, || 1), finished_within(limit, || thread::sleep(Duration::from_secs(5))));

        // then
        assert_eq!(results, (Some(1), None));
    }

    #[test]
    fn should_compile_every_language_query_with_known_keys() {
        // given
        let languages: Vec<Language> = Language::all().collect();

        // when
        let failures: Vec<String> = languages.into_iter().filter_map(compile_failure).collect();

        // then
        assert!(failures.is_empty(), "{failures:#?}");
    }

    #[test]
    fn should_read_underscores_of_capture_name_as_spaces_of_key_id() {
        // given
        let captures = ["Groovy_var", "GROOVY_KEYWORD", "Not_a_key"];

        // when
        let ids = captures.map(|capture| captured_key(capture).map(ColorKey::id));

        // then
        assert_eq!(ids, [Some("Groovy var"), Some("GROOVY_KEYWORD"), None]);
    }

    #[test]
    fn should_name_an_existing_page_for_every_language() {
        // given
        let languages: Vec<Language> = Language::all().collect();

        // when
        let missing: Vec<&str> =
            languages.iter().filter(|language| Page::titled(language.spec().page).is_none()).map(|l| l.name()).collect();

        // then
        assert!(missing.is_empty(), "languages naming unknown pages: {missing:?}");
    }

    #[test]
    fn should_not_claim_an_extension_file_name_or_alias_twice() {
        // given
        let fields: [fn(&LanguageSpec) -> &[&'static str]; 4] =
            [|spec| spec.extensions, |spec| spec.file_names, |spec| spec.aliases, |spec| spec.shebangs];

        // when
        let duplicates: Vec<_> = fields.into_iter().flat_map(claimed_twice).collect();

        // then
        assert!(duplicates.is_empty(), "{duplicates:?}");
    }

    #[test]
    fn should_keep_extensions_and_aliases_lower_case() {
        // given
        let entries: Vec<&str> = LANGUAGES.iter().flat_map(|spec| spec.extensions.iter().chain(spec.aliases)).copied().collect();

        // when
        let upper: Vec<&str> = entries.into_iter().filter(|entry| entry.to_ascii_lowercase() != *entry).collect();

        // then
        assert!(upper.is_empty(), "{upper:?}");
    }

    #[test]
    fn should_pick_dockerfile_variants_by_name_pattern() {
        // given
        let paths = ["Dockerfile.dev", "Dockerfile.prod", "dockerfile", "Containerfile.ci", "Dockerfile"];

        // when
        let languages: Vec<_> = paths.iter().map(|path| for_name(path)).collect();

        // then
        assert_eq!(languages, [language("Dockerfile"); 5]);
    }

    #[test]
    fn should_not_take_a_name_pattern_for_a_bare_prefix() {
        // given
        let paths = ["Dockerfile.", "Dockerfiles", "Dockerfile.md.bak"];

        // when
        let languages: Vec<_> = paths.iter().map(|path| for_name(path)).collect();

        // then
        assert_eq!(languages, [None, None, language("Dockerfile")]);
    }

    #[test]
    fn should_pick_language_by_extension_ignoring_case() {
        // given
        let paths = ["Main.java", "app.MJS", "data.json", "lib.rs", "Cargo.toml", "x.cts", "View.tsx", "ci.yml"];

        // when
        let languages: Vec<_> = paths.iter().map(|path| for_name(path)).collect();

        // then
        let expected = ["Java", "JavaScript", "JSON", "Rust", "TOML", "TypeScript", "TSX", "YAML"];
        assert_eq!(languages, expected.map(language));
    }

    #[test]
    fn should_pick_language_by_shebang_with_env_and_version_suffix() {
        // given
        let lines: [&[u8]; 3] = [b"#!/usr/bin/env node\n", b"#!/usr/bin/env -S deno run\n", b"#!/usr/local/bin/node22.1"];

        // when
        let languages: Vec<_> = lines.iter().map(|line| Language::for_path(Path::new("script"), line)).collect();

        // then
        assert_eq!(languages, [language("JavaScript"), language("TypeScript"), language("JavaScript")]);
    }

    #[test]
    fn should_leave_unknown_interpreter_and_extension_plain() {
        // given
        let cases: [(&str, &[u8]); 3] = [("server.log", b""), ("notes.txt", b"#!/bin/frobnicate\n"), ("run", b"echo hi")];

        // when
        let languages: Vec<_> = cases.iter().map(|(path, line)| Language::for_path(Path::new(path), line)).collect();

        // then
        assert_eq!(languages, [None, None, None]);
    }

    #[test]
    fn should_resolve_alias_ignoring_case() {
        // given
        let aliases = ["JS", " ts ", "yml"];

        // when
        let languages: Vec<_> = aliases.iter().map(|alias| Language::for_alias(alias)).collect();

        // then
        assert_eq!(languages, [language("JavaScript"), language("TypeScript"), language("YAML")]);
    }
}
