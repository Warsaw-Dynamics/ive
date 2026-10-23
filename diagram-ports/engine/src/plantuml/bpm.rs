//! Immutable legacy BPM instructions and explicit flow graph; MIT source traces in provenance.
use super::{
    budget::Budget,
    parser::{bpm::Storage, error::ParseFailure},
};
use std::ops::Range;

#[derive(Debug)]
pub(super) struct Bpm {
    pub instructions: Vec<Instruction>,
    pub blocks: Vec<Block>,
    pub nodes: Vec<FlowNode>,
    pub edges: Vec<FlowEdge>,
    pub root: usize,
}
#[derive(Debug)]
pub(super) struct Block {
    pub instructions: Vec<usize>,
    pub span: Range<usize>,
}
#[derive(Debug)]
pub(super) struct Instruction {
    pub kind: InstructionKind,
    pub span: Range<usize>,
}
#[derive(Debug)]
pub(super) enum InstructionKind {
    Event(usize),
    Branch {
        entry: usize,
        exit: usize,
        arms: Vec<usize>,
    },
}
impl Instruction {
    pub fn ports(&self) -> (usize, usize) {
        match self.kind {
            InstructionKind::Event(id) => (id, id),
            InstructionKind::Branch { entry, exit, .. } => (entry, exit),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Owner {
    Document,
    Branch(usize),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum NodeKind {
    Start,
    Event,
    Merge(MergeKind),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MergeKind {
    Split,
    Join,
}
#[derive(Debug)]
pub(super) struct FlowNode {
    pub kind: NodeKind,
    pub label: String,
    pub owner: Owner,
    pub span: Range<usize>,
}
#[derive(Clone, Copy, Debug)]
pub(super) enum EdgeKind {
    Next,
    ArmEnter,
    ArmExit,
}
#[derive(Debug)]
pub(super) struct FlowEdge {
    pub from: usize,
    pub to: usize,
    pub owner: Owner,
    pub kind: EdgeKind,
}
#[derive(Clone, Copy)]
enum Exit {
    Root,
    Join(usize),
}
struct Task {
    block: usize,
    tail: usize,
    exit: Exit,
    owner: Owner,
}
pub(super) struct Builder<'a> {
    pub model: Bpm,
    pub storage: Storage,
    pub budget: &'a mut Budget,
}
impl<'a> Builder<'a> {
    pub fn new(storage: Storage, budget: &'a mut Budget) -> Result<Self, ParseFailure> {
        let mut builder = Self {
            model: Bpm {
                instructions: Vec::new(),
                blocks: Vec::new(),
                nodes: Vec::new(),
                edges: Vec::new(),
                root: 0,
            },
            storage,
            budget,
        };
        builder.node(NodeKind::Start, String::new(), 0..0)?;
        Ok(builder)
    }
    fn node(
        &mut self,
        kind: NodeKind,
        label: String,
        span: Range<usize>,
    ) -> Result<usize, ParseFailure> {
        self.budget.item()?;
        let id = self.model.nodes.len();
        self.storage.push(
            &mut self.model.nodes,
            FlowNode {
                kind,
                label,
                span,
                owner: Owner::Document,
            },
            self.budget,
        )?;
        Ok(id)
    }
    pub fn append(
        &mut self,
        mut values: Vec<usize>,
        id: usize,
    ) -> Result<Vec<usize>, ParseFailure> {
        self.budget.item()?;
        self.storage.push(&mut values, id, self.budget)?;
        Ok(values)
    }
    pub fn block(
        &mut self,
        span: Range<usize>,
        instructions: Vec<usize>,
    ) -> Result<usize, ParseFailure> {
        self.budget.item()?;
        let id = self.model.blocks.len();
        self.storage.push(
            &mut self.model.blocks,
            Block { instructions, span },
            self.budget,
        )?;
        Ok(id)
    }
    fn instruction(
        &mut self,
        span: Range<usize>,
        kind: InstructionKind,
    ) -> Result<usize, ParseFailure> {
        self.budget.item()?;
        let id = self.model.instructions.len();
        self.storage.push(
            &mut self.model.instructions,
            Instruction { kind, span },
            self.budget,
        )?;
        Ok(id)
    }
    pub fn event(&mut self, span: Range<usize>, raw: &str) -> Result<usize, ParseFailure> {
        super::text::validate_label(raw, 1)?;
        self.budget.item()?;
        self.budget.text(raw.len())?;
        let label = raw.to_owned();
        let id = self.node(NodeKind::Event, label, span.clone())?;
        self.instruction(span, InstructionKind::Event(id))
    }
    pub fn branch(&mut self, span: Range<usize>, arms: Vec<usize>) -> Result<usize, ParseFailure> {
        if arms.len() < 2 {
            return Err(ParseFailure::unsupported(
                span,
                "legacy BPM single-arm branch has unverified exit semantics",
            ));
        }
        for &arm in &arms {
            self.budget.charge(1)?;
            if self.model.blocks[arm].instructions.is_empty() {
                return Err(ParseFailure::unsupported(
                    self.model.blocks[arm].span.clone(),
                    "empty legacy BPM branch arm",
                ));
            }
        }
        let entry = self.node(
            NodeKind::Merge(MergeKind::Split),
            String::new(),
            span.clone(),
        )?;
        let exit = self.node(
            NodeKind::Merge(MergeKind::Join),
            String::new(),
            span.clone(),
        )?;
        self.instruction(span, InstructionKind::Branch { entry, exit, arms })
    }
    fn edge(
        &mut self,
        from: usize,
        to: usize,
        owner: Owner,
        kind: EdgeKind,
    ) -> Result<(), ParseFailure> {
        self.budget.item()?;
        self.storage.push(
            &mut self.model.edges,
            FlowEdge {
                from,
                to,
                owner,
                kind,
            },
            self.budget,
        )
    }
    pub fn finish(mut self, root: usize) -> Result<Bpm, ParseFailure> {
        self.model.root = root;
        let mut pending = Vec::new();
        self.storage.push(
            &mut pending,
            Task {
                block: root,
                tail: 0,
                exit: Exit::Root,
                owner: Owner::Document,
            },
            self.budget,
        )?;
        while let Some(task) = pending.pop() {
            self.lower(task, &mut pending)?;
        }
        Ok(self.model)
    }
    fn lower(&mut self, task: Task, pending: &mut Vec<Task>) -> Result<(), ParseFailure> {
        let mut tail = task.tail;
        for index in 0..self.model.blocks[task.block].instructions.len() {
            self.budget.charge(1)?;
            let id = self.model.blocks[task.block].instructions[index];
            let (entry, exit) = self.model.instructions[id].ports();
            let kind = if index == 0 && matches!(task.exit, Exit::Join(_)) {
                EdgeKind::ArmEnter
            } else {
                EdgeKind::Next
            };
            self.edge(tail, entry, task.owner, kind)?;
            self.model.nodes[entry].owner = task.owner;
            self.model.nodes[exit].owner = task.owner;
            if let InstructionKind::Branch { ref arms, .. } = self.model.instructions[id].kind {
                for &block in arms.iter().rev() {
                    self.storage.push(
                        pending,
                        Task {
                            block,
                            tail: entry,
                            exit: Exit::Join(exit),
                            owner: Owner::Branch(id),
                        },
                        self.budget,
                    )?;
                }
            }
            tail = exit;
        }
        if let Exit::Join(to) = task.exit {
            self.edge(tail, to, task.owner, EdgeKind::ArmExit)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RenderOptions, plantuml::parser};
    fn parse(body: &str) -> Bpm {
        let mut budget = Budget::new(&RenderOptions::default());
        parser::test_support::body((body, "bpm"), &mut budget, parser::bpm::parse).unwrap()
    }
    #[test]
    fn should_lower_implicit_start_events_and_two_synthetic_branch_merges() {
        // given
        let body = ":Receive;\nnew branch\n:Check;\nelse\n:Retry;\nend branch\n:Archive;";
        // when
        let model = parse(body);
        // then
        assert_eq!(model.nodes[0].kind, NodeKind::Start);
        let node = |name| model.nodes.iter().position(|n| n.label == name).unwrap();
        let split = model
            .nodes
            .iter()
            .position(|n| n.kind == NodeKind::Merge(MergeKind::Split))
            .unwrap();
        let merge = model
            .nodes
            .iter()
            .position(|n| n.kind == NodeKind::Merge(MergeKind::Join))
            .unwrap();
        let edges = model
            .edges
            .iter()
            .map(|e| (e.from, e.to))
            .collect::<Vec<_>>();
        assert_eq!(model.nodes.len(), 7);
        assert_eq!(model.edges.len(), 7);
        for edge in [
            (0, node("Receive")),
            (node("Receive"), split),
            (split, node("Check")),
            (split, node("Retry")),
            (node("Check"), merge),
            (node("Retry"), merge),
            (merge, node("Archive")),
        ] {
            assert!(edges.contains(&edge));
        }
        assert!(
            model
                .nodes
                .iter()
                .filter(|n| n.kind == NodeKind::Event)
                .all(|n| n.span.end > n.span.start)
        );
    }
    #[test]
    fn should_keep_nested_instruction_arms_as_ids_and_distinct_owned_graph_edges() {
        // given
        let body = "new branch\nnew branch\n:A;\nelse\n:B;\nend branch\nelse\n:C;\nelse\n:D;\nend branch\n:E;";
        // when
        let model = parse(body);
        // then
        let branches = model
            .instructions
            .iter()
            .enumerate()
            .filter_map(|(i, n)| {
                if let InstructionKind::Branch { arms, .. } = &n.kind {
                    Some((i, arms))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        assert_eq!(branches.len(), 2);
        assert_eq!(branches[0].1.len(), 2);
        assert_eq!(branches[1].1.len(), 3);
        for (id, arms) in branches {
            assert!(model.edges.iter().any(|e| e.owner == Owner::Branch(id)));
            assert!(arms.iter().all(|arm| *arm < model.root));
        }
        assert_eq!(
            model
                .nodes
                .iter()
                .filter(|n| n.kind == NodeKind::Merge(MergeKind::Join))
                .count(),
            2
        );
    }
    #[test]
    fn should_reject_empty_document_under_shared_nonempty_source_admission() {
        // given
        let body = "\n";
        // when
        let mut budget = Budget::new(&RenderOptions::default());
        let result = parser::test_support::body((body, "bpm"), &mut budget, parser::bpm::parse);
        // then
        assert!(
            matches!(result,Err(crate::RenderError::InvalidSource(ref detail)) if detail.contains("nonempty complete PlantUML block"))
        );
    }
}
