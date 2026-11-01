//! The files whose long lines wrap, switched per file (the gutter menu, `act::ToggleSoftWrap`) and kept in
//! `state.toml`: a file opens wrapped when it was last left wrapped.

use std::path::{Path, PathBuf};

use codeview::panel::{Cast, Event, Panel};
use codeview::text_editor::SoftWrap;
use gpui::{App, Entity, Global};

/// Files remembered as wrapped; switching on one more forgets the one switched on longest ago.
pub const MAX_SOFT_WRAPPED: usize = 1000;

/// The wrapped files, the one switched on last at the end.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SoftWraps {
    paths: Vec<PathBuf>,
}

impl Global for SoftWraps {}

impl SoftWraps {
    /// As `state.toml` kept them; past [`MAX_SOFT_WRAPPED`], the last ones.
    pub fn restored(paths: &[PathBuf]) -> Self {
        Self { paths: paths[paths.len().saturating_sub(MAX_SOFT_WRAPPED)..].to_vec() }
    }

    pub fn of(&self, path: &Path) -> SoftWrap {
        if self.paths.iter().any(|wrapped| wrapped == path) {
            return SoftWrap::On;
        }
        SoftWrap::Off
    }

    /// These files with `path` wrapped as `soft_wrap` says.
    pub fn with(&self, path: &Path, soft_wrap: SoftWrap) -> Self {
        let others = self.paths.iter().filter(|wrapped| *wrapped != path).cloned();
        let paths: Vec<PathBuf> = match soft_wrap {
            SoftWrap::On => others.chain([path.to_path_buf()]).collect(),
            SoftWrap::Off => others.collect(),
        };
        Self::restored(&paths)
    }

    pub fn paths(&self) -> &[PathBuf] {
        &self.paths
    }
}

/// The panel of the file or scratch at `path` opens wrapped when it was left wrapped, and its switches are kept.
/// Call it before the panel's first [`Cast::Open`].
pub fn follow(panel: &Entity<Panel>, path: PathBuf, cx: &mut App) {
    let soft_wrap = cx.try_global::<SoftWraps>().map(|wraps| wraps.of(&path)).unwrap_or_default();
    panel.update(cx, |panel, cx| panel.handle(Cast::SetSoftWrap(soft_wrap), cx));
    cx.subscribe(panel, move |_panel, event: &Event, cx| {
        if let Event::SoftWrapToggled(soft_wrap) = event {
            let wraps = cx.try_global::<SoftWraps>().cloned().unwrap_or_default();
            cx.set_global(wraps.with(&path, *soft_wrap));
        }
    })
    .detach();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths(count: usize) -> Vec<PathBuf> {
        (0..count).map(|at| PathBuf::from(format!("/f{at}"))).collect()
    }

    #[test]
    fn should_wrap_the_files_switched_on_and_not_those_switched_off() {
        // given
        let wraps = SoftWraps::default().with(Path::new("/a"), SoftWrap::On).with(Path::new("/b"), SoftWrap::On);

        // when
        let wraps = wraps.with(Path::new("/a"), SoftWrap::Off);

        // then
        assert_eq!([wraps.of(Path::new("/a")), wraps.of(Path::new("/b"))], [SoftWrap::Off, SoftWrap::On]);
    }

    #[test]
    fn should_forget_the_file_switched_on_longest_ago_past_the_limit() {
        // given
        let wraps = SoftWraps::restored(&paths(MAX_SOFT_WRAPPED)).with(Path::new("/f0"), SoftWrap::On);

        // when
        let wraps = wraps.with(Path::new("/new"), SoftWrap::On);

        // then
        assert_eq!(wraps.paths().len(), MAX_SOFT_WRAPPED);
        assert_eq!([wraps.of(Path::new("/f0")), wraps.of(Path::new("/f1")), wraps.of(Path::new("/new"))], [SoftWrap::On, SoftWrap::Off, SoftWrap::On]);
    }

    #[test]
    fn should_keep_the_last_files_of_an_oversized_list() {
        // given
        let saved = paths(MAX_SOFT_WRAPPED + 5);

        // when
        let wraps = SoftWraps::restored(&saved);

        // then
        assert_eq!(wraps.paths(), &saved[5..]);
    }
}
