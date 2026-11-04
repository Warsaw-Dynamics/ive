//! The checkout rows `config.worktree_row` built, by what they were built from, and which rows to ask for next.

use std::collections::{HashMap, HashSet};

use crate::script::worktree_row::{RowFunction, RowInput, RowLook};

#[derive(Debug, Default)]
pub struct CheckoutLooks {
    /// the load's function; none leaves every row to IVE
    function: Option<RowFunction>,
    asked: HashSet<RowInput>,
    built: HashMap<RowInput, RowLook>,
}

impl CheckoutLooks {
    /// Takes a load's function; rows built by another are dropped. Returns whether it changed.
    pub fn use_function(&mut self, function: Option<RowFunction>) -> bool {
        if self.function == function {
            return false;
        }
        *self = Self { function, ..Self::default() };
        true
    }

    /// The rows of `current`, the checkouts the tree has now, that were not asked for yet, with the function to
    /// build them; rows of checkouts gone or changed are forgotten.
    pub fn take_unasked(&mut self, current: Vec<RowInput>) -> Option<(RowFunction, Vec<RowInput>)> {
        let function = self.function?;
        let now: HashSet<&RowInput> = current.iter().collect();
        self.asked.retain(|input| now.contains(input));
        self.built.retain(|input, _| now.contains(input));
        let new: Vec<RowInput> = current.iter().filter(|input| self.asked.insert((*input).clone())).cloned().collect();
        if new.is_empty() {
            return None;
        }
        Some((function, new))
    }

    /// Keeps rows load `generation` built; those of an older load, or that failed, are dropped. Returns whether any
    /// was kept.
    pub fn take_built(&mut self, generation: u64, looks: Vec<(RowInput, Option<RowLook>)>) -> bool {
        if self.function.map(|function| function.generation) != Some(generation) {
            return false;
        }
        let before = self.built.len();
        self.built.extend(looks.into_iter().filter_map(|(input, look)| look.map(|look| (input, look))));
        self.built.len() != before
    }

    pub fn built(&self) -> &HashMap<RowInput, RowLook> {
        &self.built
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::script::worktree_row::default_look;

    fn input(folder: &str) -> RowInput {
        RowInput {
            project: "p".to_owned(),
            project_dir: "/p".to_owned(),
            path: format!("/p/{folder}"),
            folder: folder.to_owned(),
            label: folder.to_owned(),
            branch: None,
            main: false,
            head: None,
            base: None,
        }
    }

    const FUNCTION: RowFunction = RowFunction { generation: 2 };

    #[test]
    fn should_ask_for_each_row_once() {
        // given
        let mut looks = CheckoutLooks::default();
        looks.use_function(Some(FUNCTION));
        looks.take_unasked(vec![input("a")]);

        // when
        let asked = looks.take_unasked(vec![input("a"), input("b")]);

        // then
        assert_eq!(asked, Some((FUNCTION, vec![input("b")])));
    }

    #[test]
    fn should_ask_for_nothing_without_a_function() {
        // given
        let mut looks = CheckoutLooks::default();

        // when
        let asked = looks.take_unasked(vec![input("a")]);

        // then
        assert_eq!(asked, None);
    }

    #[test]
    fn should_drop_rows_built_by_an_older_load() {
        // given
        let mut looks = CheckoutLooks::default();
        looks.use_function(Some(FUNCTION));
        looks.take_unasked(vec![input("a")]);

        // when
        let kept = looks.take_built(1, vec![(input("a"), Some(default_look(&input("a"))))]);

        // then
        assert!(!kept);
        assert!(looks.built().is_empty());
    }

    #[test]
    fn should_forget_a_row_once_its_checkout_changed() {
        // given
        let mut looks = CheckoutLooks::default();
        looks.use_function(Some(FUNCTION));
        looks.take_unasked(vec![input("a")]);
        looks.take_built(2, vec![(input("a"), Some(default_look(&input("a"))))]);
        let renamed = RowInput { branch: Some("x".to_owned()), ..input("a") };

        // when
        let asked = looks.take_unasked(vec![renamed.clone()]);

        // then
        assert_eq!(asked, Some((FUNCTION, vec![renamed])));
        assert!(looks.built().is_empty());
    }
}
