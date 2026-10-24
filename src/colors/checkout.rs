//! Checkout colours, which are not theme fields.

use gpui::{Rgba, rgb};

use crate::project::Checkout;
use crate::theme::Theme;

/// A checkout's colour (card border, overview underline): the accent for the main checkout, otherwise one of
/// 8 hues picked by a hash of its path, so it stays the same across runs.
pub fn checkout_colour(checkout: &Checkout, theme: Theme) -> u32 {
    if checkout.is_main {
        return theme.accent;
    }
    let hues = theme.checkout_hues;
    hues[(fnv1a(checkout.path.as_os_str().as_encoded_bytes()) % hues.len() as u64) as usize]
}

/// `colour` (0xRRGGBB) at `fraction` opacity: a worktree's tint over what is under it.
pub fn tinted(colour: u32, fraction: f32) -> Rgba {
    Rgba { a: fraction, ..rgb(colour) }
}

/// 64-bit FNV-1a: stable across runs and platforms, unlike `std`'s hasher.
pub fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::theme::Appearance;

    fn checkout(path: &str, is_main: bool) -> Checkout {
        Checkout { path: PathBuf::from(path), label: String::new(), branch: None, head: None, is_main, status: None }
    }

    #[test]
    fn should_tint_with_the_fraction_as_alpha() {
        // given
        let colour = 0x3574f0;

        // when
        let tint = tinted(colour, 0.12);

        // then
        assert_eq!((tint.r, tint.g, tint.b), (f32::from(0x35_u8) / 255., f32::from(0x74_u8) / 255., f32::from(0xf0_u8) / 255.));
        assert!((tint.a - 0.12).abs() < 1e-6);
    }

    #[test]
    fn should_hash_like_reference_fnv1a() {
        // given
        let inputs: [&[u8]; 2] = [b"", b"a"];

        // when
        let hashes = inputs.map(fnv1a);

        // then
        assert_eq!(hashes, [0xcbf2_9ce4_8422_2325, 0xaf63_dc4c_8601_ec8c]);
    }

    #[test]
    fn should_give_main_checkout_the_accent_and_worktrees_a_stable_hue() {
        // given
        let theme = Theme::of(Appearance::Dark);
        let worktree = checkout("/r/.claude/worktrees/GEN-817-ive", false);

        // when
        let colours = [checkout_colour(&checkout("/r", true), theme), checkout_colour(&worktree, theme), checkout_colour(&worktree, theme)];

        // then
        assert_eq!(colours[0], theme.accent);
        assert!(theme.checkout_hues.contains(&colours[1]));
        assert_eq!(colours[1], colours[2]);
    }
}
