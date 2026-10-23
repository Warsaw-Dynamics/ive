// Adapted from pgui crates/gpui/src/svg_renderer.rs at
// f5d5fa2f8f07b0be68442e694d22ed6064609e73 (Apache-2.0).
// Copyright 2022 - 2025 Zed Industries, Inc.
// Modified for the standalone viewer's empty asset source; no bundled fonts.
// Full licence and provenance: ../../THIRD-PARTY-NOTICES.md.
use std::sync::{Arc, LazyLock};
use usvg::fontdb::{Database, Family, ID, Query};

#[cfg(target_os = "macos")]
const EMOJI_FAMILIES: &[&str] = &["Apple Color Emoji", ".AppleColorEmojiUI"];
#[cfg(target_os = "windows")]
const EMOJI_FAMILIES: &[&str] = &["Segoe UI Emoji", "Segoe UI Symbol"];
#[cfg(any(target_os = "linux", target_os = "freebsd"))]
const EMOJI_FAMILIES: &[&str] = &[
    "Noto Color Emoji",
    "Emoji One",
    "Twitter Color Emoji",
    "JoyPixels",
];
#[cfg(not(any(
    target_os = "macos",
    target_os = "windows",
    target_os = "linux",
    target_os = "freebsd"
)))]
const EMOJI_FAMILIES: &[&str] = &[];

pub(super) fn options() -> usvg::Options<'static> {
    static DATABASE: LazyLock<Arc<Database>> = LazyLock::new(|| {
        let mut database = Database::new();
        database.load_system_fonts();
        fix_generic_families(&mut database);
        Arc::new(database)
    });
    let select = usvg::FontResolver::default_font_selector();
    let fallback = usvg::FontResolver::default_fallback_selector();
    usvg::Options {
        fontdb: DATABASE.clone(),
        font_resolver: usvg::FontResolver {
            select_font: Box::new(move |font, db| {
                select(font, db)
                    .or_else(|| {
                        db.query(&Query {
                            families: &[Family::SansSerif],
                            ..Default::default()
                        })
                    })
                    .or_else(|| db.faces().next().map(|face| face.id))
            }),
            select_fallback: Box::new(move |ch, used, db| {
                select_emoji(ch, used, db).or_else(|| fallback(ch, used, db))
            }),
        },
        ..Default::default()
    }
}

fn select_emoji(ch: char, used: &[ID], db: &Database) -> Option<ID> {
    static EMOJI: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new("\\p{Emoji_Presentation}").unwrap());
    if EMOJI.is_match(ch.encode_utf8(&mut [0; 4])) {
        return EMOJI_FAMILIES.iter().find_map(|name| {
            let id = db.query(&Query {
                families: &[Family::Name(name)],
                ..Default::default()
            })?;
            let has_char = db
                .with_face_data(id, |data, index| {
                    ttf_parser::Face::parse(data, index)
                        .ok()
                        .and_then(|face| face.glyph_index(ch))
                        .is_some()
                })
                .unwrap_or(false);
            if used.contains(&id) {
                return None;
            }
            has_char.then_some(id)
        });
    }
    None
}

fn fix_generic_families(db: &mut Database) {
    for (family, fallback) in [
        (Family::SansSerif, "IBM Plex Sans"),
        (Family::Serif, "IBM Plex Sans"),
        (Family::Monospace, "Lilex"),
        (Family::Cursive, "IBM Plex Sans"),
        (Family::Fantasy, "IBM Plex Sans"),
    ] {
        if db
            .query(&Query {
                families: &[family],
                ..Default::default()
            })
            .is_none()
        {
            match family {
                Family::SansSerif => db.set_sans_serif_family(fallback),
                Family::Serif => db.set_serif_family(fallback),
                Family::Monospace => db.set_monospace_family(fallback),
                Family::Cursive => db.set_cursive_family(fallback),
                Family::Fantasy => db.set_fantasy_family(fallback),
                _ => {}
            }
        }
    }
}
