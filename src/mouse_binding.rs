//! Mouse bindings on elements: gpui's mouse events as the keymap's [`MouseTrigger`]s, and the element a press began
//! on, so an `Up` binding fires only where its press began (as a click does).

use std::rc::Rc;

use gpui::{App, Global, InteractiveElement, Modifiers, MouseButton as GpuiButton, Window};

use crate::script::keymap::{MouseButton, MouseKind, MouseTrigger};

/// The trigger of a gpui mouse event; `None` for the back and forward buttons, which bindings cannot name.
pub fn trigger(kind: MouseKind, button: GpuiButton, click_count: usize, mods: Modifiers, mouse_reporting: bool) -> Option<MouseTrigger> {
    let button = match button {
        GpuiButton::Left => MouseButton::Left,
        GpuiButton::Middle => MouseButton::Middle,
        GpuiButton::Right => MouseButton::Right,
        GpuiButton::Navigate(_) => return None,
    };
    let streak = click_count.clamp(1, 3) as u8;
    Some(MouseTrigger { kind, button, streak, mods, mouse_reporting })
}

/// The element (by key) and button of the last press on an element with bindings.
#[derive(Default)]
struct Pressed(Option<(String, MouseButton)>);

impl Global for Pressed {}

/// `element` with its presses and releases of the left, middle and right button reported to `on` as `Down` and `Up`
/// triggers; `key` names the element, and a release counts only on the element its press began on. A press stops
/// there: the elements around it (the title bar's window drag) do not see it.
pub fn on_mouse_triggers<E: InteractiveElement>(mut element: E, key: String, on: impl Fn(MouseTrigger, &mut Window, &mut App) + 'static) -> E {
    let on = Rc::new(on);
    for button in [GpuiButton::Left, GpuiButton::Middle, GpuiButton::Right] {
        let (on_down, pressed_key) = (on.clone(), key.clone());
        element = element.on_mouse_down(button, move |event, window, cx| {
            cx.stop_propagation();
            let Some(down) = trigger(MouseKind::Down, event.button, event.click_count, event.modifiers, false) else {
                return;
            };
            cx.default_global::<Pressed>().0 = Some((pressed_key.clone(), down.button));
            on_down(down, window, cx);
        });
        let (on_up, released_key) = (on.clone(), key.clone());
        element = element.on_mouse_up(button, move |event, window, cx| {
            let pressed = cx.default_global::<Pressed>().0.take();
            let Some(up) = trigger(MouseKind::Up, event.button, event.click_count, event.modifiers, false) else {
                return;
            };
            if pressed == Some((released_key.clone(), up.button)) {
                on_up(up, window, cx);
            }
        });
    }
    element
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_build_trigger_with_streak() {
        // when
        let built = trigger(MouseKind::Up, GpuiButton::Middle, 2, Modifiers { control: true, ..Modifiers::default() }, false);

        // then
        let expected = MouseTrigger { kind: MouseKind::Up, button: MouseButton::Middle, streak: 2, mods: Modifiers { control: true, ..Modifiers::default() }, mouse_reporting: false };
        assert_eq!(built, Some(expected));
    }

    #[test]
    fn should_clamp_streak_to_three() {
        // when
        let streaks = [0, 1, 3, 7].map(|clicks| trigger(MouseKind::Down, GpuiButton::Left, clicks, Modifiers::default(), false).map(|built| built.streak));

        // then
        assert_eq!(streaks, [Some(1), Some(1), Some(3), Some(3)]);
    }

    #[test]
    fn should_ignore_navigate_buttons() {
        // when
        let built = trigger(MouseKind::Up, GpuiButton::Navigate(gpui::NavigationDirection::Back), 1, Modifiers::default(), false);

        // then
        assert_eq!(built, None);
    }
}
