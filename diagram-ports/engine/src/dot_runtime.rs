//! Bounded lowering and rendering for user-supplied DOT.
mod font_metrics;
mod gd_raster;
mod html_table;
mod svg_output;
use crate::dot::{
    self, Attribute, AttributeScope, Document, EdgeOp, Endpoint, GraphKind, IdKind, NodeId,
    Statement, Subgraph,
};
use crate::plantuml::layered::{self, LayoutGraph, LayoutWork};
use crate::{
    RenderControl, RenderError, RenderLimits, RenderOptions, RenderedDiagram, TextMeasurementPhase,
    TextStyle,
};
use dugong::{
    EdgeLabel, GraphLabel, NodeLabel, RankDir,
    graphlib::{Graph, GraphOptions},
};
use merman_render::{
    environment::{RenderEnvironment, RenderSession},
    text::TextMeasurer,
};
use serde_json::{Map, Value, json};
use std::collections::{BTreeMap, HashMap, HashSet};

type Attributes = BTreeMap<String, String>;

#[derive(Clone)]
struct Defaults {
    graph: Attributes,
    node: Attributes,
    node_label_kind: IdKind,
    edge: Attributes,
    parent: String,
    subgraph: usize,
}
impl Default for Defaults {
    fn default() -> Self {
        Self {
            graph: Attributes::new(),
            node: Attributes::new(),
            node_label_kind: IdKind::Bare,
            edge: Attributes::new(),
            parent: String::new(),
            subgraph: usize::MAX,
        }
    }
}
struct Node {
    id: String,
    attrs: Attributes,
    label_kind: IdKind,
    parent: String,
}
struct RecordField {
    port: String,
    text: String,
    width: f64,
    height: f64,
    children: Vec<RecordField>,
    vertical: bool,
}
struct RecordSpec {
    root: RecordField,
    width: f64,
    height: f64,
}
struct RecordCell<'a> {
    port: &'a str,
    text: &'a str,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

struct RecordDivider {
    start: (f64, f64),
    end: (f64, f64),
}

enum RecordOperation<'a> {
    Text(RecordCell<'a>),
    Divider(RecordDivider),
}

impl RecordSpec {
    fn cells(&self) -> Vec<RecordCell<'_>> {
        self.operations()
            .into_iter()
            .filter_map(|operation| match operation {
                RecordOperation::Text(cell) => Some(cell),
                RecordOperation::Divider(_) => None,
            })
            .collect()
    }

    fn operations(&self) -> Vec<RecordOperation<'_>> {
        let mut operations = Vec::new();
        self.root
            .collect((0., 0.), (self.width, self.height - 1.), &mut operations);
        operations
    }
}

impl RecordField {
    fn leaf_count(&self) -> usize {
        if self.children.is_empty() {
            return 1;
        }
        self.children.iter().map(Self::leaf_count).sum()
    }

    fn collect<'a>(
        &'a self,
        center: (f64, f64),
        size: (f64, f64),
        output: &mut Vec<RecordOperation<'a>>,
    ) {
        if self.children.is_empty() {
            output.push(RecordOperation::Text(RecordCell {
                port: &self.port,
                text: &self.text,
                x: center.0,
                y: center.1,
                width: size.0,
                height: size.1,
            }));
            return;
        }
        let natural = self
            .children
            .iter()
            .map(|field| {
                if self.vertical {
                    field.height
                } else {
                    field.width
                }
            })
            .sum::<f64>();
        let available = if self.vertical { size.1 } else { size.0 };
        // Graphviz truncates cumulative resize increments at each child boundary.
        let increment = (available - natural) / self.children.len() as f64;
        let mut offset = -available / 2.;
        for (index, child) in self.children.iter().enumerate() {
            let adjustment =
                ((index + 1) as f64 * increment).trunc() - (index as f64 * increment).trunc();
            let extent = if self.vertical {
                child.height
            } else {
                child.width
            } + adjustment;
            if index > 0 {
                let divider = if self.vertical {
                    RecordDivider {
                        start: (center.0 - size.0 / 2., center.1 + offset),
                        end: (center.0 + size.0 / 2., center.1 + offset),
                    }
                } else {
                    RecordDivider {
                        start: (center.0 + offset, center.1 + size.1 / 2.),
                        end: (center.0 + offset, center.1 - size.1 / 2.),
                    }
                };
                output.push(RecordOperation::Divider(divider));
            }
            let child_center = if self.vertical {
                (center.0, center.1 + offset + extent / 2.)
            } else {
                (center.0 + offset + extent / 2., center.1)
            };
            let child_size = if self.vertical {
                (size.0, extent)
            } else {
                (extent, size.1)
            };
            child.collect(child_center, child_size, output);
            offset += extent;
        }
    }
}
struct Edge {
    from: String,
    to: String,
    attrs: Attributes,
}

