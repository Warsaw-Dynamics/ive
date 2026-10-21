//! Named click targets for MCP's `click`. gpui 0.2.2 offers no way to inject a mouse click
//! (`Window::dispatch_event` returns a crate-private type, and its X11 backend reads XInput2 events, which a
//! client cannot send), so the elements agents need register their click handler under a stable name while
//! they draw, and `click { target }` calls it. Only what is on screen is registered.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use gpui::{App, ClickEvent, Global, Window};

/// A click handler that can be shared: by an element and by its target.
pub type Listener = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// The targets of the frame being drawn; a gpui global.
#[derive(Default)]
pub struct ClickTargets {
    frame: RefCell<BTreeMap<String, Listener>>,
}

impl Global for ClickTargets {}

impl ClickTargets {
    /// Forgets the last frame's targets; the workspace calls it as it starts drawing.
    pub fn clear(&self) {
        self.frame.borrow_mut().clear();
    }

    /// The first registration of a name in a frame wins: lists draw some rows twice (to measure them).
    pub fn register(&self, name: String, listener: Listener) {
        self.frame.borrow_mut().entry(name).or_insert(listener);
    }

    pub fn get(&self, name: &str) -> Option<Listener> {
        self.frame.borrow().get(name).cloned()
    }

    /// In name order.
    pub fn names(&self) -> Vec<String> {
        self.frame.borrow().keys().cloned().collect()
    }
}

/// Registers `listener` as target `name` and returns it, for `.on_click(…)`.
pub fn target(
    name: String,
    listener: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    cx: &App,
) -> impl Fn(&ClickEvent, &mut Window, &mut App) + 'static {
    let listener: Listener = Rc::new(listener);
    if let Some(targets) = cx.try_global::<ClickTargets>() {
        targets.register(name, listener.clone());
    }
    move |event, window, cx| listener(event, window, cx)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nothing() -> Listener {
        Rc::new(|_, _, _| {})
    }

    #[test]
    fn should_keep_the_first_listener_registered_under_a_name_in_a_frame() {
        // given
        let targets = ClickTargets::default();
        let (first, second) = (nothing(), nothing());

        // when
        targets.register("card:dev".to_owned(), first.clone());
        targets.register("card:dev".to_owned(), second.clone());

        // then
        let kept = targets.get("card:dev").unwrap();
        assert!(Rc::ptr_eq(&kept, &first) && !Rc::ptr_eq(&kept, &second));
    }

    #[test]
    fn should_list_each_name_once_and_forget_them_with_the_frame() {
        // given
        let targets = ClickTargets::default();
        targets.register("section:Worktrees".to_owned(), nothing());
        targets.register("card:dev".to_owned(), nothing());
        targets.register("card:dev".to_owned(), nothing());

        // when
        let names = targets.names();
        targets.clear();

        // then
        assert_eq!(names, ["card:dev", "section:Worktrees"]);
        assert!(targets.get("card:dev").is_none());
    }
}
