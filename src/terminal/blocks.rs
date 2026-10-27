//! Block elements (U+2580–U+259F) drawn as rectangles filling their cell, as WezTerm and Alacritty draw them: a font's
//! glyphs are shorter than a terminal line, so blocks drawn from it leave gaps between lines.

/// A part of a cell as (left, top, right, bottom) fractions of its width and height.
pub type Part = [f32; 4];

const UPPER_LEFT: Part = [0., 0., 0.5, 0.5];
const UPPER_RIGHT: Part = [0.5, 0., 1., 0.5];
const LOWER_LEFT: Part = [0., 0.5, 0.5, 1.];
const LOWER_RIGHT: Part = [0.5, 0.5, 1., 1.];
const UPPER_HALF: Part = [0., 0., 1., 0.5];
const LEFT_HALF: Part = [0., 0., 0.5, 1.];
const RIGHT_HALF: Part = [0.5, 0., 1., 1.];

/// U+2581..=U+2588: the lower 1/8 to 8/8 of a cell.
static LOWER_EIGHTHS: [Part; 8] = [
    [0., 0.875, 1., 1.],
    [0., 0.75, 1., 1.],
    [0., 0.625, 1., 1.],
    [0., 0.5, 1., 1.],
    [0., 0.375, 1., 1.],
    [0., 0.25, 1., 1.],
    [0., 0.125, 1., 1.],
    [0., 0., 1., 1.],
];
/// U+2589..=U+258F: the left 7/8 to 1/8 of a cell.
static LEFT_EIGHTHS: [Part; 7] = [
    [0., 0., 0.875, 1.],
    [0., 0., 0.75, 1.],
    [0., 0., 0.625, 1.],
    [0., 0., 0.5, 1.],
    [0., 0., 0.375, 1.],
    [0., 0., 0.25, 1.],
    [0., 0., 0.125, 1.],
];

/// The parts of its cell `character` covers; None for characters drawn from the font, shades among them.
pub fn block_parts(character: char) -> Option<&'static [Part]> {
    let parts: &'static [Part] = match character {
        '▀' => &[UPPER_HALF],
        '▁'..='█' => std::slice::from_ref(&LOWER_EIGHTHS[character as usize - 0x2581]),
        '▉'..='▏' => std::slice::from_ref(&LEFT_EIGHTHS[character as usize - 0x2589]),
        '▐' => &[RIGHT_HALF],
        '▔' => &[[0., 0., 1., 0.125]],
        '▕' => &[[0.875, 0., 1., 1.]],
        '▖' => &[LOWER_LEFT],
        '▗' => &[LOWER_RIGHT],
        '▘' => &[UPPER_LEFT],
        '▙' => &[LEFT_HALF, LOWER_RIGHT],
        '▚' => &[UPPER_LEFT, LOWER_RIGHT],
        '▛' => &[UPPER_HALF, LOWER_LEFT],
        '▜' => &[UPPER_HALF, LOWER_RIGHT],
        '▝' => &[UPPER_RIGHT],
        '▞' => &[UPPER_RIGHT, LOWER_LEFT],
        '▟' => &[RIGHT_HALF, LOWER_LEFT],
        _ => return None,
    };
    Some(parts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_cover_cells_as_block_elements_draw() {
        // given
        let characters = ['█', '▄', '▉', '▏', '▛', '░', 'a'];

        // when
        let parts: Vec<_> = characters.iter().map(|character| block_parts(*character)).collect();

        // then
        let expected: [Option<&[Part]>; 7] = [
            Some(&[[0., 0., 1., 1.]]),
            Some(&[[0., 0.5, 1., 1.]]),
            Some(&[[0., 0., 0.875, 1.]]),
            Some(&[[0., 0., 0.125, 1.]]),
            Some(&[UPPER_HALF, LOWER_LEFT]),
            None,
            None,
        ];
        assert_eq!(parts, expected);
    }
}