#[derive(Clone, Copy)]
struct ArrowSpec {
    head: bool,
    tail: bool,
    head_shape: ArrowShape,
    tail_shape: ArrowShape,
    size: f64,
    penwidth: f64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ArrowShape {
    Normal,
    Vee,
    Diamond,
    Box,
    None,
}

impl ArrowShape {
    fn parse(value: &str) -> Option<Self> {
        // Graphviz arrows.c also composes up to four modified shapes. Keep
        // those outside this bounded renderer until their routes and paint match.
        match value {
            "normal" => Some(Self::Normal),
            "vee" => Some(Self::Vee),
            "diamond" => Some(Self::Diamond),
            "box" => Some(Self::Box),
            "none" => Some(Self::None),
            _ => None,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Vee => "vee",
            Self::Diamond => "diamond",
            Self::Box => "box",
            Self::None => "none",
        }
    }
}

fn arrow_spec(edge: &Edge, kind: GraphKind) -> Result<ArrowSpec, RenderError> {
    let direction =
        edge.attrs
            .get("dir")
            .map(String::as_str)
            .unwrap_or(if kind == GraphKind::Digraph {
                "forward"
            } else {
                "none"
            });
    if !matches!(direction, "forward" | "back" | "both" | "none") {
        return Err(RenderError::InvalidSource(format!(
            "DOT edge {:?}->{:?} dir {direction:?} is invalid",
            edge.from, edge.to
        )));
    }
    let head = edge
        .attrs
        .get("arrowhead")
        .map(String::as_str)
        .unwrap_or("normal");
    let tail = edge
        .attrs
        .get("arrowtail")
        .map(String::as_str)
        .unwrap_or("normal");
    let parse_shape = |name: &str, value: &str| {
        ArrowShape::parse(value).ok_or_else(|| {
            RenderError::UnsupportedFeature(format!(
                "DOT edge {:?}->{:?} {name} {value:?}",
                edge.from, edge.to
            ))
        })
    };
    // Graphviz arrow_flags ignores the shape attribute at an inactive end.
    let head_active = matches!(direction, "forward" | "both");
    let tail_active = matches!(direction, "back" | "both");
    let head_shape = if head_active {
        parse_shape("arrowhead", head)?
    } else {
        ArrowShape::None
    };
    let tail_shape = if tail_active {
        parse_shape("arrowtail", tail)?
    } else {
        ArrowShape::None
    };
    let size = edge.attrs.get("arrowsize").map_or(Ok(1.), |raw| {
        raw.parse::<f64>().map_err(|_| {
            RenderError::InvalidSource(format!(
                "DOT edge {:?}->{:?} arrowsize {raw:?} is invalid",
                edge.from, edge.to
            ))
        })
    })?;
    if !size.is_finite() || !(0.01..=100.).contains(&size) {
        return Err(RenderError::InvalidSource(format!(
            "DOT edge {:?}->{:?} arrowsize {size} is outside 0.01..=100",
            edge.from, edge.to
        )));
    }
    let penwidth = edge.attrs.get("penwidth").map_or(Ok(1.), |raw| {
        raw.parse::<f64>().map_err(|_| {
            RenderError::InvalidSource(format!(
                "DOT edge {:?}->{:?} penwidth {raw:?} is invalid",
                edge.from, edge.to
            ))
        })
    })?;
    if !penwidth.is_finite() || !(0. ..=4.).contains(&penwidth) {
        return Err(RenderError::UnsupportedFeature(format!(
            "DOT edge {:?}->{:?} penwidth {penwidth} is outside the bounded 0..=4 range",
            edge.from, edge.to
        )));
    }
    if penwidth != 1.
        && ((matches!(direction, "forward" | "both")
            && !matches!(head_shape, ArrowShape::Normal | ArrowShape::None))
            || (matches!(direction, "back" | "both")
                && !matches!(tail_shape, ArrowShape::Normal | ArrowShape::None)))
    {
        return Err(RenderError::UnsupportedFeature(format!(
            "DOT edge {:?}->{:?} penwidth {penwidth} with non-normal arrow geometry",
            edge.from, edge.to
        )));
    }
    Ok(ArrowSpec {
        head: head_active && head_shape != ArrowShape::None,
        tail: tail_active && tail_shape != ArrowShape::None,
        head_shape,
        tail_shape,
        size,
        penwidth,
    })
}

fn validate_extended_arrow(
    model: &Model,
    edge: &Edge,
    arrows: ArrowSpec,
) -> Result<(), RenderError> {
    let extended = (arrows.head && arrows.head_shape != ArrowShape::Normal)
        || (arrows.tail && arrows.tail_shape != ArrowShape::Normal)
        || ((arrows.head || arrows.tail) && (arrows.size != 1. || arrows.penwidth != 1.));
    if !extended {
        return Ok(());
    }
    let fixed_box = |id: &str| {
        let node = &model.nodes[model.index[id]];
        node.attrs.get("shape").is_some_and(|value| value == "box")
            && node
                .attrs
                .get("fixedsize")
                .is_some_and(|value| value == "true")
            && node.attrs.get("label").is_some_and(String::is_empty)
    };
    let supported = model.edges.len() == 1
        && model.kind == GraphKind::Digraph
        && model
            .graph_attrs
            .get("rankdir")
            .is_none_or(|value| value == "TB")
        && model
            .graph_attrs
            .get("splines")
            .is_some_and(|value| value == "polyline")
        && model.subgraphs.is_empty()
        && edge.from != edge.to
        && fixed_box(&edge.from)
        && fixed_box(&edge.to);
    if supported || is_default_ellipse_arrow_path(model) {
        return Ok(());
    }
    Err(RenderError::UnsupportedFeature(format!(
        "DOT edge {:?}->{:?} arrowhead/arrowtail/arrowsize/penwidth requires one straight TB polyline edge between fixed empty box nodes or a default-ellipse directed path",
        edge.from, edge.to
    )))
}

fn is_default_ellipse_arrow_path(model: &Model) -> bool {
    model.kind == GraphKind::Digraph
        && !model.strict
        && model.graph_attrs.is_empty()
        && model.subgraphs.is_empty()
        && (2..=16).contains(&model.edges.len())
        && model.nodes.len() == model.edges.len() + 1
        && model.nodes.iter().all(|node| node.attrs.is_empty())
        && model.edges.iter().all(|edge| {
            edge.from != edge.to
                && edge.attrs.keys().all(|key| {
                    matches!(
                        key.as_str(),
                        "dir" | "arrowhead" | "arrowtail" | "arrowsize"
                    )
                })
        })
        && model
            .edges
            .windows(2)
            .all(|pair| pair[0].to == pair[1].from)
}

fn is_plain_shortcut_triangle(model: &Model) -> bool {
    let [first, shortcut, last] = model.edges.as_slice() else {
        return false;
    };
    model.kind == GraphKind::Digraph
        && !model.strict
        && model.graph_attrs.is_empty()
        && model.subgraphs.is_empty()
        && model.nodes.len() == 3
        && model.nodes.iter().all(|node| node.attrs.is_empty())
        && model.edges.iter().all(|edge| edge.attrs.is_empty())
        && first.from == shortcut.from
        && first.to == last.from
        && shortcut.to == last.to
        && first.from != first.to
        && first.from != last.to
        && first.to != last.to
}

fn validate_shortcut_routes(model: &Model) -> Result<(), RenderError> {
    if is_plain_shortcut_triangle(model) {
        return Err(RenderError::UnsupportedFeature(
            "DOT plain three-node shortcut triangle needs Graphviz multi-rank routing".into(),
        ));
    }
    if model.kind != GraphKind::Digraph
        || model.edges.len() < 3
        || !model
            .edges
            .iter()
            .any(|edge| edge.attrs.contains_key("style"))
    {
        return Ok(());
    }
    let routes: HashSet<(&str, &str)> = model
        .edges
        .iter()
        .map(|edge| (edge.from.as_str(), edge.to.as_str()))
        .collect();
    for first in &model.edges {
        for second in &model.edges {
            if first.from != first.to
                && second.from != second.to
                && first.from != second.to
                && first.to == second.from
                && routes.contains(&(first.from.as_str(), second.to.as_str()))
            {
                return Err(RenderError::UnsupportedFeature(format!(
                    "DOT styled shortcut route {:?}->{:?} through {:?} needs Graphviz multi-rank routing",
                    first.from, second.to, first.to
                )));
            }
        }
    }
    Ok(())
}
struct ResolvedEndpoint {
    node: String,
    port: String,
}
struct SubgraphModel {
    name: String,
    attrs: Attributes,
    nodes: Vec<String>,
    edges: Vec<usize>,
    parent: usize,
    layout_id: String,
}
struct Model {
    kind: GraphKind,
    strict: bool,
    name: String,
    graph_attrs: Attributes,
    graph_label_kind: IdKind,
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    subgraphs: Vec<SubgraphModel>,
    index: HashMap<String, usize>,
    user_node_ids: HashSet<String>,
    synthetic_cluster_ids: HashSet<String>,
    clusters: usize,
    anonymous: usize,
}
struct Work {
    control: RenderControl,
    limits: RenderLimits,
    used: usize,
}

impl Work {
    fn new(options: &RenderOptions) -> Self {
        Self {
            control: crate::child_control(options),
            limits: options.limits.clone(),
            used: 0,
        }
    }
}
impl LayoutWork for Work {
    fn charge(&mut self, units: usize) -> Result<(), RenderError> {
        self.control
            .checkpoint()
            .map_err(|error| RenderError::Cancelled(error.to_string()))?;
        let next = self
            .used
            .checked_add(units)
            .ok_or_else(|| RenderError::Backend("DOT work count overflowed".into()))?;
        if next > self.limits.layout_work {
            return Err(RenderError::Backend(format!(
                "DOT work count {next} exceeds {}",
                self.limits.layout_work
            )));
        }
        self.used = next;
        Ok(())
    }
    fn depth(&self, depth: usize) -> Result<(), RenderError> {
        if depth > self.limits.nesting_depth {
            return Err(RenderError::Backend(format!(
                "DOT nesting depth {depth} exceeds {}",
                self.limits.nesting_depth
            )));
        }
        Ok(())
    }
}

impl Model {
    fn new(document: &Document, user_node_ids: HashSet<String>) -> Self {
        Self {
            kind: document.kind,
            strict: document.strict,
            name: document
                .id
                .as_ref()
                .map_or_else(|| "%1".into(), |id| id.text.clone()),
            graph_attrs: Attributes::new(),
            graph_label_kind: IdKind::Bare,
            nodes: Vec::new(),
            edges: Vec::new(),
            subgraphs: Vec::new(),
            index: HashMap::new(),
            user_node_ids,
            synthetic_cluster_ids: HashSet::new(),
            clusters: 0,
            anonymous: if document.id.is_some() { 1 } else { 3 },
        }
    }
    fn ensure_node(
        &mut self,
        id: &str,
        defaults: &Defaults,
        work: &mut Work,
    ) -> Result<usize, RenderError> {
        if let Some(&index) = self.index.get(id) {
            return Ok(index);
        }
        work.charge(1)?;
        if self.nodes.len() >= work.limits.model_items {
            return Err(RenderError::Backend(format!(
                "DOT node count exceeds {}",
                work.limits.model_items
            )));
        }
        let index = self.nodes.len();
        self.index.insert(id.into(), index);
        self.nodes.push(Node {
            id: id.into(),
            attrs: defaults.node.clone(),
            label_kind: defaults.node_label_kind,
            parent: defaults.parent.clone(),
        });
        Ok(index)
    }
    fn push_edge(
        &mut self,
        from: &ResolvedEndpoint,
        to: &ResolvedEndpoint,
        mut attrs: Attributes,
        work: &mut Work,
    ) -> Result<(), RenderError> {
        work.charge(1)?;
        if self.edges.len() >= work.limits.model_items {
            return Err(RenderError::Backend(format!(
                "DOT edge count exceeds {}",
                work.limits.model_items
            )));
        }
        if !from.port.is_empty() {
            attrs.insert("tailport".into(), from.port.clone());
        }
        if !to.port.is_empty() {
            attrs.insert("headport".into(), to.port.clone());
        }
        if self.strict
            && let Some(edge) = self.edges.iter_mut().find(|edge| {
                edge.from == from.node && edge.to == to.node
                    || self.kind == GraphKind::Graph && edge.from == to.node && edge.to == from.node
            })
        {
            edge.attrs.extend(attrs);
            return Ok(());
        }
        self.edges.push(Edge {
            from: from.node.clone(),
            to: to.node.clone(),
            attrs,
        });
        // cgraph/id.c shares an odd-numbered anonymous ID sequence between
        // unnamed edges and anonymous subgraphs.
        self.anonymous = self
            .anonymous
            .checked_add(2)
            .ok_or_else(|| RenderError::Backend("DOT anonymous object ID overflowed".into()))?;
        Ok(())
    }
}

fn attributes(items: &[Attribute]) -> Attributes {
    items
        .iter()
        .map(|item| {
            (
                item.name.text.clone(),
                item.value
                    .as_ref()
                    .map_or_else(String::new, |value| value.text.clone()),
            )
        })
        .collect()
}

fn label_kind(items: &[Attribute]) -> Option<IdKind> {
    items
        .iter()
        .rev()
        .find(|item| item.name.text == "label")
        .and_then(|item| item.value.as_ref().map(|value| value.kind))
}

fn collect_user_node_ids(statements: &[Statement]) -> HashSet<String> {
    fn collect(statements: &[Statement], ids: &mut HashSet<String>) {
        for statement in statements {
            match statement {
                Statement::Node { node, .. } => {
                    ids.insert(node.id.text.clone());
                }
                Statement::Nodes { nodes, .. } => {
                    ids.extend(nodes.iter().map(|node| node.id.text.clone()));
                }
                Statement::Edge { endpoints, .. } => {
                    for endpoint in endpoints {
                        match endpoint {
                            Endpoint::Node(node) => {
                                ids.insert(node.id.text.clone());
                            }
                            Endpoint::Nodes(nodes) => {
                                ids.extend(nodes.iter().map(|node| node.id.text.clone()));
                            }
                            Endpoint::Subgraph(subgraph) => collect(&subgraph.statements, ids),
                        }
                    }
                }
                Statement::Subgraph(subgraph) => collect(&subgraph.statements, ids),
                Statement::Attributes { .. } | Statement::Assignment { .. } => {}
            }
        }
    }
    let mut ids = HashSet::new();
    collect(statements, &mut ids);
    ids
}

fn lower(document: &Document, work: &mut Work) -> Result<Model, RenderError> {
    let mut model = Model::new(document, collect_user_node_ids(&document.statements));
    visit(
        &document.statements,
        &mut model,
        &mut Defaults::default(),
        0,
        work,
    )?;
    Ok(model)
}

fn visit(
    statements: &[Statement],
    model: &mut Model,
    defaults: &mut Defaults,
    depth: usize,
    work: &mut Work,
) -> Result<Vec<String>, RenderError> {
    work.depth(depth)?;
    let mut members = Vec::new();
    for statement in statements {
        work.charge(1)?;
        match statement {
            Statement::Node {
                node,
                attributes: values,
            } => {
                endpoint_port(node)?;
                let index = model.ensure_node(&node.id.text, defaults, work)?;
                model.nodes[index].attrs.extend(attributes(values));
                if let Some(kind) = label_kind(values) {
                    model.nodes[index].label_kind = kind;
                }
                members.push(node.id.text.clone());
            }
            Statement::Nodes {
                nodes,
                attributes: values,
            } => {
                for node in nodes {
                    endpoint_port(node)?;
                    let index = model.ensure_node(&node.id.text, defaults, work)?;
                    model.nodes[index].attrs.extend(attributes(values));
                    if let Some(kind) = label_kind(values) {
                        model.nodes[index].label_kind = kind;
                    }
                    members.push(node.id.text.clone());
                }
            }
            Statement::Edge {
                endpoints,
                operators,
                attributes: values,
            } => {
                let mut resolved = Vec::with_capacity(endpoints.len());
                for endpoint in endpoints {
                    resolved.push(resolve(endpoint, model, defaults, depth + 1, work)?);
                }
                for (pair, operator) in resolved.windows(2).zip(operators) {
                    if (*operator == EdgeOp::Directed) != (model.kind == GraphKind::Digraph) {
                        return Err(RenderError::InvalidSource(
                            "DOT edge operator conflicts with graph kind".into(),
                        ));
                    }
                    for from in &pair[0] {
                        for to in &pair[1] {
                            work.charge(1)?;
                            let mut attrs = defaults.edge.clone();
                            attrs.extend(attributes(values));
                            model.push_edge(from, to, attrs, work)?;
                        }
                    }
                }
                for ids in resolved {
                    members.extend(ids.into_iter().map(|id| id.node));
                }
            }
            Statement::Attributes {
                scope,
                attributes: values,
            } => {
                let attrs = attributes(values);
                match scope {
                    AttributeScope::Graph => {
                        if let Some(kind) = label_kind(values)
                            && depth == 0
                        {
                            model.graph_label_kind = kind;
                        }
                        defaults.graph.extend(attrs.clone());
                        if depth == 0 {
                            model.graph_attrs.extend(attrs);
                        }
                    }
                    AttributeScope::Node => {
                        if let Some(kind) = label_kind(values) {
                            defaults.node_label_kind = kind;
                        }
                        defaults.node.extend(attrs)
                    }
                    AttributeScope::Edge => defaults.edge.extend(attrs),
                }
            }
            Statement::Assignment { name, value } => {
                if name.text == "label" && depth == 0 {
                    model.graph_label_kind = value.kind;
                }
                defaults.graph.insert(name.text.clone(), value.text.clone());
                if depth == 0 {
                    model
                        .graph_attrs
                        .insert(name.text.clone(), value.text.clone());
                }
            }
            Statement::Subgraph(subgraph) => {
                members.extend(visit_subgraph(subgraph, model, defaults, depth + 1, work)?)
            }
        }
    }
    let mut seen = HashSet::new();
    members.retain(|member| seen.insert(member.clone()));
    Ok(members)
}

fn resolve(
    endpoint: &Endpoint,
    model: &mut Model,
    defaults: &Defaults,
    depth: usize,
    work: &mut Work,
) -> Result<Vec<ResolvedEndpoint>, RenderError> {
    match endpoint {
        Endpoint::Node(node) => {
            model.ensure_node(&node.id.text, defaults, work)?;
            Ok(vec![ResolvedEndpoint {
                node: node.id.text.clone(),
                port: endpoint_port(node)?,
            }])
        }
        Endpoint::Nodes(nodes) => {
            let mut ids = Vec::with_capacity(nodes.len());
            for node in nodes {
                model.ensure_node(&node.id.text, defaults, work)?;
                ids.push(ResolvedEndpoint {
                    node: node.id.text.clone(),
                    port: endpoint_port(node)?,
                });
            }
            Ok(ids)
        }
        Endpoint::Subgraph(subgraph) => Ok(visit_subgraph(subgraph, model, defaults, depth, work)?
            .into_iter()
            .map(|node| ResolvedEndpoint {
                node,
                port: String::new(),
            })
            .collect()),
    }
}

fn endpoint_port(node: &NodeId) -> Result<String, RenderError> {
    if node.port.is_empty() {
        return Ok(String::new());
    }
    let parts = node
        .port
        .iter()
        .map(|id| id.text.as_str())
        .collect::<Vec<_>>();
    if parts.iter().any(|part| part.is_empty() || part.len() > 128) {
        return Err(RenderError::InvalidSource(format!(
            "DOT port on {:?} has an empty or overlong identifier",
            node.id.text
        )));
    }
    if let [_, compass] = parts.as_slice()
        && !matches!(
            *compass,
            "n" | "ne" | "e" | "se" | "s" | "sw" | "w" | "nw" | "c" | "_"
        )
    {
        return Err(RenderError::InvalidSource(format!(
            "DOT compass {compass:?} on {:?} is invalid",
            node.id.text
        )));
    }
    Ok(parts.join(":"))
}

fn visit_subgraph(
    subgraph: &Subgraph,
    model: &mut Model,
    defaults: &Defaults,
    depth: usize,
    work: &mut Work,
) -> Result<Vec<String>, RenderError> {
    work.charge(1)?;
    if model.subgraphs.len() >= work.limits.model_items {
        return Err(RenderError::Backend(format!(
            "DOT subgraph count exceeds {}",
            work.limits.model_items
        )));
    }
    let mut scoped = defaults.clone();
    let name = subgraph.id.as_ref().map_or_else(
        || {
            let name = format!("%{}", model.anonymous);
            model.anonymous += 2;
            name
        },
        |id| id.text.clone(),
    );
    let index = model.subgraphs.len();
    model.subgraphs.push(SubgraphModel {
        name,
        attrs: Attributes::new(),
        nodes: Vec::new(),
        edges: Vec::new(),
        parent: scoped.subgraph,
        layout_id: String::new(),
    });
    scoped.subgraph = index;
    if subgraph
        .id
        .as_ref()
        .is_some_and(|id| id.text.starts_with("cluster"))
    {
        let id = loop {
            work.charge(1)?;
            let id = format!("__ive_dot_cluster_{}", model.clusters);
            model.clusters = model.clusters.checked_add(1).ok_or_else(|| {
                RenderError::Backend("DOT synthetic cluster ID count overflowed".into())
            })?;
            if !model.user_node_ids.contains(&id) {
                break id;
            }
        };
        model.ensure_node(&id, defaults, work)?;
        model.synthetic_cluster_ids.insert(id.clone());
        scoped.parent = id;
        model.subgraphs[index].layout_id = scoped.parent.clone();
    }
    let edge_start = model.edges.len();
    let members = visit(&subgraph.statements, model, &mut scoped, depth, work)?;
    model.subgraphs[index].attrs = scoped.graph;
    model.subgraphs[index].nodes = members.clone();
    model.subgraphs[index].edges = (edge_start..model.edges.len()).collect();
    Ok(members)
}

#[derive(Clone, Copy)]
enum NumericField {
    NodeWidth,
    NodeHeight,
    FontSize,
    NodeSep,
    RankSep,
}

impl NumericField {
    fn spec(self) -> (&'static str, f64, f64) {
        match self {
            Self::NodeWidth => ("width", 0.75, 0.01),
            Self::NodeHeight => ("height", 0.5, 0.02),
            Self::FontSize => ("fontsize", 14., 1.),
            Self::NodeSep => ("nodesep", 0.25, 0.02),
            Self::RankSep => ("ranksep", 0.5, 0.02),
        }
    }
}

fn number(attrs: &Attributes, field: NumericField) -> Result<f64, RenderError> {
    let (key, fallback, minimum) = field.spec();
    let Some(raw) = attrs.get(key) else {
        return Ok(fallback);
    };
    let text = if matches!(field, NumericField::RankSep) {
        raw.split_whitespace().next().unwrap_or("")
    } else {
        raw.as_str()
    };
    let value = text.parse::<f64>().unwrap_or(fallback);
    if !value.is_finite() || value > 1000. {
        return Err(RenderError::InvalidSource(format!(
            "DOT {key} value {raw:?} exceeds safe numeric limit 1000"
        )));
    }
    Ok(value.max(minimum))
}

fn graph_label(attrs: &Attributes) -> Result<GraphLabel, RenderError> {
    if let Some(size) = attrs.get("size") {
        return Err(RenderError::UnsupportedFeature(format!(
            "DOT graph size {size:?} requires output scaling"
        )));
    }
    let rankdir = match attrs
        .get("rankdir")
        .map(String::as_str)
        .unwrap_or("TB")
        .to_ascii_uppercase()
        .as_str()
    {
        "TB" => RankDir::TB,
        "BT" => RankDir::BT,
        "LR" => RankDir::LR,
        "RL" => RankDir::RL,
        value => {
            return Err(RenderError::InvalidSource(format!(
                "DOT rankdir {value:?} is invalid"
            )));
        }
    };
    Ok(GraphLabel {
        rankdir,
        nodesep: (number(attrs, NumericField::NodeSep)? * 72.).round(),
        ranksep: (number(attrs, NumericField::RankSep)? * 72.).round(),
        ..Default::default()
    })
}

#[derive(Clone, Copy)]
enum RootLabelLocation {
    Top,
    Bottom,
}

#[derive(Clone, Copy)]
enum RootLabelJustification {
    Left,
    Center,
    Right,
}

struct RootLabelSpec<'a> {
    text: &'a str,
    width: f64,
    height: f64,
    location: RootLabelLocation,
    justification: RootLabelJustification,
}

