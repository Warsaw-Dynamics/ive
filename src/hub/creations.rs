//! The worktrees the new terminal picker is creating, so a branch is not created twice, whichever hub tab asks.

use std::collections::HashSet;

use crate::project::ProjectId;

#[derive(Debug, Default)]
pub struct Creations(HashSet<(ProjectId, String)>);

impl Creations {
    /// Whether a creation of `branch` may start: none of it runs yet.
    pub fn start(&mut self, project: &ProjectId, branch: &str) -> bool {
        self.0.insert((project.clone(), branch.to_owned()))
    }

    pub fn finish(&mut self, project: &ProjectId, branch: &str) {
        self.0.remove(&(project.clone(), branch.to_owned()));
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn demo() -> ProjectId {
        ProjectId::parse("demo").unwrap()
    }

    #[test]
    fn should_refuse_a_second_creation_of_the_same_branch() {
        // given
        let mut creations = Creations::default();
        creations.start(&demo(), "topic");

        // when
        let again = creations.start(&demo(), "topic");

        // then
        assert_eq!((again, creations.start(&demo(), "other")), (false, true));
    }

    #[test]
    fn should_allow_a_branch_again_once_its_creation_finished() {
        // given
        let mut creations = Creations::default();
        creations.start(&demo(), "topic");

        // when
        creations.finish(&demo(), "topic");

        // then
        assert_eq!((creations.is_empty(), creations.start(&demo(), "topic")), (true, true));
    }
}
