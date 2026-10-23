//! Ordered files/notes semantics translated from PlantUML MIT FEntry/FilesListing/FilesDiagram.
//! Copyright 2009-2024 Arnaud Roques; MIT grant and source traces in provenance/.
use super::{
    budget::{Budget, bound},
    parser::{error::ParseFailure, outline::Arena},
};
use std::ops::Range;
#[derive(Debug, PartialEq, Eq)]
pub(super) enum Kind {
    Root,
    Folder,
    File,
    Note,
}
#[derive(Debug)]
pub(super) struct Entry {
    pub parent: usize,
    pub depth: usize,
    pub kind: Kind,
    pub name: String,
    pub children: Vec<usize>,
}
pub(super) struct Files {
    pub entries: Vec<Entry>,
}

pub(in crate::plantuml) struct Builder<'b> {
    pub budget: &'b mut Budget,
    arena: Arena,
    model: Files,
    anchor: usize,
    parent: usize,
    note: String,
    note_rows: usize,
}
impl<'b> Builder<'b> {
    #[cfg(test)]
    pub fn with_test_model(budget: &'b mut Budget, arena: Arena, model: &mut Files) -> Self {
        Self {
            budget,
            arena,
            model: Files {
                entries: std::mem::take(&mut model.entries),
            },
            anchor: 0,
            parent: 0,
            note: String::new(),
            note_rows: 0,
        }
    }
    #[cfg(test)]
    pub fn restore_test_model(self, model: &mut Files) -> usize {
        model.entries = self.model.entries;
        self.anchor
    }
    pub fn new(budget: &'b mut Budget, arena: Arena) -> Result<Self, ParseFailure> {
        budget.item().map_err(ParseFailure::terminal)?;
        let mut builder = Self {
            budget,
            arena,
            model: Files {
                entries: Vec::new(),
            },
            anchor: 0,
            parent: 0,
            note: String::new(),
            note_rows: 0,
        };
        builder.arena.push(
            &mut builder.model.entries,
            Entry {
                parent: 0,
                depth: 0,
                kind: Kind::Root,
                name: String::new(),
                children: Vec::new(),
            },
            builder.budget,
        )?;
        Ok(builder)
    }
    pub fn begin_path(&mut self) -> Result<(), ParseFailure> {
        self.budget.charge(1).map_err(ParseFailure::terminal)?;
        self.parent = 0;
        Ok(())
    }
    pub fn folder(&mut self, name: String) -> Result<(), ParseFailure> {
        for &id in &self.model.entries[self.parent].children {
            self.budget
                .charge(name.len() + 1)
                .map_err(ParseFailure::terminal)?;
            let entry = &self.model.entries[id];
            if entry.kind == Kind::Folder && entry.name == name {
                self.parent = id;
                return Ok(());
            }
        }
        self.budget.item().map_err(ParseFailure::terminal)?;
        self.budget
            .text(name.len())
            .map_err(ParseFailure::terminal)?;
        self.parent = self.append((self.parent, Kind::Folder, name))?;
        Ok(())
    }
    pub fn file(&mut self, name: String) -> Result<(), ParseFailure> {
        self.budget.item().map_err(ParseFailure::terminal)?;
        self.budget
            .text(name.len())
            .map_err(ParseFailure::terminal)?;
        self.append((self.parent, Kind::File, name))?;
        self.anchor = self.parent;
        Ok(())
    }
    pub fn folder_only(&mut self) -> Result<(), ParseFailure> {
        self.budget.charge(1).map_err(ParseFailure::terminal)?;
        self.anchor = 0;
        Ok(())
    }
    pub fn begin_note(&mut self) -> Result<(), ParseFailure> {
        self.budget
            .depth(self.model.entries[self.anchor].depth + 1)
            .map_err(ParseFailure::terminal)?;
        self.budget.item().map_err(ParseFailure::terminal)?;
        self.note.clear();
        self.note_rows = 0;
        Ok(())
    }
    pub fn note_row(&mut self, row: String) -> Result<(), ParseFailure> {
        let separator = usize::from(self.note_rows > 0);
        bound(
            "label bytes",
            self.note
                .len()
                .saturating_add(row.len())
                .saturating_add(separator),
            16 * 1024,
        )
        .map_err(ParseFailure::terminal)?;
        self.budget
            .text(row.len() + separator)
            .map_err(ParseFailure::terminal)?;
        if self.note.len() + row.len() + separator > self.note.capacity() {
            self.budget
                .charge(self.note.len())
                .map_err(ParseFailure::terminal)?;
        }
        if separator > 0 {
            self.note.push('\n');
        }
        self.note.push_str(&row);
        self.note_rows += 1;
        Ok(())
    }
    pub fn finish_note(&mut self) -> Result<(), ParseFailure> {
        let note = std::mem::take(&mut self.note);
        self.append((self.anchor, Kind::Note, note))?;
        Ok(())
    }
    fn append(&mut self, entry: (usize, Kind, String)) -> Result<usize, ParseFailure> {
        let depth = self.model.entries[entry.0].depth + 1;
        self.budget.depth(depth).map_err(ParseFailure::terminal)?;
        let id = self.model.entries.len();
        self.arena.push(
            &mut self.model.entries,
            Entry {
                parent: entry.0,
                depth,
                kind: entry.1,
                name: entry.2,
                children: Vec::new(),
            },
            self.budget,
        )?;
        self.arena
            .push(&mut self.model.entries[entry.0].children, id, self.budget)?;
        Ok(id)
    }
    pub fn finish(self, span: Range<usize>) -> Result<Files, ParseFailure> {
        self.budget.checkpoint().map_err(ParseFailure::terminal)?;
        if self.model.entries.len() == 1 {
            return Err(ParseFailure::syntax(span, "files tree has no entries"));
        }
        Ok(self.model)
    }
}
#[cfg(test)]
fn add_test_path(
    model: &mut Files,
    path: (usize, &str),
    budget: &mut Budget,
) -> Result<usize, crate::RenderError> {
    // Feed typed actions directly so the existing tests isolate semantic admission.
    let folders = path.1.ends_with('/');
    let raw = path.1.strip_suffix('/').unwrap_or(path.1);
    budget.charge(raw.len())?;
    let components = raw.split('/').collect::<Vec<_>>();
    budget.depth(components.len())?;
    let mut builder = Builder::with_test_model(budget, Arena::for_test(), model);
    let result = (|| {
        builder.begin_path()?;
        let count = components.len();
        for (index, name) in components.into_iter().enumerate() {
            builder
                .budget
                .charge(name.len() + 1)
                .map_err(ParseFailure::terminal)?;
            if folders || index + 1 < count {
                builder.folder(name.into())?;
            } else {
                builder.file(name.into())?;
            }
        }
        if folders {
            builder.folder_only()?;
        }
        Ok(())
    })();
    let anchor = builder.restore_test_model(model);
    result
        .map(|()| anchor)
        .map_err(|error: ParseFailure| error.into_line(path.0))
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::RenderOptions;

    #[test]
    fn should_keep_stable_parents_distinct_kinds_and_note_rows() {
        // given
        let input = "/a\n/a/x\n/a/x\n<note>\n\nline\n</note>\n/empty/\n<note>\nroot\n</note>";
        let lines = input.to_owned();
        let mut budget = Budget::new(&RenderOptions::default());
        // when
        let files = crate::plantuml::parser::test_support::body(
            (&lines, "files"),
            &mut budget,
            crate::plantuml::parser::files::parse,
        )
        .unwrap();
        // then
        assert_eq!(files.entries[0].children, [1, 2, 6, 7]);
        assert_eq!(files.entries[1].kind, Kind::File);
        assert_eq!(files.entries[2].kind, Kind::Folder);
        assert_eq!(files.entries[2].children, [3, 4, 5]);
        assert_eq!(files.entries[5].parent, 2);
        assert_eq!(files.entries[5].name, "\nline");
        assert_eq!(files.entries[7].parent, 0);
    }

    #[test]
    fn should_bound_file_depth_items_names_and_wide_lookup_work() {
        // given
        let mut options = RenderOptions::default();
        options.limits.nesting_depth = 2;
        let mut files = Files {
            entries: vec![Entry {
                parent: 0,
                depth: 0,
                kind: Kind::Root,
                name: String::new(),
                children: Vec::new(),
            }],
        };
        let mut budget = Budget::new(&options);
        // when
        let deep = add_test_path(&mut files, (1, "a/b/c"), &mut budget);
        // then
        assert!(deep.is_err());
        assert_eq!(files.entries.len(), 1);
        options.limits.layout_work = 20;
        let mut budget = Budget::new(&options);
        assert!(add_test_path(&mut files, (1, &"a".repeat(100)), &mut budget).is_err());
        assert_eq!(files.entries.len(), 1);
        options.limits.layout_work = 5_000_000;
        options.limits.model_items = 1;
        let mut budget = Budget::new(&options);
        budget.item().unwrap();
        assert!(add_test_path(&mut files, (1, "a"), &mut budget).is_err());
        assert_eq!(files.entries.len(), 1);
    }

    #[test]
    fn should_charge_wide_sibling_lookup_before_retaining_a_new_folder() {
        // given
        let mut budget = Budget::new(&RenderOptions::default());
        let lines = (0..40)
            .map(|n| format!("/folder{n}/"))
            .collect::<Vec<_>>()
            .join("\n");
        let mut model = crate::plantuml::parser::test_support::body(
            (&lines, "files"),
            &mut budget,
            crate::plantuml::parser::files::parse,
        )
        .unwrap();
        let mut options = RenderOptions::default();
        options.limits.layout_work = 32;
        let mut budget = Budget::new(&options);
        // when
        let result = add_test_path(&mut model, (41, "other/"), &mut budget);
        // then
        assert!(result.is_err());
        assert_eq!(model.entries.len(), 41);
        assert_eq!(model.entries[0].children.len(), 40);
    }
}