fn root_label_lines(text: &str) -> Vec<&str> {
    text.split("\\n")
        .flat_map(|segment| segment.split('\n'))
        .collect()
}

impl RootLabelSpec<'_> {
    fn position(&self, graph: &LayoutGraph) -> (f64, f64) {
        let x = match self.justification {
            RootLabelJustification::Left => 8. + self.width / 2.,
            RootLabelJustification::Center => graph.graph().width / 2.,
            RootLabelJustification::Right => graph.graph().width - 8. - self.width / 2.,
        };
        let y = match self.location {
            RootLabelLocation::Top => graph.graph().height - self.height / 2. - 4.,
            RootLabelLocation::Bottom => self.height / 2. + 4.,
        };
        (x, y)
    }
}

fn root_label_spec(model: &Model) -> Result<Option<RootLabelSpec<'_>>, RenderError> {
    let Some(text) = model
        .graph_attrs
        .get("label")
        .filter(|text| !text.is_empty())
    else {
        return Ok(None);
    };
    if model.graph_label_kind == IdKind::Html {
        return Err(RenderError::UnsupportedFeature(
            "DOT graph HTML labels require table rendering".into(),
        ));
    }
    let lines = root_label_lines(text);
    if text.len() > 4096 || lines.len() > 32 || lines.iter().any(|line| line.contains('\\')) {
        return Err(RenderError::UnsupportedFeature(format!(
            "DOT graph label {text:?} exceeds the bounded plain-text subset"
        )));
    }
    let font_size = number(&model.graph_attrs, NumericField::FontSize)?;
    let font_name = model
        .graph_attrs
        .get("fontname")
        .map(String::as_str)
        .unwrap_or("Times-Roman");
    let width = lines
        .iter()
        .map(|line| {
            font_metrics::ascii_advance(line, font_name, font_size).ok_or_else(|| {
                RenderError::UnsupportedFeature(format!(
                    "DOT graph label {text:?} in font {font_name:?}"
                ))
            })
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .fold(0., f64::max);
    let location = match model
        .graph_attrs
        .get("labelloc")
        .map(String::as_str)
        .unwrap_or("b")
    {
        "b" => RootLabelLocation::Bottom,
        "t" => RootLabelLocation::Top,
        value => {
            return Err(RenderError::UnsupportedFeature(format!(
                "DOT graph labelloc {value:?}"
            )));
        }
    };
    let justification = match model
        .graph_attrs
        .get("labeljust")
        .map(String::as_str)
        .unwrap_or("c")
    {
        "l" => RootLabelJustification::Left,
        "c" => RootLabelJustification::Center,
        "r" => RootLabelJustification::Right,
        value => {
            return Err(RenderError::UnsupportedFeature(format!(
                "DOT graph labeljust {value:?}"
            )));
        }
    };
    Ok(Some(RootLabelSpec {
        text,
        width,
        height: font_size * 1.2 * lines.len() as f64,
        location,
        justification,
    }))
}

fn position_root_label(
    model: &Model,
    graph: &mut LayoutGraph,
    work: &mut Work,
) -> Result<(), RenderError> {
    let Some(label) = root_label_spec(model)? else {
        return Ok(());
    };
    let old_width = graph.graph().width;
    let old_height = graph.graph().height;
    let new_width = old_width.max(label.width + 16.);
    let extra_height = label.height + 8.;
    let dx = (new_width - old_width) / 2.;
    let dy = if matches!(label.location, RootLabelLocation::Top) {
        extra_height
    } else {
        0.
    };
    let node_ids = graph.node_ids();
    let edge_keys = graph.edge_keys();
    work.charge(node_ids.len() + edge_keys.len() + label.text.len())?;
    for id in node_ids {
        let node = graph.node_mut(&id).ok_or_else(|| {
            RenderError::Backend(format!("DOT root label node {id:?} disappeared"))
        })?;
        node.x = node.x.map(|x| x + dx);
        node.y = node.y.map(|y| y + dy);
    }
    for key in edge_keys {
        let edge = graph
            .edge_mut_by_key(&key)
            .ok_or_else(|| RenderError::Backend("DOT root label edge disappeared".into()))?;
        work.charge(edge.points.len())?;
        edge.x = edge.x.map(|x| x + dx);
        edge.y = edge.y.map(|y| y + dy);
        for point in &mut edge.points {
            point.x += dx;
            point.y += dy;
        }
    }
    graph.graph_mut().width = new_width;
    graph.graph_mut().height = old_height + extra_height;
    Ok(())
}

fn measure_label(
    label: &str,
    attrs: &Attributes,
    session: &RenderSession,
    work: &mut Work,
) -> Result<(f64, f64), RenderError> {
    if label.is_empty() {
        return Ok((0., 0.));
    }
    work.charge(label.len() + 1)?;
    let font_size = number(attrs, NumericField::FontSize)?;
    let font_name = attrs
        .get("fontname")
        .map(String::as_str)
        .unwrap_or("Times-Roman");
    let style = TextStyle {
        font_family: Some(font_name.into()),
        font_size,
        font_weight: Some("normal".into()),
        font_style: Some("normal".into()),
    };
    let spans = label_spans(label)?;
    let measurer = session.text_measurer(TextMeasurementPhase::Layout);
    let mut width: f64 = 0.;
    for span in &spans {
        work.charge(1)?;
        let line = span.text;
        let width_here = font_metrics::ascii_advance(line, font_name, font_size)
            .unwrap_or_else(|| measurer.measure(line, &style).width);
        if !width_here.is_finite() || width_here < 0. {
            return Err(RenderError::Backend(format!(
                "DOT text measurement failed for label {line:?}"
            )));
        }
        width = width.max(width_here);
    }
    Ok((width, spans.len() as f64 * font_size * 1.2))
}

#[derive(Clone, Copy)]
enum LabelAlignment {
    Center,
    Left,
    Right,
}

struct LabelSpan<'a> {
    text: &'a str,
    alignment: LabelAlignment,
}

fn label_spans(label: &str) -> Result<Vec<LabelSpan<'_>>, RenderError> {
    let mut spans = Vec::new();
    let mut start = 0;
    let mut index = 0;
    let bytes = label.as_bytes();
    while index < bytes.len() {
        let delimiter = if bytes[index] == b'\\' {
            match bytes.get(index + 1) {
                Some(b'n') => Some((2, LabelAlignment::Center)),
                Some(b'l') => Some((2, LabelAlignment::Left)),
                Some(b'r') => Some((2, LabelAlignment::Right)),
                _ => None,
            }
        } else if bytes[index] == b'\n' {
            Some((1, LabelAlignment::Center))
        } else {
            None
        };
        if let Some((length, alignment)) = delimiter {
            spans.push(LabelSpan {
                text: &label[start..index],
                alignment,
            });
            index += length;
            start = index;
        } else {
            index += 1;
        }
    }
    if start < label.len() {
        spans.push(LabelSpan {
            text: &label[start..],
            alignment: LabelAlignment::Center,
        });
    }
    if spans.len() > 16 || spans.iter().any(|span| span.text.is_empty()) {
        return Err(RenderError::UnsupportedFeature(format!(
            "DOT multiline label {label:?} exceeds 16 nonempty lines"
        )));
    }
    Ok(spans)
}

fn record_spec(node: &Node, rankdir: RankDir) -> Result<RecordSpec, RenderError> {
    if node.label_kind == IdKind::Html {
        return Err(RenderError::UnsupportedFeature(format!(
            "DOT record node {:?} with HTML label",
            node.id
        )));
    }
    let label = node.attrs.get("label").ok_or_else(|| {
        RenderError::UnsupportedFeature(format!("DOT record node {:?} without a label", node.id))
    })?;
    if label.len() > 4096 || label.contains(['\\', '\n']) {
        return Err(RenderError::UnsupportedFeature(format!(
            "DOT record label on {:?} exceeds the bounded field subset",
            node.id
        )));
    }
    if node.attrs.contains_key("margin") || node.attrs.contains_key("fixedsize") {
        return Err(RenderError::UnsupportedFeature(format!(
            "DOT record margin or fixedsize on {:?}",
            node.id
        )));
    }
    let font_size = number(&node.attrs, NumericField::FontSize)?;
    let font_name = node
        .attrs
        .get("fontname")
        .map(String::as_str)
        .unwrap_or("Times-Roman");
    let vertical = matches!(rankdir, RankDir::LR | RankDir::RL);
    let root = record_group(node, label, vertical, 0, (font_name, font_size))?;
    if root.leaf_count() > 64 {
        return Err(RenderError::InvalidSource(format!(
            "DOT record node {:?} has more than 64 leaf fields",
            node.id
        )));
    }
    // Graphviz shapes.c applies PAD (16 x 8 points) per field and adds one
    // point to record node height after fitting fields to minimum node size.
    Ok(RecordSpec {
        width: root
            .width
            .max(number(&node.attrs, NumericField::NodeWidth)? * 72.),
        height: root
            .height
            .max(number(&node.attrs, NumericField::NodeHeight)? * 72.)
            + 1.,
        root,
    })
}

fn validate_nested_record_edge(model: &Model, edge: &Edge) -> Result<(), RenderError> {
    let nested = |id: &str| {
        let node = &model.nodes[model.index[id]];
        node.attrs
            .get("shape")
            .is_some_and(|shape| shape == "record")
            && node
                .attrs
                .get("label")
                .is_some_and(|label| label.contains('{'))
    };
    if !nested(&edge.from) && !nested(&edge.to) {
        return Ok(());
    }
    let compass = |name: &str| {
        edge.attrs.get(name).is_some_and(|port| {
            port.split_once(':').is_some_and(|(field, direction)| {
                !field.is_empty() && matches!(direction, "n" | "s" | "e" | "w")
            })
        })
    };
    if model.edges.len() == 1 && compass("tailport") && compass("headport") {
        return Ok(());
    }
    Err(RenderError::UnsupportedFeature(format!(
        "DOT edge {:?}->{:?} with nested record ports requires one edge and explicit n/s/e/w compass ports",
        edge.from, edge.to
    )))
}

fn record_group(
    node: &Node,
    text: &str,
    vertical: bool,
    depth: usize,
    font: (&str, f64),
) -> Result<RecordField, RenderError> {
    if depth > 4 {
        return Err(RenderError::UnsupportedFeature(format!(
            "DOT record node {:?} exceeds four nested levels",
            node.id
        )));
    }
    let mut fields = Vec::new();
    let mut level = 0_usize;
    let mut start = 0;
    for (index, character) in text.char_indices() {
        match character {
            '{' => level += 1,
            '}' if level > 0 => level -= 1,
            '}' => {
                return Err(RenderError::InvalidSource(format!(
                    "DOT record node {:?} has unmatched }} in {text:?}",
                    node.id
                )));
            }
            '|' if level == 0 => {
                fields.push(record_part(
                    node,
                    &text[start..index],
                    vertical,
                    depth,
                    font,
                )?);
                start = index + 1;
            }
            _ => {}
        }
    }
    if level != 0 {
        return Err(RenderError::InvalidSource(format!(
            "DOT record node {:?} has unmatched {{ in {text:?}",
            node.id
        )));
    }
    fields.push(record_part(node, &text[start..], vertical, depth, font)?);
    if fields.len() > 64 {
        return Err(RenderError::InvalidSource(format!(
            "DOT record node {:?} has more than 64 fields",
            node.id
        )));
    }
    let (width, height) = if vertical {
        (
            fields.iter().map(|field| field.width).fold(0., f64::max),
            fields.iter().map(|field| field.height).sum(),
        )
    } else {
        (
            fields.iter().map(|field| field.width).sum(),
            fields.iter().map(|field| field.height).fold(0., f64::max),
        )
    };
    Ok(RecordField {
        port: String::new(),
        text: String::new(),
        width,
        height,
        children: fields,
        vertical,
    })
}

fn record_part(
    node: &Node,
    part: &str,
    vertical: bool,
    depth: usize,
    font: (&str, f64),
) -> Result<RecordField, RenderError> {
    let field = part.trim();
    if let Some(inner) = field
        .strip_prefix('{')
        .and_then(|value| value.strip_suffix('}'))
    {
        return record_group(node, inner, !vertical, depth + 1, font);
    }
    if field.contains(['{', '}']) {
        return Err(RenderError::UnsupportedFeature(format!(
            "DOT record node {:?} field {part:?} has unsupported nesting",
            node.id
        )));
    }
    let (port, text) = if let Some(raw) = field.strip_prefix('<') {
        raw.split_once('>').ok_or_else(|| {
            RenderError::InvalidSource(format!(
                "DOT record node {:?} has an unterminated port in {part:?}",
                node.id
            ))
        })?
    } else {
        ("", field)
    };
    let text = text.trim();
    if port.len() > 128 || text.contains(['<', '>']) {
        return Err(RenderError::UnsupportedFeature(format!(
            "DOT record field {part:?} on {:?}",
            node.id
        )));
    }
    let width = font_metrics::ascii_advance(text, font.0, font.1).ok_or_else(|| {
        RenderError::UnsupportedFeature(format!(
            "DOT record field {part:?} in font {:?} on {:?}",
            font.0, node.id
        ))
    })? + 16.;
    Ok(RecordField {
        port: port.into(),
        text: text.into(),
        width,
        height: font.1 * 1.2 + 8.,
        children: Vec::new(),
        vertical: false,
    })
}

fn node_peripheries(node: &Node) -> Result<usize, RenderError> {
    let Some(raw) = node.attrs.get("peripheries") else {
        return Ok(1);
    };
    let shape = node
        .attrs
        .get("shape")
        .map(String::as_str)
        .unwrap_or("ellipse");
    if raw == "1" {
        return Ok(1);
    }
    if raw == "2"
        && matches!(
            shape,
            "box" | "rect" | "rectangle" | "square" | "ellipse" | "oval" | "circle"
        )
    {
        return Ok(2);
    }
    Err(RenderError::UnsupportedFeature(format!(
        "DOT node {:?} shape {shape:?} peripheries {raw:?}",
        node.id
    )))
}

