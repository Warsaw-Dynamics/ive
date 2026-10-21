use alacritty_terminal::term::color::Colors;
use alacritty_terminal::vte::ansi::{Color, NamedColor, Rgb};

use crate::theme::TerminalPalette;

/// Channel levels of the xterm 6×6×6 colour cube (indices 16–231).
const CUBE_LEVELS: [u32; 6] = [0x00, 0x5f, 0x87, 0xaf, 0xd7, 0xff];
const FIRST_CUBE_INDEX: usize = 16;
const FIRST_GRAY_INDEX: usize = 232;

/// Resolves cell colours against the theme palette and the colours the program changed (OSC 4/10/11).
#[derive(Clone, Copy)]
pub struct ColorResolver<'a> {
    pub palette: &'a TerminalPalette,
    pub overrides: &'a Colors,
}

impl ColorResolver<'_> {
    /// The colour as 0xRRGGBB.
    pub fn resolve(&self, color: Color) -> u32 {
        match color {
            Color::Spec(rgb) => pack(rgb),
            Color::Indexed(index) => self.indexed(usize::from(index)),
            Color::Named(name) => self.named(name),
        }
    }

    /// Colour `index` of the 256-colour table; 256 and up are the named foreground, background and cursor.
    pub fn indexed(&self, index: usize) -> u32 {
        if let Some(rgb) = self.overrides[index] {
            return pack(rgb);
        }
        match index {
            0..FIRST_CUBE_INDEX => self.palette.ansi[index],
            FIRST_CUBE_INDEX..FIRST_GRAY_INDEX => cube(index - FIRST_CUBE_INDEX),
            FIRST_GRAY_INDEX..256 => gray(index - FIRST_GRAY_INDEX),
            _ => self.special(index),
        }
    }

    fn named(&self, name: NamedColor) -> u32 {
        self.indexed(name as usize)
    }

    /// Foreground, background, cursor and the dim variants, indices 256 and up in alacritty's table.
    fn special(&self, index: usize) -> u32 {
        let palette = self.palette;
        match index {
            i if i == NamedColor::Background as usize => palette.background,
            i if i == NamedColor::Cursor as usize => palette.cursor,
            i if i == NamedColor::DimForeground as usize => dim(palette.foreground),
            i if (NamedColor::DimBlack as usize..=NamedColor::DimWhite as usize).contains(&i) => {
                dim(palette.ansi[i - NamedColor::DimBlack as usize])
            }
            _ => palette.foreground,
        }
    }
}

/// Darkens a colour to two thirds of its brightness, how the dim (faint) attribute is drawn.
pub fn dim(color: u32) -> u32 {
    let channel = |shift: u32| ((color >> shift & 0xff) * 2 / 3) << shift;
    channel(16) | channel(8) | channel(0)
}

fn cube(offset: usize) -> u32 {
    let level = |digit: usize| CUBE_LEVELS[digit % 6];
    level(offset / 36) << 16 | level(offset / 6) << 8 | level(offset)
}

fn gray(offset: usize) -> u32 {
    let level = 8 + 10 * offset as u32;
    level << 16 | level << 8 | level
}

/// The 0xRRGGBB colour as alacritty's `Rgb`.
pub fn unpack(color: u32) -> Rgb {
    Rgb { r: (color >> 16) as u8, g: (color >> 8) as u8, b: color as u8 }
}

fn pack(rgb: Rgb) -> u32 {
    u32::from(rgb.r) << 16 | u32::from(rgb.g) << 8 | u32::from(rgb.b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Appearance, Theme};

    fn resolve_all(colors: &[Color], overrides: &Colors) -> Vec<u32> {
        let palette = Theme::of(Appearance::Dark).terminal;
        let resolver = ColorResolver { palette: &palette, overrides };
        colors.iter().map(|color| resolver.resolve(*color)).collect()
    }

    #[test]
    fn should_resolve_xterm_cube_and_gray_ramp() {
        // given
        let colors = [Color::Indexed(16), Color::Indexed(196), Color::Indexed(231), Color::Indexed(232), Color::Indexed(255)];

        // when
        let resolved = resolve_all(&colors, &Colors::default());

        // then
        assert_eq!(resolved, [0x000000, 0xff0000, 0xffffff, 0x080808, 0xeeeeee]);
    }

    #[test]
    fn should_take_ansi_and_special_colors_from_palette() {
        // given
        let palette = Theme::of(Appearance::Dark).terminal;
        let colors = [
            Color::Named(NamedColor::Red),
            Color::Indexed(9),
            Color::Named(NamedColor::Foreground),
            Color::Named(NamedColor::Background),
            Color::Named(NamedColor::DimRed),
        ];

        // when
        let resolved = resolve_all(&colors, &Colors::default());

        // then
        let expected = [palette.ansi[1], palette.ansi[9], palette.foreground, palette.background, dim(palette.ansi[1])];
        assert_eq!(resolved, expected);
    }

    #[test]
    fn should_prefer_colors_the_program_changed() {
        // given
        let mut overrides = Colors::default();
        overrides[1] = Some(Rgb { r: 1, g: 2, b: 3 });
        overrides[NamedColor::Background] = Some(Rgb { r: 4, g: 5, b: 6 });

        // when
        let resolved = resolve_all(&[Color::Named(NamedColor::Red), Color::Named(NamedColor::Background)], &overrides);

        // then
        assert_eq!(resolved, [0x010203, 0x040506]);
    }
}
