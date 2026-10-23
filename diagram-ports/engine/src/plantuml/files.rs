//! Ordered files/notes semantics translated from PlantUML MIT FEntry/FilesListing/FilesDiagram.
//! Copyright 2009-2024 Arnaud Roques; MIT grant and source traces in provenance/.
use super::{
    budget::{Budget, bound, invalid, unsupported},
    source::Line,
};
use crate::RenderError;

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

pub(super) fn parse(lines: &[Line], budget: &mut Budget) -> Result<Files, RenderError> {
    budget.item()?;
    let mut model = Files {
        entries: vec![Entry {
            parent: 0,
            depth: 0,
            kind: Kind::Root,
            name: String::new(),
            children: Vec::new(),
        }],
    };
    let mut anchor = 0;
    let mut index = 0;
    while let Some(line) = lines.get(index) {
        budget.charge(line.text.len() + 1)?;
        let text = line.text.as_str();
        index += 1;
        if text.trim().is_empty() {
            continue;
        }
        if text == "<note>" {
            budget.depth(model.entries[anchor].depth + 1)?;
            budget.item()?;
            let mut note = String::new();
            let mut rows = 0;
            let mut closed = false;
            while let Some(row) = lines.get(index) {
                index += 1;
                budget.charge(row.text.len() + 1)?;
                if row.text.trim() == "</note>" {
                    closed = true;
                    break;
                }
                bound(
                    "label bytes",
                    note.len()
                        .saturating_add(row.text.len())
                        .saturating_add(usize::from(rows > 0)),
                    16 * 1024,
                )?;
                if rows > 0 {
                    budget.text(1)?;
                    note.push('\n');
                }
                budget.text(row.text.len())?;
                note.push_str(&row.text);
                rows += 1;
            }
            if !closed {
                return Err(invalid(line.number, "unterminated files <note>"));
            }
            append(&mut model, (anchor, Kind::Note, note), budget)?;
        } else if let Some(path) = text.strip_prefix('/') {
            anchor = add_path(&mut model, (line.number, path), budget)?;
        } else {
            return Err(unsupported(line.number, text));
        }
    }
    if model.entries.len() == 1 {
        return Err(invalid(1, "files tree has no entries"));
    }
    Ok(model)
}

fn add_path(
    model: &mut Files,
    path: (usize, &str),
    budget: &mut Budget,
) -> Result<usize, RenderError> {
    let folder_only = path.1.ends_with('/');
    let raw = if folder_only {
        &path.1[..path.1.len() - 1]
    } else {
        path.1
    };
    if raw.is_empty() {
        return Err(invalid(path.0, format!("empty files path {:?}", path.1)));
    }
    budget.charge(raw.len())?;
    let count = raw.split('/').count();
    budget.depth(count)?;
    let mut parent = 0;
    for (index, segment) in raw.split('/').enumerate() {
        budget.charge(segment.len() + 1)?;
        if segment.is_empty() {
            return Err(invalid(
                path.0,
                format!("empty files component in {:?}", path.1),
            ));
        }
        bound("label bytes", segment.len(), 16 * 1024)?;
        let kind = if index + 1 == count && !folder_only {
            Kind::File
        } else {
            Kind::Folder
        };
        let mut existing = None;
        if kind == Kind::Folder {
            for &child in &model.entries[parent].children {
                budget.charge(segment.len() + 1)?;
                let entry = &model.entries[child];
                if entry.kind == Kind::Folder && entry.name == segment {
                    existing = Some(child);
                    break;
                }
            }
        }
        parent = if let Some(id) = existing {
            id
        } else {
            budget.item()?;
            budget.text(segment.len())?;
            append(model, (parent, kind, segment.to_owned()), budget)?
        };
    }
    Ok(if folder_only {
        0
    } else {
        model.entries[parent].parent
    })
}

fn append(
    model: &mut Files,
    entry: (usize, Kind, String),
    budget: &mut Budget,
) -> Result<usize, RenderError> {
    budget.depth(model.entries[entry.0].depth + 1)?;
    let id = model.entries.len();
    model.entries.push(Entry {
        parent: entry.0,
        depth: model.entries[entry.0].depth + 1,
        kind: entry.1,
        name: entry.2,
        children: Vec::new(),
    });
    model.entries[entry.0].children.push(id);
    Ok(id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RenderOptions;

    #[test]
    fn should_keep_stable_parents_distinct_kinds_and_note_rows() {
        // given
        let input = "/a\n/a/x\n/a/x\n<note>\n\nline\n</note>\n/empty/\n<note>\nroot\n</note>";
        let lines: Vec<_> = input
            .lines()
            .enumerate()
            .map(|(i, s)| Line {
                number: i + 1,
                text: s.into(),
            })
            .collect();
        let mut budget = Budget::new(&RenderOptions::default());
        // when
        let files = parse(&lines, &mut budget).unwrap();
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
        let deep = add_path(&mut files, (1, "a/b/c"), &mut budget);
        // then
        assert!(deep.is_err());
        assert_eq!(files.entries.len(), 1);
        options.limits.layout_work = 20;
        let mut budget = Budget::new(&options);
        assert!(add_path(&mut files, (1, &"a".repeat(100)), &mut budget).is_err());
        assert_eq!(files.entries.len(), 1);
        options.limits.layout_work = 5_000_000;
        options.limits.model_items = 1;
        let mut budget = Budget::new(&options);
        budget.item().unwrap();
        assert!(add_path(&mut files, (1, "a"), &mut budget).is_err());
        assert_eq!(files.entries.len(), 1);
    }

    #[test]
    fn should_charge_wide_sibling_lookup_before_retaining_a_new_folder() {
        // given
        let mut budget = Budget::new(&RenderOptions::default());
        let lines: Vec<_> = (0..40)
            .map(|n| Line {
                number: n + 1,
                text: format!("/folder{n}/"),
            })
            .collect();
        let mut model = parse(&lines, &mut budget).unwrap();
        let mut options = RenderOptions::default();
        options.limits.layout_work = 32;
        let mut budget = Budget::new(&options);
        // when
        let result = add_path(&mut model, (41, "other/"), &mut budget);
        // then
        assert!(result.is_err());
        assert_eq!(model.entries.len(), 41);
        assert_eq!(model.entries[0].children.len(), 40);
    }
}