fn node_penwidth(node: &Node) -> Result<f64, RenderError> {
    let width = node.attrs.get("penwidth").map_or(Ok(1.), |raw| {
        raw.parse::<f64>().map_err(|_| {
            RenderError::UnsupportedFeature(format!("DOT node {:?} penwidth {raw:?}", node.id))
        })
    })?;
    if !width.is_finite() || !(0. ..=100.).contains(&width) {
        return Err(RenderError::UnsupportedFeature(format!(
            "DOT node {:?} penwidth {width} is outside 0..=100",
            node.id
        )));
    }
    let bold = node
        .attrs
        .get("style")
        .is_some_and(|style| style.split(',').any(|part| part.trim() == "bold"));
    Ok(if bold { width.max(2.) } else { width })
}

fn node_dimensions(
    node: &Node,
    rankdir: RankDir,
    session: &RenderSession,
    work: &mut Work,
) -> Result<(f64, f64), RenderError> {
    let peripheries = node_peripheries(node)?;
    let (width, height) = node_base_dimensions(node, rankdir, session, work)?;
    // Graphviz shapes.c grows each outer periphery by GAP=4 points per side.
    let expansion = 8. * (peripheries - 1) as f64;
    Ok((width + expansion, height + expansion))
}

fn node_base_dimensions(
    node: &Node,
    rankdir: RankDir,
    session: &RenderSession,
    work: &mut Work,
) -> Result<(f64, f64), RenderError> {
    if let Some(margin) = node.attrs.get("margin") {
        return Err(RenderError::UnsupportedFeature(format!(
            "DOT node {:?} margin {margin:?} requires Graphviz text padding",
            node.id
        )));
    }
    if node
        .attrs
        .get("shape")
        .is_some_and(|shape| shape == "record")
    {
        let spec = record_spec(node, rankdir)?;
        return Ok((spec.width, spec.height));
    }
    let shape = node
        .attrs
        .get("shape")
        .map(String::as_str)
        .unwrap_or("ellipse");
    if matches!(shape, "diamond" | "triangle" | "hexagon") {
        for key in ["orientation", "skew", "distortion", "regular", "sides"] {
            if let Some(value) = node.attrs.get(key) {
                return Err(RenderError::UnsupportedFeature(format!(
                    "DOT {shape} node {:?} {key}={value:?}",
                    node.id
                )));
            }
        }
    }
    let width = number(&node.attrs, NumericField::NodeWidth)? * 72.;
    let height = number(&node.attrs, NumericField::NodeHeight)? * 72.;
    let (width, height) = if shape == "circle" {
        let size = if node.attrs.contains_key("width") || node.attrs.contains_key("height") {
            let explicit_width = if node.attrs.contains_key("width") {
                width
            } else {
                0.
            };
            let explicit_height = if node.attrs.contains_key("height") {
                height
            } else {
                0.
            };
            explicit_width.max(explicit_height)
        } else {
            width.min(height)
        };
        (size, size)
    } else {
        (width, height)
    };
    if node.label_kind == IdKind::Html {
        if node
            .attrs
            .get("fixedsize")
            .is_some_and(|value| value == "true")
        {
            return Err(RenderError::UnsupportedFeature(format!(
                "DOT HTML table node {:?} with fixedsize=true",
                node.id
            )));
        }
        let label = node.attrs.get("label").ok_or_else(|| {
            RenderError::InvalidSource(format!("DOT HTML table node {:?} has no label", node.id))
        })?;
        work.charge(label.len() + 1)?;
        let table = html_table::HtmlTable::parse(label, &node.attrs)?;
        return Ok(table.node_dimensions((width, height)));
    }
    if node
        .attrs
        .get("fixedsize")
        .is_some_and(|value| value == "true")
    {
        return Ok((width, height));
    }
    let label = effective_node_label(node);
    if label.is_empty() {
        return Ok((width, height));
    }
    let (measured_width, measured_height) = measure_label(&label, &node.attrs, session, work)?;
    let mut label_width = measured_width + 16.;
    let mut label_height = measured_height + 8.;
    if shape == "diamond" {
        if height > label_height * std::f64::consts::SQRT_2 {
            let fraction = label_height / height;
            label_width /= (1. - fraction * fraction).sqrt();
        } else {
            label_width *= std::f64::consts::SQRT_2;
            label_height *= std::f64::consts::SQRT_2;
        }
        label_width *= std::f64::consts::SQRT_2;
        label_height *= std::f64::consts::SQRT_2;
    }
    if matches!(
        shape,
        "ellipse" | "oval" | "circle" | "triangle" | "hexagon"
    ) {
        if height > label_height {
            let fraction = label_height / height;
            label_width /= (1. - fraction * fraction).sqrt();
        } else {
            label_width *= std::f64::consts::SQRT_2;
            label_height *= std::f64::consts::SQRT_2;
        }
    }
    if matches!(shape, "triangle" | "hexagon") {
        let sides = if shape == "triangle" { 3. } else { 6. };
        let angle = std::f64::consts::PI / sides;
        label_width /= angle.cos();
        label_height /= angle.cos();
    }
    if shape == "circle" {
        let size = width.max(label_width).max(label_height);
        return Ok((size, size));
    }
    Ok((width.max(label_width), height.max(label_height)))
}

fn effective_node_label(node: &Node) -> String {
    node.attrs
        .get("label")
        .map(String::as_str)
        .unwrap_or("\\N")
        .replace("\\N", &node.id)
}

fn build_graph(
    model: &Model,
    work: &mut Work,
    session: &RenderSession,
) -> Result<(LayoutGraph, BTreeMap<String, f64>), RenderError> {
    let mut graph = Graph::new(GraphOptions {
        directed: model.kind == GraphKind::Digraph,
        multigraph: true,
        compound: true,
    });
    let graph_label = graph_label(&model.graph_attrs)?;
    let rankdir = graph_label.rankdir;
    graph.set_graph(graph_label);
    let clusters = model
        .subgraphs
        .iter()
        .filter(|cluster| !cluster.layout_id.is_empty())
        .map(|cluster| (cluster.layout_id.as_str(), cluster))
        .collect::<HashMap<_, _>>();
    let mut cluster_label_widths = BTreeMap::new();
    for node in &model.nodes {
        work.charge(1)?;
        let incident = model
            .edges
            .iter()
            .any(|edge| edge.from == node.id || edge.to == node.id);
        let shape = node
            .attrs
            .get("shape")
            .map(String::as_str)
            .unwrap_or("ellipse");
        if incident && matches!(shape, "triangle" | "hexagon") {
            return Err(RenderError::UnsupportedFeature(format!(
                "DOT {shape} node {:?} with incident edges requires unimplemented polygon clipping",
                node.id
            )));
        }
        if incident && shape == "diamond" {
            let bounded = rankdir == RankDir::TB
                && node.attrs.contains_key("width")
                && node.attrs.contains_key("height")
                && model.edges.iter().all(|edge| edge.from != edge.to)
                && model.subgraphs.is_empty();
            if !bounded {
                return Err(RenderError::UnsupportedFeature(format!(
                    "DOT diamond node {:?} with incident edges requires explicit dimensions and straight TB routing",
                    node.id
                )));
            }
        }
        let (width, height) = if let Some(cluster) = clusters.get(node.id.as_str()) {
            let (margin, label_width, label_height) = cluster_metadata(cluster, session, work)?;
            cluster_label_widths.insert(node.id.clone(), label_width);
            (margin, label_height)
        } else {
            node_dimensions(node, rankdir, session, work)?
        };
        graph.set_node(
            &node.id,
            NodeLabel {
                width,
                height,
                dummy: model
                    .synthetic_cluster_ids
                    .contains(&node.id)
                    .then(|| "ive-dot-cluster".into()),
                ..Default::default()
            },
        );
    }
    let parents = model
        .nodes
        .iter()
        .filter(|node| !node.parent.is_empty())
        .map(|node| {
            let child = graph.node_ix(&node.id).ok_or_else(|| {
                RenderError::Backend(format!("DOT node {:?} disappeared", node.id))
            })?;
            let parent = graph.node_ix(&node.parent).ok_or_else(|| {
                RenderError::Backend(format!("DOT cluster {:?} disappeared", node.parent))
            })?;
            Ok((child, parent))
        })
        .collect::<Result<Vec<_>, RenderError>>()?;
    graph
        .try_set_unparented_parents_ix(&parents)
        .map_err(|error| RenderError::InvalidSource(format!("DOT cluster nesting: {error}")))?;
    for (index, edge) in model.edges.iter().enumerate() {
        work.charge(1)?;
        let label = edge.attrs.get("label").map(String::as_str).unwrap_or("");
        let (width, measured_height) = measure_label(label, &edge.attrs, session, work)?;
        let minlen = edge.attrs.get("minlen").map_or(Ok(1), |value| {
            value
                .parse::<usize>()
                .map_err(|_| RenderError::InvalidSource(format!("DOT minlen {value:?} is invalid")))
        })?;
        if minlen > 1000 {
            return Err(RenderError::InvalidSource(format!(
                "DOT minlen {minlen} exceeds 1000"
            )));
        }
        let weight = edge.attrs.get("weight").map_or(Ok(1.), |value| {
            value
                .parse::<f64>()
                .map_err(|_| RenderError::InvalidSource(format!("DOT weight {value:?} is invalid")))
        })?;
        if !weight.is_finite() || !(0. ..=1000.).contains(&weight) {
            return Err(RenderError::InvalidSource(format!(
                "DOT weight {weight} is outside 0..=1000"
            )));
        }
        let constraint = edge
            .attrs
            .get("constraint")
            .map(String::as_str)
            .unwrap_or("true");
        if !matches!(constraint, "true" | "false") {
            return Err(RenderError::InvalidSource(format!(
                "DOT constraint {constraint:?} is invalid"
            )));
        }
        let arrows = arrow_spec(edge, model.kind)?;
        validate_extended_arrow(model, edge, arrows)?;
        validate_nested_record_edge(model, edge)?;
        let mut extras = BTreeMap::from([
            ("constraint".into(), json!(constraint == "true")),
            ("dot_label_present".into(), json!(!label.is_empty())),
            ("dot_original_tail".into(), json!(edge.from)),
            ("dot_original_head".into(), json!(edge.to)),
            ("dot_arrow_present".into(), json!(arrows.head)),
            ("dot_arrow_tail_present".into(), json!(arrows.tail)),
            (
                "dot_arrow_head_shape".into(),
                json!(arrows.head_shape.name()),
            ),
            (
                "dot_arrow_tail_shape".into(),
                json!(arrows.tail_shape.name()),
            ),
            ("dot_arrow_size".into(), json!(arrows.size)),
            ("dot_arrow_penwidth".into(), json!(arrows.penwidth)),
        ]);
        for (shape_key, periphery_key, penwidth_key, node_id) in [
            (
                "dot_tail_shape",
                "dot_tail_peripheries",
                "dot_tail_penwidth",
                &edge.from,
            ),
            (
                "dot_head_shape",
                "dot_head_peripheries",
                "dot_head_penwidth",
                &edge.to,
            ),
        ] {
            let node = &model.nodes[model.index[node_id]];
            let shape = node
                .attrs
                .get("shape")
                .map(String::as_str)
                .unwrap_or("ellipse");
            extras.insert(shape_key.into(), json!(shape));
            extras.insert(periphery_key.into(), json!(node_peripheries(node)?));
            extras.insert(penwidth_key.into(), json!(node_penwidth(node)?));
        }
        for (key, node_id) in [
            ("dot_tail_record_fields", &edge.from),
            ("dot_head_record_fields", &edge.to),
        ] {
            let node = &model.nodes[model.index[node_id]];
            if node
                .attrs
                .get("shape")
                .is_some_and(|shape| shape == "record")
            {
                let fields = record_spec(node, rankdir)?
                    .cells()
                    .into_iter()
                    .map(|cell| json!({"port":cell.port,"x":cell.x,"y":cell.y,"width":cell.width,"height":cell.height}))
                    .collect::<Vec<_>>();
                extras.insert(key.into(), json!(fields));
            }
        }
        for (key, port) in [
            ("dot_tail_port", edge.attrs.get("tailport")),
            ("dot_head_port", edge.attrs.get("headport")),
        ] {
            if let Some(port) = port {
                extras.insert(key.into(), json!(port));
            }
        }
        graph
            .try_set_edge_named(
                &edge.from,
                &edge.to,
                Some(format!("e{index}")),
                Some(EdgeLabel {
                    width,
                    height: measured_height,
                    minlen,
                    weight,
                    extras,
                    ..Default::default()
                }),
            )
            .map_err(|error| {
                RenderError::InvalidSource(format!(
                    "DOT edge {:?}->{:?}: {error}",
                    edge.from, edge.to
                ))
            })?;
    }
    validate_shortcut_routes(model)?;
    Ok((graph, cluster_label_widths))
}

fn cluster_metadata(
    cluster: &SubgraphModel,
    session: &RenderSession,
    work: &mut Work,
) -> Result<(f64, f64, f64), RenderError> {
    let margin = cluster.attrs.get("margin").map_or(Ok(8.), |raw| {
        let value = raw.parse::<f64>().map_err(|_| {
            RenderError::InvalidSource(format!(
                "DOT cluster {:?} margin {raw:?} is invalid",
                cluster.name
            ))
        })?;
        if !value.is_finite() || !(0. ..=1000.).contains(&value) {
            return Err(RenderError::InvalidSource(format!(
                "DOT cluster {:?} margin {raw:?} is outside 0..=1000 points",
                cluster.name
            )));
        }
        Ok(value)
    })?;
    let label = cluster.attrs.get("label").map(String::as_str).unwrap_or("");
    let (label_width, label_height) = measure_label(label, &cluster.attrs, session, work)?;
    Ok((margin, label_width, label_height))
}

fn prepare(
    source: &str,
    options: &RenderOptions,
) -> Result<(Model, LayoutGraph, Work, RenderSession), RenderError> {
    crate::limits::validate_source(source, &options.limits)?;
    let policy = crate::limits::resource_policy(&options.limits)?;
    let mut work = Work::new(options);
    let session = RenderEnvironment::deterministic()
        .without_math_renderer()
        .with_resource_policy(policy)
        .with_text_measurement_policy(options.text_measurement.clone())
        .begin_session_with_control(work.control.clone())
        .map_err(|error| RenderError::Backend(format!("DOT text measurement setup: {error}")))?;
    let parser = dot::ParseOptions {
        source_bytes: options.limits.source_bytes,
        tokens: options.limits.model_items.saturating_mul(20),
        statements: options.limits.model_items,
        nesting_depth: options.limits.nesting_depth,
        control: work.control.clone(),
    };
    let document = dot::parse(source, &parser)
        .map_err(|error| RenderError::InvalidSource(error.to_string()))?;
    let model = lower(&document, &mut work)?;
    let (mut graph, cluster_label_widths) = build_graph(&model, &mut work, &session)?;
    layered::layout_raw_dot_with_cluster_labels(&mut graph, &mut work, &cluster_label_widths)?;
    position_root_label(&model, &mut graph, &mut work)?;
    Ok((model, graph, work, session))
}

