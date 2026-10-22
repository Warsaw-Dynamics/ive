//! The icons the views draw. The host serves them as embedded assets under `icons/dark/` and `icons/light/`.

use gpui::{ImageSource, Resource};

use crate::style::Appearance;

pub const CHEVRON_RIGHT: &str = "chevron-right.svg";
pub const CHEVRON_DOWN: &str = "chevron-down.svg";

/// The embedded icon `name` in the variant drawn for `appearance`.
pub fn embedded(appearance: Appearance, name: &str) -> ImageSource {
    let variant = match appearance {
        Appearance::Dark => "dark",
        Appearance::Light => "light",
    };
    // built explicitly: `ImageSource::from(&str)` would read the path as a URI if it parses as one
    ImageSource::Resource(Resource::Embedded(format!("icons/{variant}/{name}").into()))
}
