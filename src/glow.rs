//! IntelliJ Islands' project glow: the frame behind rounded panels lit at the top left in the colour of the worktree
//! being worked in.

use std::path::Path;

use gpui::{Div, Hsla, div, linear_color_stop, linear_gradient, prelude::*, px, rgb};

use crate::colors::fnv1a;
use crate::theme::Theme;

/// Which of the theme's glow colours the worktree at `checkout` gets: the same one every run.
pub fn glow_index(checkout: &Path) -> usize {
    (fnv1a(checkout.as_os_str().as_encoded_bytes()) % 9) as usize
}

/// The glow colours of `checkouts`, in order: each its own `glow_index` unless an earlier one took it, then the next
/// free one, so up to nine worktrees never share a colour.
pub fn glow_indices(checkouts: &[&Path]) -> Vec<usize> {
    let mut taken = [false; 9];
    checkouts
        .iter()
        .map(|checkout| {
            let wanted = glow_index(checkout);
            let index = (wanted..wanted + 9).map(|index| index % 9).find(|index| !taken[*index]).unwrap_or(wanted);
            taken[index] = true;
            if taken.iter().all(|taken| *taken) {
                taken = [false; 9];
            }
            index
        })
        .collect()
}

/// `frame` moved toward `project` by `blend` (0 keeps the frame colour, 1 is the project colour), per channel.
pub fn glow_colour(frame: u32, project: u32, blend: f32) -> u32 {
    let channel = |shift: u32| {
        let (from, to) = (((frame >> shift) & 0xff) as f32, ((project >> shift) & 0xff) as f32);
        ((from + (to - from) * blend).round() as u32) << shift
    };
    channel(16) | channel(8) | channel(0)
}

/// The glow over the frame's top left in the theme's glow colour `index`: the frame colour rising to the lit colour
/// at `glow_peak`, falling back over `glow_width`, the whole faded into the frame colour over `glow_height`.
pub fn frame_glow(theme: Theme, index: usize) -> Div {
    let panels = theme.panels;
    let (peak, width) = (panels.glow_peak.0, panels.glow_width.0);
    let lit: Hsla = rgb(glow_colour(theme.frame, theme.glow[index % theme.glow.len()], panels.glow_blend.0)).into();
    let frame: Hsla = rgb(theme.frame).into();
    let clear = Hsla { a: 0., ..frame };
    let rising = div().absolute().top_0().left_0().w(px(peak)).h_full().bg(linear_gradient(90., linear_color_stop(frame, 0.), linear_color_stop(lit, 1.)));
    let falling = div().absolute().top_0().left(px(peak)).w(px(width)).h_full().bg(linear_gradient(90., linear_color_stop(lit, 0.), linear_color_stop(frame, 1.)));
    let fading = div().absolute().top_0().left_0().size_full().bg(linear_gradient(180., linear_color_stop(clear, 0.), linear_color_stop(frame, 1.)));
    div().absolute().top_0().left_0().w(px(peak + width)).h(panels.glow_height.px()).child(rising).child(falling).child(fading)
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::*;

    #[test]
    fn should_pick_same_glow_for_same_worktree() {
        // given
        let (dev, ive) = (Path::new("/p/atlassian"), Path::new("/p/atlassian/.claude/worktrees/GEN-817-ive"));

        // when
        let (first, again, other) = (glow_index(dev), glow_index(dev), glow_index(ive));

        // then
        assert_eq!(first, again);
        assert_ne!(first, other);
        assert!(other < 9);
    }

    #[test]
    fn should_give_worktrees_whose_colours_clash_the_next_free_one() {
        // given: both hash to colour 4
        let (reference, configs) = (Path::new("/mnt/data/Projects/atlassian-wt/GEN-811-paas-reference"), Path::new("/mnt/data/Projects/atlassian/.claude/worktrees/GEN-817-ive-run-configs"));

        // when
        let indices = glow_indices(&[reference, configs]);

        // then
        assert_eq!((glow_index(reference), glow_index(configs)), (4, 4));
        assert_eq!(indices, [4, 5]);
    }

    #[test]
    fn should_repeat_colours_past_nine_worktrees() {
        // given
        let paths: Vec<PathBuf> = (0..10).map(|n| PathBuf::from(format!("/w/{n}"))).collect();

        // when
        let indices = glow_indices(&paths.iter().map(PathBuf::as_path).collect::<Vec<_>>());

        // then
        let mut distinct = indices[..9].to_vec();
        distinct.sort_unstable();
        assert_eq!(distinct, (0..9).collect::<Vec<_>>());
        assert!(indices[9] < 9);
    }

    #[test]
    fn should_blend_glow_toward_project_colour() {
        // when
        let blended = glow_colour(0x26282c, 0x534c33, 0.85);

        // then
        assert_eq!(blended, 0x4c4732);
    }

    #[test]
    fn should_keep_frame_colour_without_blend() {
        // when
        let blended = glow_colour(0x26282c, 0x534c33, 0.);

        // then
        assert_eq!(blended, 0x26282c);
    }
}