pub(crate) fn render(
    source: &str,
    options: &RenderOptions,
) -> Result<RenderedDiagram, RenderError> {
    let (model, graph, work, session) = prepare(source, options)?;
    let svg = svg_output::render(&model, &graph, options.limits.svg_bytes)?;
    crate::svg::validate(&svg, &options.limits)?;
    Ok(RenderedDiagram {
        svg,
        language: crate::Language::Dot,
        backend: "native Rust DOT",
        upstream: "Graphviz 16.1.0 target",
        family: "dot".into(),
        diagnostics: Vec::new(),
        measurement: session.text_measurement_report(),
        layout_work_units: work.used,
    })
}

pub(crate) fn native_png(
    source: &str,
    options: &RenderOptions,
) -> Result<Option<Vec<u8>>, RenderError> {
    let (model, graph, mut work, _) = prepare(source, options)?;
    if !gd_raster::is_supported(&model, &graph) {
        return Ok(None);
    }
    let svg = svg_output::render(&model, &graph, options.limits.svg_bytes)?;
    crate::svg::validate(&svg, &options.limits)?;
    match gd_raster::render(&model, &graph, &mut work) {
        Err(RenderError::UnsupportedFeature(_)) => Ok(None),
        result => result,
    }
}

pub(crate) fn json0(source: &str, options: &RenderOptions) -> Result<String, RenderError> {
    let (model, graph, _, _) = prepare(source, options)?;
    let height = graph.graph().height;
    let node_ids = model
        .nodes
        .iter()
        .filter(|node| !model.synthetic_cluster_ids.contains(&node.id))
        .enumerate()
        .map(|(index, node)| (node.id.as_str(), model.subgraphs.len() + index))
        .collect::<HashMap<_, _>>();
    let edge_labels = if model
        .edges
        .iter()
        .any(|edge| edge.attrs.contains_key("label"))
    {
        EdgeLabelPolicy::Declared
    } else {
        EdgeLabelPolicy::Absent
    };
    let edge_ids = graphviz_edge_ids(&model);
    let context = JsonContext {
        node_ids: &node_ids,
        edge_ids: &edge_ids,
        graph: &graph,
        height,
        rankdir: graph_label(&model.graph_attrs)?.rankdir,
        kind: model.kind,
        edge_labels,
    };
    let mut objects = model
        .subgraphs
        .iter()
        .enumerate()
        .map(|(index, subgraph)| subgraph_json(&model.subgraphs, (index, subgraph), &context))
        .collect::<Result<Vec<_>, _>>()?;
    for node in model
        .nodes
        .iter()
        .filter(|node| !model.synthetic_cluster_ids.contains(&node.id))
    {
        let id = node_ids[&node.id.as_str()];
        objects.push(node_json(node, id, &context)?);
    }
    let edges = model
        .edges
        .iter()
        .enumerate()
        .map(|(index, edge)| edge_json(edge, (index, edge_ids[index]), &context))
        .collect::<Result<Vec<_>, _>>()?;
    let mut result = object();
    result.insert("name".into(), json!(model.name));
    result.insert("directed".into(), json!(model.kind == GraphKind::Digraph));
    result.insert("strict".into(), json!(model.strict));
    let mut graph_attrs = model.graph_attrs.clone();
    if model
        .subgraphs
        .iter()
        .any(|subgraph| subgraph.attrs.contains_key("label"))
    {
        graph_attrs.entry("label".into()).or_default();
    }
    graph_attrs.insert(
        "bb".into(),
        format!("0,0,{},{}", format5(graph.graph().width), format5(height)),
    );
    if let Some(label) = root_label_spec(&model)? {
        let (x, y) = label.position(&graph);
        graph_attrs.insert("lp".into(), point5(x, y));
        graph_attrs.insert("lwidth".into(), format!("{:.2}", label.width / 72.));
        graph_attrs.insert("lheight".into(), format!("{:.2}", label.height / 72.));
    }
    extend_attrs(&mut result, graph_attrs);
    result.insert("_subgraph_cnt".into(), json!(model.subgraphs.len()));
    if !objects.is_empty() {
        result.insert("objects".into(), Value::Array(objects));
    }
    if !edges.is_empty() {
        result.insert("edges".into(), Value::Array(edges));
    }
    let result = Value::Object(result);
    let output = serde_json::to_string(&result)
        .map_err(|error| RenderError::Backend(format!("cannot serialize DOT geometry: {error}")))?;
    if output.len() > options.limits.svg_bytes {
        return Err(RenderError::Backend(format!(
            "DOT JSON has {} bytes, exceeding output limit {}",
            output.len(),
            options.limits.svg_bytes
        )));
    }
    Ok(output)
}

fn object() -> Map<String, Value> {
    Map::new()
}

#[derive(Clone, Copy)]
enum EdgeLabelPolicy {
    Absent,
    Declared,
}

fn graphviz_edge_ids(model: &Model) -> Vec<usize> {
    // gvrender_core_json.c numbers outgoing edges by tail node sequence;
    // cgraph/edge.c orders each adjacency by head node, then edge sequence.
    // The JSON edge array itself remains in global edge creation order.
    let mut outgoing = (0..model.edges.len()).collect::<Vec<_>>();
    outgoing.sort_by_key(|&index| {
        (
            model.index[&model.edges[index].from],
            model.index[&model.edges[index].to],
            index,
        )
    });
    let mut ids = vec![0; outgoing.len()];
    for (id, index) in outgoing.into_iter().enumerate() {
        ids[index] = id;
    }
    ids
}

struct JsonContext<'a> {
    node_ids: &'a HashMap<&'a str, usize>,
    edge_ids: &'a [usize],
    graph: &'a LayoutGraph,
    height: f64,
    rankdir: RankDir,
    kind: GraphKind,
    edge_labels: EdgeLabelPolicy,
}

fn extend_attrs(result: &mut Map<String, Value>, attrs: Attributes) {
    for (key, value) in attrs {
        if !value.is_empty() || key == "label" {
            result.insert(key, Value::String(value));
        }
    }
}

fn subgraph_json(
    subgraphs: &[SubgraphModel],
    (id, subgraph): (usize, &SubgraphModel),
    context: &JsonContext<'_>,
) -> Result<Value, RenderError> {
    let mut result = object();
    result.insert("name".into(), json!(subgraph.name));
    let mut attrs = subgraph.attrs.clone();
    if subgraphs
        .iter()
        .any(|item| item.attrs.contains_key("label"))
    {
        attrs.entry("label".into()).or_default();
    }
    if !subgraph.layout_id.is_empty() {
        let placed = context.graph.node(&subgraph.layout_id).ok_or_else(|| {
            RenderError::Backend(format!("DOT cluster {:?} layout is missing", subgraph.name))
        })?;
        let x = placed.x.unwrap_or(0.);
        let y = placed.y.unwrap_or(0.);
        attrs.insert(
            "bb".into(),
            format!(
                "{},{},{},{}",
                format5(x - placed.width / 2.),
                format5(context.height - y - placed.height / 2.),
                format5(x + placed.width / 2.),
                format5(context.height - y + placed.height / 2.)
            ),
        );
        if let Some(label) = attrs.get("label").filter(|label| !label.is_empty()) {
            let font_size = number(&attrs, NumericField::FontSize)?;
            let font_name = attrs
                .get("fontname")
                .map(String::as_str)
                .unwrap_or("Times-Roman");
            let width =
                font_metrics::ascii_advance(label, font_name, font_size).ok_or_else(|| {
                    RenderError::UnsupportedFeature(format!(
                        "DOT JSON cluster label {label:?} in font {font_name:?}"
                    ))
                })?;
            attrs.insert("lheight".into(), format!("{:.2}", font_size * 1.2 / 72.));
            attrs.insert("lwidth".into(), format!("{:.2}", width / 72.));
            attrs.insert(
                "lp".into(),
                point5(
                    x,
                    context.height - y + placed.height / 2. - (font_size * 0.6 + 4.),
                ),
            );
        }
    }
    extend_attrs(&mut result, attrs);
    result.insert("_gvid".into(), json!(id));
    let nested = subgraphs
        .iter()
        .enumerate()
        .filter_map(|(index, item)| (item.parent == id).then_some(index))
        .collect::<Vec<_>>();
    if !nested.is_empty() {
        result.insert("subgraphs".into(), json!(nested));
    }
    let children = subgraph
        .nodes
        .iter()
        .filter_map(|node| context.node_ids.get(node.as_str()).copied())
        .collect::<Vec<_>>();
    if !children.is_empty() {
        result.insert("nodes".into(), json!(children));
    }
    if !subgraph.edges.is_empty() {
        let ids = subgraph
            .edges
            .iter()
            .map(|&index| context.edge_ids[index])
            .collect::<Vec<_>>();
        result.insert("edges".into(), json!(ids));
    }
    Ok(Value::Object(result))
}

fn node_json(node: &Node, id: usize, context: &JsonContext<'_>) -> Result<Value, RenderError> {
    let placed = context
        .graph
        .node(&node.id)
        .ok_or_else(|| RenderError::Backend(format!("DOT node {:?} layout is missing", node.id)))?;
    let mut result = object();
    result.insert("_gvid".into(), json!(id));
    result.insert("name".into(), json!(node.id));
    let mut attrs = node.attrs.clone();
    attrs.entry("label".into()).or_insert("\\N".into());
    attrs.insert("height".into(), format5(placed.height / 72.));
    attrs.insert("width".into(), format5(placed.width / 72.));
    attrs.insert(
        "pos".into(),
        point5(
            placed.x.unwrap_or(0.),
            context.height - placed.y.unwrap_or(0.),
        ),
    );
    if node
        .attrs
        .get("shape")
        .is_some_and(|shape| shape == "record")
    {
        let rects = record_spec(node, context.rankdir)?
            .cells()
            .into_iter()
            .map(|cell| {
                let x = placed.x.unwrap_or(0.) + cell.x;
                let y = context.height - placed.y.unwrap_or(0.) - cell.y;
                format!(
                    "{},{},{},{}",
                    format5(x - cell.width / 2.),
                    format5(y - cell.height / 2.),
                    format5(x + cell.width / 2.),
                    format5(y + cell.height / 2.)
                )
            })
            .collect::<Vec<_>>()
            .join(" ");
        attrs.insert("rects".into(), rects);
    }
    extend_attrs(&mut result, attrs);
    Ok(Value::Object(result))
}

fn edge_json(
    edge: &Edge,
    (index, id): (usize, usize),
    context: &JsonContext<'_>,
) -> Result<Value, RenderError> {
    let placed = context
        .graph
        .edge(&edge.from, &edge.to, Some(&format!("e{index}")))
        .ok_or_else(|| RenderError::Backend(format!("DOT edge {index} layout is missing")))?;
    let tail = context
        .node_ids
        .get(edge.from.as_str())
        .ok_or_else(|| RenderError::Backend(format!("DOT edge tail {:?} is missing", edge.from)))?;
    let head = context
        .node_ids
        .get(edge.to.as_str())
        .ok_or_else(|| RenderError::Backend(format!("DOT edge head {:?} is missing", edge.to)))?;
    let mut result = object();
    result.insert("_gvid".into(), json!(id));
    result.insert("tail".into(), json!(tail));
    result.insert("head".into(), json!(head));
    let mut attrs = edge.attrs.clone();
    if matches!(context.edge_labels, EdgeLabelPolicy::Declared) {
        attrs.entry("label".into()).or_default();
    }
    if attrs.get("label").is_some_and(|label| !label.is_empty()) {
        attrs.insert(
            "lp".into(),
            point5(
                placed.x.unwrap_or(0.),
                context.height - placed.y.unwrap_or(0.),
            ),
        );
    }
    if !placed.points.is_empty() {
        let points = placed
            .points
            .iter()
            .map(|point| point5(point.x, context.height - point.y))
            .collect::<Vec<_>>();
        let arrows = arrow_spec(edge, context.kind)?;
        let mut route = Vec::new();
        if arrows.tail {
            route.push(format!("s,{}", points[0]));
        }
        if arrows.head {
            route.push(format!("e,{}", points[points.len() - 1]));
        }
        // Fallback line routes reuse their endpoints as arrow tips.
        let (start, end) = if points.len() == 2 {
            (0, points.len())
        } else {
            (
                usize::from(arrows.tail),
                points
                    .len()
                    .checked_sub(usize::from(arrows.head))
                    .ok_or_else(|| RenderError::Backend("DOT edge route has no head tip".into()))?,
            )
        };
        if end < start {
            return Err(RenderError::Backend(
                "DOT edge route has no controls between arrow tips".into(),
            ));
        }
        route.extend(points.into_iter().skip(start).take(end - start));
        attrs.insert("pos".into(), route.join(" "));
    }
    extend_attrs(&mut result, attrs);
    Ok(Value::Object(result))
}

fn point5(x: f64, y: f64) -> String {
    format!("{},{}", format5(x), format5(y))
}

fn format5(value: f64) -> String {
    if value == 0. {
        return "0".into();
    }
    let exponent = value.abs().log10().floor() as i32;
    if !(-4..5).contains(&exponent) {
        let raw = format!("{value:.4e}");
        let (mantissa, exponent) = raw.split_once('e').expect("scientific float");
        let mantissa = mantissa.trim_end_matches('0').trim_end_matches('.');
        return format!("{mantissa}e{:+03}", exponent.parse::<i32>().unwrap_or(0));
    }
    let decimals = (4 - exponent).max(0) as usize;
    let formatted = format!("{value:.decimals$}");
    if formatted.contains('.') {
        return formatted.trim_end_matches('0').trim_end_matches('.').into();
    }
    formatted
}

#[cfg(test)]
mod tests {
    use super::*;
    fn parse_model(source: &str) -> Model {
        let options = RenderOptions {
            language: crate::Language::Dot,
            ..Default::default()
        };
        let document = dot::parse(source, &dot::ParseOptions::default()).unwrap();
        lower(&document, &mut Work::new(&options)).unwrap()
    }

    #[test]
    fn should_render_standalone_dot_without_a_process() {
        // given
        let source = "digraph G { node [shape=box]; a -> b [label=x]; }";
        let options = RenderOptions {
            language: crate::Language::Dot,
            ..Default::default()
        };
        // when
        let rendered = render(source, &options).unwrap();
        let geometry = json0(source, &options).unwrap();
        // then
        assert!(rendered.svg.contains("<svg"));
        assert!(rendered.svg.contains("a"));
        assert!(geometry.contains("\"edges\""));
    }

    #[test]
    fn should_render_user_node_ids_that_resemble_internal_clusters() {
        // given
        let sources = [
            "digraph G { __ive_dot_cluster_0; a }",
            "digraph G { __ive_dot_cluster_0; subgraph cluster_x { a -> b } }",
            "digraph G { subgraph cluster_x { a -> b } \"__ive_dot_cluster_0\" -> b }",
            "digraph G { __ive_dot_cluster_0; __ive_dot_cluster_1; subgraph cluster_x { a -> b } }",
        ];
        let options = RenderOptions {
            language: crate::Language::Dot,
            ..Default::default()
        };
        // when and then
        for source in sources {
            let geometry: Value = serde_json::from_str(&json0(source, &options).unwrap()).unwrap();
            let image = super::render(source, &options).unwrap();
            let names = geometry["objects"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|object| object["name"].as_str())
                .collect::<Vec<_>>();
            assert!(names.contains(&"__ive_dot_cluster_0"), "{source}");
            assert!(
                image.svg.contains("<title>__ive_dot_cluster_0</title>"),
                "{source}"
            );
            if source.contains("__ive_dot_cluster_1") {
                assert!(names.contains(&"__ive_dot_cluster_1"), "{source}");
                assert!(
                    image.svg.contains("<title>__ive_dot_cluster_1</title>"),
                    "{source}"
                );
            }
        }
    }

    #[test]
    fn should_reject_external_paint_urls_in_dot_svg() {
        // given
        let source = "digraph G { a [shape=box, color=\"url(https://example.invalid/p.svg#x)\"] }";
        let options = RenderOptions {
            language: crate::Language::Dot,
            ..Default::default()
        };
        // when
        let result = render(source, &options);
        // then
        assert!(
            matches!(result, Err(RenderError::ExternalResource(_))),
            "{result:?}"
        );
    }

    #[test]
    fn should_reject_size_and_node_margin_until_geometry_matches() {
        // given
        let cases = [
            ("digraph G { size=\"1,1\"; a -> b }", "graph size"),
            ("digraph G { a [shape=ellipse, margin=0.2] }", "margin"),
            ("digraph G { a [shape=box, margin=0.2] }", "margin"),
        ];
        let options = RenderOptions {
            language: crate::Language::Dot,
            ..Default::default()
        };
        // when and then
        for (source, expected) in cases {
            let geometry = json0(source, &options);
            let image = render(source, &options);
            assert!(
                matches!(geometry, Err(RenderError::UnsupportedFeature(message)) if message.contains(expected)),
                "JSON accepted ignored attribute in {source:?}"
            );
            assert!(
                matches!(image, Err(RenderError::UnsupportedFeature(message)) if message.contains(expected)),
                "SVG accepted ignored attribute in {source:?}"
            );
        }
    }

    #[test]
    fn should_expand_node_lists_and_merge_strict_reverse_edges() {
        // given
        let source = "strict graph G { a,b -- c,d [color=red]; c -- a [label=x] }";
        // when
        let model = parse_model(source);
        // then
        assert_eq!(model.nodes.len(), 4);
        assert_eq!(model.edges.len(), 4);
        let edge = model
            .edges
            .iter()
            .find(|edge| edge.from == "a" && edge.to == "c")
            .unwrap();
        assert_eq!(edge.attrs.get("label").map(String::as_str), Some("x"));
    }

    #[test]
    fn should_apply_node_defaults_only_when_node_is_created() {
        // given
        let source = "digraph G { node [color=red]; a; node [color=blue]; a; b }";
        // when
        let model = parse_model(source);
        // then
        assert_eq!(
            model.nodes[0].attrs.get("color").map(String::as_str),
            Some("red")
        );
        assert_eq!(
            model.nodes[1].attrs.get("color").map(String::as_str),
            Some("blue")
        );
    }

    #[test]
    fn should_reject_unbounded_minlen_instead_of_clamping() {
        // given
        let source = "digraph G { a -> b [minlen=1001] }";
        let options = RenderOptions {
            language: crate::Language::Dot,
            ..Default::default()
        };
        // when
        let result = json0(source, &options);
        // then
        assert!(
            matches!(result, Err(RenderError::InvalidSource(message)) if message.contains("minlen 1001"))
        );
    }

    #[test]
    fn should_paint_cluster_without_exposing_its_internal_node_name() {
        // given
        let source = "digraph G { subgraph cluster_one { a; b; a -> b; } }";
        let options = RenderOptions {
            language: crate::Language::Dot,
            ..Default::default()
        };
        // when
        let result = render(source, &options).unwrap();
        // then
        assert!(result.svg.contains("class=\"cluster\""));
        assert!(result.svg.contains("<polygon"));
        assert!(!result.svg.contains("__ive_dot_cluster_"));
    }

    #[test]
    fn should_match_graphviz_json0_for_empty_graph() {
        // given
        let source = "digraph G {}";
        let options = RenderOptions {
            language: crate::Language::Dot,
            ..Default::default()
        };
        let oracle =
            json!({"name":"G","directed":true,"strict":false,"bb":"0,0,0,0","_subgraph_cnt":0});
        // when
        let actual: Value = serde_json::from_str(&json0(source, &options).unwrap()).unwrap();
        // then
        assert_eq!(actual, oracle);
    }

    #[test]
    fn should_match_graphviz_json0_for_single_default_node() {
        // given
        let source = "digraph G { a; }";
        let options = RenderOptions {
            language: crate::Language::Dot,
            ..Default::default()
        };
        let oracle = json!({"name":"G","directed":true,"strict":false,"bb":"0,0,54,36","_subgraph_cnt":0,
            "objects":[{"_gvid":0,"name":"a","height":"0.5","label":"\\N","pos":"27,18","width":"0.75"}]});
        // when
        let actual: Value = serde_json::from_str(&json0(source, &options).unwrap()).unwrap();
        // then
        assert_eq!(actual, oracle);
    }

    #[test]
    fn should_number_feedback_edges_by_source_node_order() {
        // given
        let source =
            include_str!("../../tools/dot-parity/fixtures-extensions/ranked_self_projection.dot");
        let options = RenderOptions {
            language: crate::Language::Dot,
            ..Default::default()
        };

        // when
        let actual: Value = serde_json::from_str(&json0(source, &options).unwrap()).unwrap();
        let ids = actual["edges"]
            .as_array()
            .unwrap()
            .iter()
            .map(|edge| edge["_gvid"].as_u64().unwrap())
            .collect::<Vec<_>>();

        // then
        assert_eq!(ids, [1, 2, 3, 0]);
    }

    #[test]
    fn should_emit_graphviz_json0_ids_and_inherited_attributes() {
        // given
        let source = "strict digraph G { node [color=red]; a -> b [label=x]; b -> c; }";
        let options = RenderOptions {
            language: crate::Language::Dot,
            ..Default::default()
        };
        // when
        let actual: Value = serde_json::from_str(&json0(source, &options).unwrap()).unwrap();
        // then
        assert_eq!(actual["objects"][0]["_gvid"], 0);
        assert_eq!(actual["objects"][1]["_gvid"], 1);
        assert_eq!(actual["objects"][2]["_gvid"], 2);
        assert_eq!(actual["objects"][2]["color"], "red");
        assert_eq!(actual["edges"][0]["tail"], 0);
        assert_eq!(actual["edges"][0]["head"], 1);
        assert_eq!(actual["edges"][1]["label"], "");
        assert!(
            actual["edges"][0]["pos"]
                .as_str()
                .unwrap()
                .starts_with("e,")
        );
    }

    #[test]
    fn should_emit_subgraph_before_nodes_with_graphviz_ids() {
        // given
        let source = "digraph G { subgraph cluster_a { a; b; a -> b; } c; a -> c; }";
        let options = RenderOptions {
            language: crate::Language::Dot,
            ..Default::default()
        };
        // when
        let actual: Value = serde_json::from_str(&json0(source, &options).unwrap()).unwrap();
        // then
        assert_eq!(actual["_subgraph_cnt"], 1);
        assert_eq!(actual["objects"].as_array().unwrap().len(), 4);
        assert_eq!(actual["objects"][0]["name"], "cluster_a");
        assert_eq!(actual["objects"][0]["_gvid"], 0);
        assert_eq!(actual["objects"][0]["nodes"], json!([1, 2]));
        assert_eq!(actual["objects"][0]["edges"], json!([0]));
        assert!(actual["objects"][0]["bb"].as_str().is_some());
        assert_eq!(actual["edges"][1]["head"], 3);
    }

    #[test]
    fn should_format_geometry_with_graphviz_five_significant_digits() {
        // given
        let numbers = [0., 0.555556, 1.111111, 268.800096, 100., -0.00001];
        // when
        let formatted = numbers.into_iter().map(format5).collect::<Vec<_>>();
        // then
        assert_eq!(
            formatted,
            ["0", "0.55556", "1.1111", "268.8", "100", "-1e-05"]
        );
    }

    #[test]
    fn should_preserve_attribute_case_and_subgraph_node_order() {
        // given
        let source = "digraph G { subgraph named { z [Color=red]; a [LABEL=Hi]; } }";
        let options = RenderOptions {
            language: crate::Language::Dot,
            ..Default::default()
        };
        // when
        let actual: Value = serde_json::from_str(&json0(source, &options).unwrap()).unwrap();
        // then
        assert_eq!(actual["objects"][0]["nodes"], json!([1, 2]));
        assert_eq!(actual["objects"][1]["Color"], "red");
        assert_eq!(actual["objects"][2]["LABEL"], "Hi");
        assert_eq!(actual["objects"][2]["label"], "\\N");
    }

    #[test]
    fn should_limit_dot_json_output_bytes() {
        // given
        let source = "digraph G { a; }";
        let mut options = RenderOptions {
            language: crate::Language::Dot,
            ..Default::default()
        };
        options.limits.svg_bytes = 32;
        // when
        let result = json0(source, &options);
        // then
        assert!(
            matches!(result, Err(RenderError::Backend(message)) if message.contains("DOT JSON") && message.contains("output limit 32"))
        );
    }

    #[test]
    fn should_clamp_numeric_attributes_to_graphviz_field_minima() {
        // given
        let attrs = Attributes::from([
            ("width".into(), "0".into()),
            ("height".into(), "0".into()),
            ("fontsize".into(), "0".into()),
            ("nodesep".into(), "0".into()),
            ("ranksep".into(), "0 equally".into()),
        ]);
        // when
        let values = [
            number(&attrs, NumericField::NodeWidth).unwrap(),
            number(&attrs, NumericField::NodeHeight).unwrap(),
            number(&attrs, NumericField::FontSize).unwrap(),
            number(&attrs, NumericField::NodeSep).unwrap(),
            number(&attrs, NumericField::RankSep).unwrap(),
        ];
        // then
        assert_eq!(values, [0.01, 0.02, 1., 0.02, 0.02]);
    }

    #[test]
    fn should_match_graphviz_161_strict_json_geometry() {
        // given
        let source = include_str!("../../tools/dot-parity/fixtures/strict.dot");
        let options = RenderOptions {
            language: crate::Language::Dot,
            ..Default::default()
        };
        let oracle = json!({"name":"G","directed":true,"strict":true,"bb":"0,0,68.493,108","rankdir":"TB","_subgraph_cnt":0,
            "objects":[{"_gvid":0,"name":"quoted id","height":"0.5","label":"\\N","pos":"34.247,90","shape":"box","width":"0.95129"},
                       {"_gvid":1,"name":"b","height":"0.5","label":"\\N","pos":"34.247,18","shape":"box","width":"0.75"}],
            "edges":[{"_gvid":0,"tail":0,"head":1,"color":"blue","id":"e0","pos":"e,34.247,36.104 34.247,71.697 34.247,64.407 34.247,55.726 34.247,47.536","weight":"3"}]});
        // when
        let actual: Value = serde_json::from_str(&json0(source, &options).unwrap()).unwrap();
        // then
        assert_eq!(actual, oracle);
    }

    #[test]
    fn should_match_graphviz_161_courier_chain_label_positions() {
        // given
        let source = include_str!("../../tools/dot-parity/fixtures/chain.dot");
        let options = RenderOptions {
            language: crate::Language::Dot,
            ..Default::default()
        };
        // when
        let actual: Value = serde_json::from_str(&json0(source, &options).unwrap()).unwrap();
        // then
        assert_eq!(actual["edges"][0]["lp"], "43.601,191.6");
        assert_eq!(actual["edges"][1]["lp"], "43.601,77.2");
        assert_eq!(
            actual["edges"][0]["pos"],
            "e,40,154.85 40,228.48 40,211.35 40,186.04 40,166.13"
        );
    }

    #[test]
    fn should_match_graphviz_161_common_edge_styles_json0() {
        // given
        let cases = [
            (
                "digraph G { a -> b [style=dashed,color=red] }",
                include_str!("dot_runtime/goldens/edge_dashed.json"),
            ),
            (
                "digraph G { a -> b [style=dotted,color=gray] }",
                include_str!("dot_runtime/goldens/edge_dotted.json"),
            ),
            (
                "digraph G { a -> b [style=bold,color=gray] }",
                include_str!("dot_runtime/goldens/edge_bold.json"),
            ),
        ];
        let options = RenderOptions {
            language: crate::Language::Dot,
            ..Default::default()
        };
        // when and then
        for (source, oracle) in cases {
            let actual: Value = serde_json::from_str(&json0(source, &options).unwrap()).unwrap();
            let expected: Value = serde_json::from_str(oracle).unwrap();
            assert_eq!(actual, expected, "{source}");
        }
    }

    #[test]
    fn should_match_graphviz_161_bounded_penwidth_and_multi_edge_style_json0() {
        // given
        let cases = [
            (
                include_str!("dot_runtime/goldens/edge_penwidth_red_dash.dot"),
                include_str!("dot_runtime/goldens/edge_penwidth_red_dash.json"),
            ),
            (
                include_str!("dot_runtime/goldens/edge_penwidth_hex_dot.dot"),
                include_str!("dot_runtime/goldens/edge_penwidth_hex_dot.json"),
            ),
            (
                include_str!("dot_runtime/goldens/edge_penwidth_bold_width.dot"),
                include_str!("dot_runtime/goldens/edge_penwidth_bold_width.json"),
            ),
            (
                include_str!("dot_runtime/goldens/edge_penwidth_width2.dot"),
                include_str!("dot_runtime/goldens/edge_penwidth_width2.json"),
            ),
            (
                include_str!("dot_runtime/goldens/edge_style_chain.dot"),
                include_str!("dot_runtime/goldens/edge_style_chain.json"),
            ),
            (
                include_str!("dot_runtime/goldens/edge_width_chain_none.dot"),
                include_str!("dot_runtime/goldens/edge_width_chain_none.json"),
            ),
        ];
        let options = RenderOptions {
            language: crate::Language::Dot,
            ..Default::default()
        };
        // when and then
        for (source, oracle) in cases {
            let actual: Value = serde_json::from_str(&json0(source, &options).unwrap()).unwrap();
            let expected: Value = serde_json::from_str(oracle).unwrap();
            assert_eq!(actual, expected, "{source}");
        }
    }

    #[test]
    fn should_reject_multi_edge_penwidth_until_nonstraight_clipping_matches() {
        // given
        let source = include_str!("../../tools/dot-parity/fixtures-features/edge_styles.dot");
        let options = RenderOptions {
            language: crate::Language::Dot,
            ..Default::default()
        };
        // when
        let result = json0(source, &options);
        // then
        assert!(
            matches!(result, Err(RenderError::UnsupportedFeature(message)) if message.contains("penwidth requires one straight"))
        );
    }

    #[test]
    fn should_reject_styled_shortcut_until_multi_rank_routing_matches() {
        // given
        let source = include_str!("dot_runtime/goldens/edge_multi_style.dot");
        let options = RenderOptions {
            language: crate::Language::Dot,
            ..Default::default()
        };
        // when
        let result = json0(source, &options);
        // then
        assert!(
            matches!(result, Err(RenderError::UnsupportedFeature(message)) if message.contains("styled shortcut route"))
        );
    }

    #[test]
    fn should_reject_plain_shortcut_triangle_until_multi_rank_routing_matches() {
        // given
        let source =
            include_str!("../../tools/dot-parity/fixtures-edge-style-triangle/plain_triangle.dot");
        let options = RenderOptions {
            language: crate::Language::Dot,
            ..Default::default()
        };
        // when
        let json = json0(source, &options);
        let svg = super::render(source, &options);
        // then
        assert!(
            matches!(json, Err(RenderError::UnsupportedFeature(message)) if message.contains("plain three-node shortcut triangle"))
        );
        assert!(
            matches!(svg, Err(RenderError::UnsupportedFeature(message)) if message.contains("plain three-node shortcut triangle"))
        );
    }

    #[test]
    fn should_match_graphviz_161_arrow_mode_json0() {
        // given
        let cases = [
            (
                include_str!("dot_runtime/goldens/arrow_none.dot"),
                include_str!("dot_runtime/goldens/arrow_none.json"),
            ),
            (
                include_str!("dot_runtime/goldens/arrow_head_none.dot"),
                include_str!("dot_runtime/goldens/arrow_head_none.json"),
            ),
            (
                include_str!("dot_runtime/goldens/arrow_back.dot"),
                include_str!("dot_runtime/goldens/arrow_back.json"),
            ),
            (
                include_str!("dot_runtime/goldens/arrow_both.dot"),
                include_str!("dot_runtime/goldens/arrow_both.json"),
            ),
        ];
        let options = RenderOptions {
            language: crate::Language::Dot,
            ..Default::default()
        };
        // when and then
        for (source, oracle) in cases {
            let actual: Value = serde_json::from_str(&json0(source, &options).unwrap()).unwrap();
            let expected: Value = serde_json::from_str(oracle).unwrap();
            assert_eq!(actual, expected, "{source}");
        }
    }

    #[test]
    fn should_reject_arrow_geometry_without_exact_routes() {
        // given
        let cases = [
            ("arrowhead=crow", "arrowhead"),
            ("arrowhead=diamondvee", "arrowhead"),
            ("arrowtail=dot,dir=back", "arrowtail"),
            ("penwidth=2", "penwidth"),
        ];
        let options = RenderOptions {
            language: crate::Language::Dot,
            ..Default::default()
        };
        // when and then
        for (attributes, name) in cases {
            let source = format!("digraph G {{ a -> b [{attributes}] }}");
            let result = json0(&source, &options);
            assert!(
                matches!(result, Err(RenderError::UnsupportedFeature(message)) if message.contains(name)),
                "{source}"
            );
        }
    }

    #[test]
    fn should_match_graphviz_161_arrow_shape_json0() {
        // given
        let cases = [
            (
                include_str!("dot_runtime/goldens/arrow_head_vee.dot"),
                include_str!("dot_runtime/goldens/arrow_head_vee.json"),
            ),
            (
                include_str!("dot_runtime/goldens/arrow_head_diamond.dot"),
                include_str!("dot_runtime/goldens/arrow_head_diamond.json"),
            ),
            (
                include_str!("dot_runtime/goldens/arrow_head_box.dot"),
                include_str!("dot_runtime/goldens/arrow_head_box.json"),
            ),
            (
                include_str!("dot_runtime/goldens/arrow_head_size2.dot"),
                include_str!("dot_runtime/goldens/arrow_head_size2.json"),
            ),
            (
                include_str!("dot_runtime/goldens/arrow_head_diamond_size2.dot"),
                include_str!("dot_runtime/goldens/arrow_head_diamond_size2.json"),
            ),
            (
                include_str!("dot_runtime/goldens/arrow_tail_vee.dot"),
                include_str!("dot_runtime/goldens/arrow_tail_vee.json"),
            ),
            (
                include_str!("dot_runtime/goldens/arrow_tail_diamond.dot"),
                include_str!("dot_runtime/goldens/arrow_tail_diamond.json"),
            ),
            (
                include_str!("dot_runtime/goldens/arrow_tail_box.dot"),
                include_str!("dot_runtime/goldens/arrow_tail_box.json"),
            ),
        ];
        let options = RenderOptions {
            language: crate::Language::Dot,
            ..Default::default()
        };
        // when and then
        for (source, oracle) in cases {
            let actual: Value = serde_json::from_str(&json0(source, &options).unwrap()).unwrap();
            let expected: Value = serde_json::from_str(oracle).unwrap();
            assert_eq!(actual, expected, "{source}");
        }
    }

    #[test]
    fn should_match_graphviz_161_multi_edge_arrow_directions_json0() {
        // given
        let cases = [
            (
                include_str!("dot_runtime/goldens/edge_arrows_feature.dot"),
                include_str!("dot_runtime/goldens/edge_arrows_feature.json"),
            ),
            (
                include_str!("dot_runtime/goldens/edge_arrow_back_path.dot"),
                include_str!("dot_runtime/goldens/edge_arrow_back_path.json"),
            ),
            (
                include_str!("dot_runtime/goldens/edge_arrow_none_path.dot"),
                include_str!("dot_runtime/goldens/edge_arrow_none_path.json"),
            ),
            (
                include_str!("dot_runtime/goldens/edge_arrow_head_size_path.dot"),
                include_str!("dot_runtime/goldens/edge_arrow_head_size_path.json"),
            ),
        ];
        let options = RenderOptions {
            language: crate::Language::Dot,
            ..Default::default()
        };
        // when and then
        for (source, oracle) in cases {
            let actual: Value = serde_json::from_str(&json0(source, &options).unwrap()).unwrap();
            let expected: Value = serde_json::from_str(oracle).unwrap();
            assert_eq!(actual, expected, "{source}");
        }
    }

    #[test]
    fn should_reject_extended_arrow_outside_straight_fixed_box_scope() {
        // given
        let cases = [
            "digraph G { a -> b [arrowhead=vee] }",
            "digraph G { graph [splines=polyline,rankdir=LR]; node [shape=box,label=\"\",fixedsize=true,width=1,height=.5]; a -> b [arrowhead=box] }",
            "digraph G { graph [splines=polyline]; node [shape=box,label=\"\",fixedsize=true,width=1,height=.5]; a -> b [arrowsize=2]; b -> c }",
            "digraph G { a -> b [arrowhead=vee]; a -> c }",
        ];
        let options = RenderOptions {
            language: crate::Language::Dot,
            ..Default::default()
        };
        // when and then
        for source in cases {
            assert!(
                matches!(json0(source, &options), Err(RenderError::UnsupportedFeature(message)) if message.contains("straight TB polyline edge")),
                "{source}"
            );
        }
    }

    #[test]
    fn should_match_graphviz_161_common_node_styles_json0() {
        // given
        let cases = [
            (
                "digraph G { a [shape=box,style=\"rounded,filled\",fillcolor=\"#ddffee\"] }",
                include_str!("dot_runtime/goldens/node_rounded_filled.json"),
            ),
            (
                "digraph G { a [shape=ellipse,style=\"dashed,bold\",color=blue,penwidth=2] }",
                include_str!("dot_runtime/goldens/node_dashed_bold.json"),
            ),
            (
                "digraph G { a [shape=box,style=dashed] }",
                include_str!("dot_runtime/goldens/node_dashed_box.json"),
            ),
            (
                "digraph G { a [shape=box,style=dotted] }",
                include_str!("dot_runtime/goldens/node_dotted_box.json"),
            ),
        ];
        let options = RenderOptions {
            language: crate::Language::Dot,
            ..Default::default()
        };
        // when and then
        for (source, oracle) in cases {
            let actual: Value = serde_json::from_str(&json0(source, &options).unwrap()).unwrap();
            let expected: Value = serde_json::from_str(oracle).unwrap();
            assert_eq!(actual, expected, "{source}");
        }
    }

    #[test]
    fn should_match_graphviz_161_two_peripheries_and_combined_styles_json0() {
        // given
        let cases = [
            (
                include_str!("dot_runtime/goldens/node_periphery_box_two.dot"),
                include_str!("dot_runtime/goldens/node_periphery_box_two.json"),
            ),
            (
                include_str!("dot_runtime/goldens/node_periphery_box_two_fill.dot"),
                include_str!("dot_runtime/goldens/node_periphery_box_two_fill.json"),
            ),
            (
                include_str!("dot_runtime/goldens/node_periphery_box_two_rounded.dot"),
                include_str!("dot_runtime/goldens/node_periphery_box_two_rounded.json"),
            ),
            (
                include_str!("dot_runtime/goldens/node_periphery_ellipse_two.dot"),
                include_str!("dot_runtime/goldens/node_periphery_ellipse_two.json"),
            ),
            (
                include_str!("dot_runtime/goldens/node_periphery_box_two_edge.dot"),
                include_str!("dot_runtime/goldens/node_periphery_box_two_edge.json"),
            ),
            (
                include_str!("dot_runtime/goldens/node_styles_feature.dot"),
                include_str!("dot_runtime/goldens/node_styles_feature.json"),
            ),
        ];
        let options = RenderOptions {
            language: crate::Language::Dot,
            ..Default::default()
        };
        // when and then
        for (source, oracle) in cases {
            let actual: Value = serde_json::from_str(&json0(source, &options).unwrap()).unwrap();
            let expected: Value = serde_json::from_str(oracle).unwrap();
            assert_eq!(actual, expected, "{source}");
        }
    }

    #[test]
    fn should_match_graphviz_161_isolated_diamond_json0() {
        // given
        let cases = [
            (
                "digraph G { a [shape=diamond] }",
                include_str!("dot_runtime/goldens/shape_diamond.json"),
            ),
            (
                "digraph G { a [shape=diamond,style=filled,fillcolor=gold] }",
                include_str!("dot_runtime/goldens/shape_diamond_filled.json"),
            ),
            (
                "digraph G { a [shape=diamond,label=\"abcdefgh\"] }",
                include_str!("dot_runtime/goldens/shape_diamond_long.json"),
            ),
        ];
        let options = RenderOptions {
            language: crate::Language::Dot,
            ..Default::default()
        };
        // when and then
        for (source, oracle) in cases {
            let actual: Value = serde_json::from_str(&json0(source, &options).unwrap()).unwrap();
            let expected: Value = serde_json::from_str(oracle).unwrap();
            assert_eq!(actual, expected, "{source}");
        }
    }

    #[test]
    fn should_match_graphviz_161_isolated_polygon_family_json0() {
        // given
        let cases = [
            (
                include_str!("dot_runtime/goldens/shape_triangle.dot"),
                include_str!("dot_runtime/goldens/shape_triangle.json"),
            ),
            (
                include_str!("dot_runtime/goldens/shape_hexagon.dot"),
                include_str!("dot_runtime/goldens/shape_hexagon.json"),
            ),
            (
                include_str!("dot_runtime/goldens/shape_circle.dot"),
                include_str!("dot_runtime/goldens/shape_circle.json"),
            ),
            (
                include_str!("dot_runtime/goldens/shape_triangle_sized.dot"),
                include_str!("dot_runtime/goldens/shape_triangle_sized.json"),
            ),
            (
                include_str!("dot_runtime/goldens/shape_hexagon_sized.dot"),
                include_str!("dot_runtime/goldens/shape_hexagon_sized.json"),
            ),
            (
                include_str!("dot_runtime/goldens/shape_circle_sized.dot"),
                include_str!("dot_runtime/goldens/shape_circle_sized.json"),
            ),
        ];
        let options = RenderOptions {
            language: crate::Language::Dot,
            ..Default::default()
        };
        // when and then
        for (source, oracle) in cases {
            let actual: Value = serde_json::from_str(&json0(source, &options).unwrap()).unwrap();
            let expected: Value = serde_json::from_str(oracle).unwrap();
            assert_eq!(actual, expected, "{source}");
        }
    }

    #[test]
    fn should_match_graphviz_161_shape_family_chain_json0() {
        // given
        let source = include_str!("dot_runtime/goldens/shape_family_chain.dot");
        let oracle: Value =
            serde_json::from_str(include_str!("dot_runtime/goldens/shape_family_chain.json"))
                .unwrap();
        let options = RenderOptions {
            language: crate::Language::Dot,
            ..Default::default()
        };
        // when
        let actual: Value = serde_json::from_str(&json0(source, &options).unwrap()).unwrap();
        // then
        assert_eq!(actual, oracle);
    }

    #[test]
    fn should_reject_unimplemented_polygon_incident_clipping() {
        // given
        let options = RenderOptions {
            language: crate::Language::Dot,
            ..Default::default()
        };
        let cases = ["triangle", "hexagon"];
        // when and then
        for shape in cases {
            let source = format!("digraph G {{ a [shape={shape}]; a -> b }}");
            assert!(
                matches!(json0(&source, &options), Err(RenderError::UnsupportedFeature(message)) if message.contains("unimplemented polygon clipping")),
                "{source}"
            );
        }
    }

    #[test]
    fn should_reject_diamond_with_incident_edges_until_clipping_matches_graphviz() {
        // given
        let source = "digraph G { a [shape=diamond]; a -> b }";
        // when
        let result = json0(source, &RenderOptions::default());
        // then
        assert!(
            matches!(result, Err(RenderError::UnsupportedFeature(message)) if message.contains("diamond node \"a\" with incident edges"))
        );
    }

    #[test]
    fn should_match_graphviz_161_self_loop_json_geometry() {
        // given
        let source = include_str!("../../tools/dot-parity/fixtures/self.dot");
        let oracle: Value =
            serde_json::from_str(include_str!("dot_runtime/goldens/self.json")).unwrap();
        let options = RenderOptions {
            language: crate::Language::Dot,
            ..Default::default()
        };
        // when
        let actual: Value = serde_json::from_str(&json0(source, &options).unwrap()).unwrap();
        // then
        assert_eq!(actual, oracle);
    }

    #[test]
    fn should_match_graphviz_161_ellipse_self_loop_json_geometry() {
        // given
        let source = include_str!("dot_runtime/goldens/ellipse-self.dot");
        let oracle: Value =
            serde_json::from_str(include_str!("dot_runtime/goldens/ellipse-self.json")).unwrap();
        let options = RenderOptions {
            language: crate::Language::Dot,
            ..Default::default()
        };
        // when
        let actual: Value = serde_json::from_str(&json0(source, &options).unwrap()).unwrap();
        // then
        assert_eq!(actual, oracle);
    }

    #[test]
    fn should_emit_graphviz_161_cluster_label_metadata() {
        // given
        let source = include_str!("../../tools/dot-parity/fixtures/subgraph_edge.dot");
        let options = RenderOptions {
            language: crate::Language::Dot,
            ..Default::default()
        };
        // when
        let actual: Value = serde_json::from_str(&json0(source, &options).unwrap()).unwrap();
        // then
        assert_eq!(actual["label"], "");
        assert_eq!(actual["objects"][0]["label"], "left");
        assert_eq!(actual["objects"][0]["lheight"], "0.23");
        assert_eq!(actual["objects"][0]["lwidth"], "0.26");
        assert_eq!(actual["objects"][1]["lwidth"], "0.37");
        assert_eq!(actual["objects"][2]["label"], "");
        assert_eq!(actual["objects"][3]["label"], "");
    }

    #[test]
    fn should_preserve_record_ports_and_graphviz_field_sizes() {
        // given
        let source = include_str!("../../tools/dot-parity/fixtures/ports.dot");
        let oracle: Value =
            serde_json::from_str(include_str!("dot_runtime/goldens/ports.json")).unwrap();
        let options = RenderOptions {
            language: crate::Language::Dot,
            ..Default::default()
        };
        // when
        let actual: Value = serde_json::from_str(&json0(source, &options).unwrap()).unwrap();
        // then
        for index in 0..2 {
            assert_eq!(
                actual["objects"][index]["width"],
                oracle["objects"][index]["width"]
            );
            assert_eq!(
                actual["objects"][index]["height"],
                oracle["objects"][index]["height"]
            );
            let actual_rects = actual["objects"][index]["rects"].as_str().unwrap();
            let oracle_rects = oracle["objects"][index]["rects"].as_str().unwrap();
            let parse = |rects: &str| {
                rects
                    .split_whitespace()
                    .map(|rect| {
                        rect.split(',')
                            .map(|value| value.parse::<f64>().unwrap())
                            .collect::<Vec<_>>()
                    })
                    .map(|rect| {
                        (
                            ((rect[2] - rect[0]) * 100.).round() as i64,
                            ((rect[3] - rect[1]) * 100.).round() as i64,
                        )
                    })
                    .collect::<Vec<_>>()
            };
            assert_eq!(parse(actual_rects), parse(oracle_rects));
        }
        assert_eq!(
            actual["edges"][0]["tailport"],
            oracle["edges"][0]["tailport"]
        );
        assert_eq!(
            actual["edges"][0]["headport"],
            oracle["edges"][0]["headport"]
        );
    }

    #[test]
    fn should_match_graphviz_161_ports_json_geometry() {
        // given
        let source = include_str!("../../tools/dot-parity/fixtures/ports.dot");
        let oracle: Value =
            serde_json::from_str(include_str!("dot_runtime/goldens/ports.json")).unwrap();
        let options = RenderOptions {
            language: crate::Language::Dot,
            ..Default::default()
        };
        // when
        let actual: Value = serde_json::from_str(&json0(source, &options).unwrap()).unwrap();
        // then
        assert_eq!(actual, oracle);
    }

    #[test]
    fn should_match_graphviz_161_nested_record_json0() {
        // given
        let cases = [
            (
                include_str!("dot_runtime/goldens/record_nested_horizontal.dot"),
                include_str!("dot_runtime/goldens/record_nested_horizontal.json"),
            ),
            (
                include_str!("dot_runtime/goldens/record_nested_vertical.dot"),
                include_str!("dot_runtime/goldens/record_nested_vertical.json"),
            ),
            (
                include_str!("dot_runtime/goldens/record_nested_ports.dot"),
                include_str!("dot_runtime/goldens/record_nested_ports.json"),
            ),
            (
                include_str!("dot_runtime/goldens/record_nested_expanded.dot"),
                include_str!("dot_runtime/goldens/record_nested_expanded.json"),
            ),
        ];
        let options = RenderOptions {
            language: crate::Language::Dot,
            ..Default::default()
        };
        // when and then
        for (source, oracle) in cases {
            let actual: Value = serde_json::from_str(&json0(source, &options).unwrap()).unwrap();
            let expected: Value = serde_json::from_str(oracle).unwrap();
            assert_eq!(actual, expected, "{source}");
        }
    }

    #[test]
    fn should_match_graphviz_161_multiline_label_json0() {
        // given
        let cases = [
            (
                include_str!("dot_runtime/goldens/multiline_node_center.dot"),
                include_str!("dot_runtime/goldens/multiline_node_center.json"),
            ),
            (
                include_str!("dot_runtime/goldens/multiline_node_left.dot"),
                include_str!("dot_runtime/goldens/multiline_node_left.json"),
            ),
            (
                include_str!("dot_runtime/goldens/multiline_node_right.dot"),
                include_str!("dot_runtime/goldens/multiline_node_right.json"),
            ),
            (
                include_str!("dot_runtime/goldens/multiline_edge_center.dot"),
                include_str!("dot_runtime/goldens/multiline_edge_center.json"),
            ),
            (
                include_str!("dot_runtime/goldens/multiline_multiline_feature.dot"),
                include_str!("dot_runtime/goldens/multiline_multiline_feature.json"),
            ),
        ];
        let options = RenderOptions {
            language: crate::Language::Dot,
            ..Default::default()
        };
        // when and then
        for (source, oracle) in cases {
            let actual: Value = serde_json::from_str(&json0(source, &options).unwrap()).unwrap();
            let expected: Value = serde_json::from_str(oracle).unwrap();
            assert_eq!(actual, expected, "{source}");
        }
    }

    #[test]
    fn should_reject_nested_record_parallel_edges_without_compass_routes() {
        // given
        let source = include_str!("../../tools/dot-parity/fixtures-features/record_nested.dot");
        let options = RenderOptions {
            language: crate::Language::Dot,
            ..Default::default()
        };
        // when
        let result = json0(source, &options);
        // then
        assert!(
            matches!(result, Err(RenderError::UnsupportedFeature(message)) if message.contains("nested record ports requires one edge"))
        );
    }

    #[test]
    fn should_match_expat_graphviz_161_html_table_json0() {
        // given
        let source = include_str!("../../tools/dot-parity/fixtures/html.dot");
        let oracle: Value =
            serde_json::from_str(include_str!("dot_runtime/goldens/html-expat.json")).unwrap();
        let options = RenderOptions {
            language: crate::Language::Dot,
            ..Default::default()
        };
        // when
        let actual: Value = serde_json::from_str(&json0(source, &options).unwrap()).unwrap();
        // then
        assert_eq!(actual, oracle);
    }

    #[test]
    fn should_match_graphviz_161_graph_label_and_background_json() {
        // given
        let cases = [
            (
                "digraph G { label=\"Hello\"; a -> b }",
                include_str!("dot_runtime/goldens/graph_label.json"),
            ),
            (
                "digraph G { label=\"Graph label longer than both nodes\"; a -> b }",
                include_str!("dot_runtime/goldens/graph_label_long.json"),
            ),
            (
                "digraph G { label=\"Hello\"; labelloc=t; a -> b }",
                include_str!("dot_runtime/goldens/graph_label_top.json"),
            ),
            (
                "digraph G { label=\"Hello\"; labeljust=l; a -> b }",
                include_str!("dot_runtime/goldens/graph_label_left.json"),
            ),
            (
                "digraph G { bgcolor=lightblue; a -> b }",
                include_str!("dot_runtime/goldens/graph_background.json"),
            ),
            (
                "digraph G { label=\"Hello\"; bgcolor=lightblue; a -> b }",
                include_str!("dot_runtime/goldens/graph_label_background.json"),
            ),
            (
                "digraph G { bgcolor=transparent; a }",
                include_str!("dot_runtime/goldens/graph_transparent.json"),
            ),
            (
                "digraph G { label=\"first\\nsecond\"; a -> b }",
                include_str!("dot_runtime/goldens/graph_label_multiline.json"),
            ),
            (
                "digraph G { rankdir=BT; label=\"Hello\"; a -> b }",
                include_str!("dot_runtime/goldens/graph_label_bt.json"),
            ),
            (
                "digraph G { rankdir=RL; label=\"Hello\"; a -> b }",
                include_str!("dot_runtime/goldens/graph_label_rl.json"),
            ),
        ];
        let options = RenderOptions {
            language: crate::Language::Dot,
            ..Default::default()
        };
        // when and then
        for (source, oracle) in cases {
            let actual: Value = serde_json::from_str(&json0(source, &options).unwrap()).unwrap();
            let oracle: Value = serde_json::from_str(oracle).unwrap();
            assert_eq!(actual, oracle, "{source}");
        }
    }

    #[test]
    fn should_reject_graph_html_label_and_unmeasured_font() {
        // given
        let cases = [
            ("digraph G { label=<<B>bold</B>>; a }", "HTML labels"),
            (
                "digraph G { graph [label=Hello,fontname=Helvetica]; a }",
                "Helvetica",
            ),
        ];
        let options = RenderOptions {
            language: crate::Language::Dot,
            ..Default::default()
        };
        // when and then
        for (source, expected) in cases {
            let json = json0(source, &options);
            let svg = render(source, &options);
            assert!(
                matches!(json, Err(RenderError::UnsupportedFeature(message)) if message.contains(expected)),
                "{source}"
            );
            assert!(
                matches!(svg, Err(RenderError::UnsupportedFeature(message)) if message.contains(expected)),
                "{source}"
            );
        }
    }

    #[test]
    fn should_match_graphviz_161_anonymous_subgraph_ids_after_edges() {
        // given
        let cases = [
            (
                include_str!("../../tools/dot-parity/fixtures-features/rank_same.dot"),
                include_str!("dot_runtime/goldens/rank_same.json"),
            ),
            (
                include_str!("../../tools/dot-parity/fixtures-features/rank_minmax.dot"),
                include_str!("dot_runtime/goldens/rank_minmax.json"),
            ),
        ];
        let options = RenderOptions {
            language: crate::Language::Dot,
            ..Default::default()
        };
        // when and then
        for (source, oracle) in cases {
            let actual: Value = serde_json::from_str(&json0(source, &options).unwrap()).unwrap();
            let oracle: Value = serde_json::from_str(oracle).unwrap();
            assert_eq!(actual, oracle, "{source}");
        }
    }

    #[test]
    fn should_distinguish_html_label_from_quoted_angle_brackets() {
        // given
        let source = r#"digraph G { a [label=<<B>x</B>>]; b [label="<B>x</B>"]; }"#;
        let document = dot::parse(source, &dot::ParseOptions::default()).unwrap();
        let options = RenderOptions {
            language: crate::Language::Dot,
            ..Default::default()
        };
        let mut work = Work::new(&options);
        // when
        let model = lower(&document, &mut work).unwrap();
        // then
        assert_eq!(model.nodes[0].label_kind, IdKind::Html);
        assert_eq!(effective_node_label(&model.nodes[0]), "<B>x</B>");
        assert!(matches!(
            html_table::HtmlTable::parse("<B>x</B>", &model.nodes[0].attrs),
            Err(RenderError::UnsupportedFeature(_))
        ));
        assert_eq!(model.nodes[1].label_kind, IdKind::Quoted);
        assert_eq!(effective_node_label(&model.nodes[1]), "<B>x</B>");
    }

    #[test]
    fn should_reject_overlong_port_identifier() {
        // given
        let source = format!("digraph G {{ a:\"{}\" -> b }}", "x".repeat(129));
        let options = RenderOptions {
            language: crate::Language::Dot,
            ..Default::default()
        };
        // when
        let result = json0(&source, &options);
        // then
        assert!(
            matches!(result, Err(RenderError::InvalidSource(message)) if message.contains("overlong identifier") && message.contains("a"))
        );
    }

    #[test]
    fn should_reject_invalid_compass_on_port() {
        // given
        let source = "digraph G { a:p:sideways -> b }";
        let options = RenderOptions {
            language: crate::Language::Dot,
            ..Default::default()
        };
        // when
        let result = json0(source, &options);
        // then
        assert!(
            matches!(result, Err(RenderError::InvalidSource(message)) if message.contains("compass") && message.contains("sideways"))
        );
    }

    #[test]
    fn should_match_graphviz_161_undirected_default_node_sizes() {
        // given
        let source = include_str!("../../tools/dot-parity/fixtures/undirected.dot");
        let options = RenderOptions {
            language: crate::Language::Dot,
            ..Default::default()
        };
        // when
        let actual: Value = serde_json::from_str(&json0(source, &options).unwrap()).unwrap();
        // then
        assert_eq!(actual["directed"], false);
        assert_eq!(actual["strict"], true);
        for node in actual["objects"].as_array().unwrap() {
            assert_eq!(node["width"], "0.75");
            assert_eq!(node["height"], "0.5");
            assert_eq!(node["shape"], "ellipse");
        }
    }
}
