//! Bounded in-process layered layout inspired by DOT rank, order, and position phases.
use super::budget::Budget;
use crate::RenderError;
use dugong::{EdgeLabel, GraphLabel, NodeLabel, Point, RankDir, graphlib::Graph};
use std::collections::{BTreeMap, HashMap};

pub(crate) type LayoutGraph = Graph<NodeLabel, EdgeLabel, GraphLabel>;

pub(crate) trait LayoutWork {
    fn charge(&mut self, units: usize) -> Result<(), RenderError>;
    fn depth(&self, depth: usize) -> Result<(), RenderError>;
}

impl LayoutWork for Budget {
    fn charge(&mut self, units: usize) -> Result<(), RenderError> {
        Budget::charge(self, units)
    }
    fn depth(&self, depth: usize) -> Result<(), RenderError> {
        Budget::depth(self, depth)
    }
}

#[derive(Clone, Copy)]
struct Size {
    width: f64,
    height: f64,
}

fn node_size(node: &NodeLabel, flavor: LayoutFlavor) -> Size {
    match flavor {
        LayoutFlavor::PlantUmlGenerated => Size {
            width: node.width.max(1.),
            height: node.height.max(1.),
        },
        LayoutFlavor::RawDot => Size {
            width: node.width.max(0.72),
            height: node.height.max(1.44),
        },
    }
}
#[derive(Clone, Copy)]
struct Placed {
    x: f64,
    y: f64,
    size: Size,
}
struct Unit {
    id: String,
    size: Size,
    nodes: HashMap<String, Placed>,
}
struct Group {
    size: Size,
    nodes: HashMap<String, Placed>,
}

#[derive(Clone, Copy)]
struct Rect {
    left: f64,
    top: f64,
    right: f64,
    bottom: f64,
}

impl Rect {
    fn union(self, other: Self) -> Self {
        Self {
            left: self.left.min(other.left),
            top: self.top.min(other.top),
            right: self.right.max(other.right),
            bottom: self.bottom.max(other.bottom),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LayoutFlavor {
    PlantUmlGenerated,
    RawDot,
}

#[derive(Clone, Copy)]
struct Link {
    from: usize,
    to: usize,
    source: Point,
    target: Point,
    label_width: f64,
    label_height: f64,
    label_present: bool,
    minlen: usize,
    weight: f64,
    constraint: bool,
    port: bool,
}

#[derive(Clone, Copy)]
struct VirtualLabelUnit {
    rank: usize,
    left_width: f64,
    right_width: f64,
}

impl VirtualLabelUnit {
    fn separation_after(self, previous: Self) -> f64 {
        (previous.right_width + self.left_width + 5.).round()
    }
}

pub(super) fn layout(graph: &mut LayoutGraph, budget: &mut Budget) -> Result<(), RenderError> {
    layout_with_control(graph, budget, LayoutFlavor::PlantUmlGenerated)
}

pub(crate) fn layout_with_control(
    graph: &mut LayoutGraph,
    budget: &mut dyn LayoutWork,
    flavor: LayoutFlavor,
) -> Result<(), RenderError> {
    layout_with_cluster_labels(graph, budget, flavor, &BTreeMap::new())
}

pub(crate) fn layout_raw_dot_with_cluster_labels(
    graph: &mut LayoutGraph,
    budget: &mut dyn LayoutWork,
    cluster_label_widths: &BTreeMap<String, f64>,
) -> Result<(), RenderError> {
    layout_with_cluster_labels(graph, budget, LayoutFlavor::RawDot, cluster_label_widths)
}

fn layout_with_cluster_labels(
    graph: &mut LayoutGraph,
    budget: &mut dyn LayoutWork,
    flavor: LayoutFlavor,
    cluster_label_widths: &BTreeMap<String, f64>,
) -> Result<(), RenderError> {
    let group = if matches!(flavor, LayoutFlavor::RawDot) && graph.nodes().any(is_dot_cluster) {
        layout_raw_clusters(graph, budget, cluster_label_widths)?
    } else {
        group(graph, None, 0, budget, flavor)?
    };
    let margin = (0., 0.);
    for (id, placed) in group.nodes {
        let node = graph
            .node_mut(&id)
            .ok_or_else(|| failure("placed node disappeared"))?;
        node.x = Some(placed.x + placed.size.width / 2. + margin.0);
        node.y = Some(placed.y + placed.size.height / 2. + margin.1);
        node.width = placed.size.width;
        node.height = placed.size.height;
    }
    graph.graph_mut().width = group.size.width + 2. * margin.0;
    graph.graph_mut().height = group.size.height + 2. * margin.1;
    route(graph, budget, flavor)?;
    if matches!(flavor, LayoutFlavor::RawDot) {
        include_edge_label_right_bounds(graph, budget)?;
    }
    Ok(())
}

fn include_edge_label_right_bounds(
    graph: &mut LayoutGraph,
    budget: &mut dyn LayoutWork,
) -> Result<(), RenderError> {
    let mut right = graph.graph().width;
    for key in graph.edges() {
        budget.charge(1)?;
        let label = graph
            .edge_by_key(key)
            .ok_or_else(|| failure("edge disappeared while finding graph bounds"))?;
        let present = label
            .extras
            .get("dot_label_present")
            .and_then(|value| value.as_bool())
            .unwrap_or(false);
        if present {
            right = right.max(label.x.unwrap_or(0.) + label.width / 2.);
        }
    }
    graph.graph_mut().width = right;
    Ok(())
}

fn group(
    graph: &LayoutGraph,
    parent: Option<&str>,
    depth: usize,
    budget: &mut dyn LayoutWork,
    flavor: LayoutFlavor,
) -> Result<Group, RenderError> {
    budget.depth(depth)?;
    let children = match parent {
        Some(id) => graph.children(id),
        None => graph.children_root(),
    };
    let mut units = Vec::with_capacity(children.len() + usize::from(parent.is_some()));
    if let Some(id) = parent {
        let node = graph
            .node(id)
            .ok_or_else(|| failure("cluster owner disappeared"))?;
        units.push(Unit {
            id: id.to_owned(),
            size: node_size(node, flavor),
            nodes: HashMap::new(),
        });
    }
    for id in children {
        budget.charge(1)?;
        units.push(unit(graph, id, depth + 1, budget, flavor)?);
    }
    place(graph, parent, units, budget, flavor)
}

fn unit(
    graph: &LayoutGraph,
    id: &str,
    depth: usize,
    budget: &mut dyn LayoutWork,
    flavor: LayoutFlavor,
) -> Result<Unit, RenderError> {
    let node = graph
        .node(id)
        .ok_or_else(|| failure("layout node disappeared"))?;
    let mut size = node_size(node, flavor);
    if graph.children(id).is_empty() {
        return Ok(Unit {
            id: id.into(),
            size,
            nodes: HashMap::from([(id.into(), Placed { x: 0., y: 0., size })]),
        });
    }
    let inner = group(graph, Some(id), depth, budget, flavor)?;
    size.width = size.width.max(inner.size.width + 24.);
    size.height = size.height.max(inner.size.height + 24.);
    let mut nodes = inner.nodes;
    for child in nodes.values_mut() {
        child.x += (size.width - inner.size.width) / 2.;
        child.y += (size.height - inner.size.height) / 2.;
    }
    nodes.insert(id.into(), Placed { x: 0., y: 0., size });
    Ok(Unit {
        id: id.into(),
        size,
        nodes,
    })
}

fn ancestor<'a>(graph: &'a LayoutGraph, id: &'a str, parent: Option<&str>) -> Option<&'a str> {
    let mut current = id;
    loop {
        if Some(current) == parent {
            return Some(current);
        }
        if graph.parent(current) == parent {
            return Some(current);
        }
        current = graph.parent(current)?;
    }
}

fn is_dot_cluster(id: &str) -> bool {
    id.starts_with("__ive_dot_cluster_")
}

fn layout_raw_clusters(
    graph: &LayoutGraph,
    budget: &mut dyn LayoutWork,
    cluster_label_widths: &BTreeMap<String, f64>,
) -> Result<Group, RenderError> {
    let units = graph
        .nodes()
        .filter(|id| !is_dot_cluster(id))
        .map(|id| unit(graph, id, 0, budget, LayoutFlavor::RawDot))
        .collect::<Result<Vec<_>, _>>()?;
    let links = links(graph, None, &units, budget, LayoutFlavor::RawDot)?;
    let ranks = ranks(units.len(), &links, budget)?;
    let mut rows = vec![Vec::new(); ranks.iter().copied().max().unwrap_or(0) + 1];
    for (index, rank) in ranks.iter().enumerate() {
        rows[*rank].push(index);
    }
    order(&mut rows, &links, units.len(), budget)?;
    for row in &mut rows {
        let parent = row
            .first()
            .and_then(|&index| graph.parent(&units[index].id));
        if parent.is_some_and(is_dot_cluster)
            && row
                .iter()
                .all(|&index| graph.parent(&units[index].id) == parent)
        {
            row.sort_unstable();
        }
    }
    let mut group = position(graph, units, rows, &links, budget, LayoutFlavor::RawDot)?;
    separate_sibling_clusters(graph, &mut group.nodes, cluster_label_widths, budget)?;
    let clusters = graph
        .nodes()
        .filter(|id| is_dot_cluster(id))
        .map(|id| {
            (
                id.to_owned(),
                cluster_rect(graph, id, &group.nodes, cluster_label_widths, 0, budget),
            )
        })
        .map(|(id, rect)| rect.map(|rect| (id, rect)))
        .collect::<Result<Vec<_>, _>>()?;
    let leaves = group.nodes.values().map(placed_rect);
    let bounds = leaves
        .chain(clusters.iter().map(|(_, rect)| *rect))
        .reduce(Rect::union)
        .ok_or_else(|| failure("DOT cluster graph has no nodes"))?;
    let touches = |edge: f64, member: fn(Rect) -> f64| {
        clusters
            .iter()
            .any(|(_, rect)| (member(*rect) - edge).abs() < 1e-9)
    };
    let left_pad = if touches(bounds.left, |rect| rect.left) {
        8.
    } else {
        0.
    };
    let top_pad = if touches(bounds.top, |rect| rect.top) {
        8.
    } else {
        0.
    };
    let right_pad = if touches(bounds.right, |rect| rect.right) {
        8.
    } else {
        0.
    };
    let bottom_pad = if touches(bounds.bottom, |rect| rect.bottom) {
        8.
    } else {
        0.
    };
    let dx = left_pad - bounds.left;
    let dy = top_pad - bounds.top;
    for placed in group.nodes.values_mut() {
        placed.x += dx;
        placed.y += dy;
    }
    for (id, rect) in clusters {
        group.nodes.insert(
            id,
            Placed {
                x: rect.left + dx,
                y: rect.top + dy,
                size: Size {
                    width: rect.right - rect.left,
                    height: rect.bottom - rect.top,
                },
            },
        );
    }
    // Graphviz's set_xcoords uses integer auxiliary ranks for nodes enclosed
    // by nested integer cluster boundaries (position.c).
    let nested_clusters = graph
        .nodes()
        .filter(|id| is_dot_cluster(id))
        .any(|id| graph.parent(id).is_some_and(is_dot_cluster));
    if nested_clusters {
        budget.charge(group.nodes.len())?;
        for (id, placed) in &mut group.nodes {
            if is_dot_cluster(id) {
                continue;
            }
            let center = placed.x + placed.size.width / 2.;
            placed.x += center.round() - center;
        }
    }
    align_single_cluster_chain_to_integer_bounds(graph, &mut group.nodes, budget)?;
    group.size = Size {
        width: bounds.right - bounds.left + left_pad + right_pad,
        height: bounds.bottom - bounds.top + top_pad + bottom_pad,
    };
    Ok(group)
}

fn align_single_cluster_chain_to_integer_bounds(
    graph: &LayoutGraph,
    placed: &mut HashMap<String, Placed>,
    budget: &mut dyn LayoutWork,
) -> Result<(), RenderError> {
    let clusters = graph
        .nodes()
        .filter(|id| is_dot_cluster(id))
        .collect::<Vec<_>>();
    if clusters.len() != 1 || !matches!(graph.graph().rankdir, RankDir::TB) {
        return Ok(());
    }
    let cluster = clusters[0];
    let inside = graph.children(cluster);
    let outside = graph
        .children_root()
        .into_iter()
        .filter(|id| !is_dot_cluster(id))
        .collect::<Vec<_>>();
    if inside.len() != 2 || outside.len() != 1 || graph.edges().count() != 2 {
        return Ok(());
    }
    let source = outside[0];
    let first = graph
        .edges()
        .find(|edge| edge.v == source && inside.contains(&edge.w.as_str()));
    let Some(first) = first else { return Ok(()) };
    let target = inside.iter().copied().find(|id| *id != first.w);
    let Some(target) = target else { return Ok(()) };
    if !graph
        .edges()
        .any(|edge| edge.v == first.w && edge.w == target)
    {
        return Ok(());
    }
    budget.charge(3)?;
    let boundary = placed
        .get(cluster)
        .ok_or_else(|| failure("DOT cluster has no placement"))?
        .x;
    let margin = graph
        .node(cluster)
        .ok_or_else(|| failure("DOT cluster disappeared"))?
        .width;
    let anchor = [first.w.as_str(), target].map(|id| {
        placed
            .get(id)
            .map(|node| boundary + margin + (node.size.width / 2.).round())
    });
    let [Some(first_x), Some(target_x)] = anchor else {
        return Ok(());
    };
    let widths = [first.w.as_str(), target].map(|id| placed.get(id).map(|node| node.size.width));
    let [Some(first_width), Some(target_width)] = widths else {
        return Ok(());
    };
    // contain_nodes rounds half-widths into auxiliary ranks; this bounded
    // correction applies when the second rank needs exactly one more point.
    if (target_width / 2.).round() - (first_width / 2.).round() != 1. {
        return Ok(());
    }
    let centers = [source, first.w.as_str(), target]
        .map(|id| placed.get(id).map(|node| node.x + node.size.width / 2.));
    if centers
        .iter()
        .any(|center| center.is_none_or(|x| (x - centers[0].unwrap()).abs() > 1.5))
    {
        return Ok(());
    }
    for (id, x) in [
        (source, first_x),
        (first.w.as_str(), first_x),
        (target, target_x),
    ] {
        if let Some(node) = placed.get_mut(id) {
            node.x = x - node.size.width / 2.;
        }
    }
    Ok(())
}

fn placed_rect(placed: &Placed) -> Rect {
    Rect {
        left: placed.x,
        top: placed.y,
        right: placed.x + placed.size.width,
        bottom: placed.y + placed.size.height,
    }
}

fn cluster_rect(
    graph: &LayoutGraph,
    id: &str,
    placed: &HashMap<String, Placed>,
    cluster_label_widths: &BTreeMap<String, f64>,
    depth: usize,
    budget: &mut dyn LayoutWork,
) -> Result<Rect, RenderError> {
    budget.depth(depth)?;
    let children = graph.children(id);
    budget.charge(children.len())?;
    let content = children
        .iter()
        .map(|child| {
            if is_dot_cluster(child) {
                cluster_rect(
                    graph,
                    child,
                    placed,
                    cluster_label_widths,
                    depth + 1,
                    budget,
                )
            } else {
                placed
                    .get(*child)
                    .map(placed_rect)
                    .ok_or_else(|| failure("DOT cluster child has no placement"))
            }
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .reduce(Rect::union)
        .ok_or_else(|| failure("DOT cluster has no children"))?;
    let owner = graph
        .node(id)
        .ok_or_else(|| failure("DOT cluster owner disappeared"))?;
    let margin = owner.width;
    let label_space = if owner.height > 0. {
        owner.height + 8.
    } else {
        0.
    };
    // Graphviz stores cluster left/right boundaries in integer auxiliary
    // ranks (position.c dot_compute_bb), even for fractional text widths.
    let left = (content.left - margin).round();
    // Graphviz pads the cluster label by four GAP units on both axes before
    // widening the enclosing cluster. The extra width grows its right edge;
    // contained nodes keep their positions.
    let padded_label_width = (cluster_label_widths.get(id).copied().unwrap_or(0.) + 16.).round();
    let right = (content.right + margin)
        .max(left + padded_label_width)
        .round();
    Ok(Rect {
        left,
        top: content.top - margin - label_space,
        right,
        bottom: content.bottom + margin,
    })
}

fn separate_sibling_clusters(
    graph: &LayoutGraph,
    placed: &mut HashMap<String, Placed>,
    cluster_label_widths: &BTreeMap<String, f64>,
    budget: &mut dyn LayoutWork,
) -> Result<(), RenderError> {
    let mut roots = graph
        .nodes()
        .filter(|id| is_dot_cluster(id) && graph.parent(id).is_none())
        .map(|id| {
            (
                id.to_owned(),
                cluster_rect(graph, id, placed, cluster_label_widths, 0, budget),
            )
        })
        .map(|(id, rect)| rect.map(|rect| (id, rect)))
        .collect::<Result<Vec<_>, _>>()?;
    roots.sort_by(|a, b| a.1.top.total_cmp(&b.1.top));
    for pair in roots.windows(2) {
        let (previous, next) = (&pair[0], &pair[1]);
        if previous.1.right <= next.1.left || next.1.right <= previous.1.left {
            continue;
        }
        let needed = previous.1.bottom + 8. - next.1.top;
        if needed <= 0. {
            continue;
        }
        budget.charge(placed.len())?;
        for (id, node) in placed.iter_mut() {
            if cluster_contains(graph, &next.0, id) {
                node.y += needed;
            }
        }
    }
    Ok(())
}

fn cluster_contains(graph: &LayoutGraph, cluster: &str, node: &str) -> bool {
    let mut parent = graph.parent(node);
    while let Some(id) = parent {
        if id == cluster {
            return true;
        }
        parent = graph.parent(id);
    }
    false
}

fn links(
    graph: &LayoutGraph,
    parent: Option<&str>,
    units: &[Unit],
    budget: &mut dyn LayoutWork,
    flavor: LayoutFlavor,
) -> Result<Vec<Link>, RenderError> {
    let index: HashMap<_, _> = units
        .iter()
        .enumerate()
        .map(|(i, unit)| (unit.id.as_str(), i))
        .collect();
    let mut links = Vec::new();
    for edge in graph.edges() {
        budget.charge(1)?;
        let from = ancestor(graph, &edge.v, parent)
            .and_then(|id| index.get(id))
            .or_else(|| index.get(edge.v.as_str()));
        let to = ancestor(graph, &edge.w, parent)
            .and_then(|id| index.get(id))
            .or_else(|| index.get(edge.w.as_str()));
        if let (Some(&a), Some(&b)) = (from, to)
            && a != b
        {
            let label = graph
                .edge_by_key(edge)
                .ok_or_else(|| failure("layout edge disappeared"))?;
            let (label_width, label_height, label_present) = match flavor {
                LayoutFlavor::PlantUmlGenerated => {
                    let font = label.height.clamp(12., 80.);
                    let columns = (label.width / 7.).ceil().clamp(1., 80.);
                    (columns * font * 0.6, font * 1.2, true)
                }
                LayoutFlavor::RawDot => (
                    label.width,
                    label.height,
                    label
                        .extras
                        .get("dot_label_present")
                        .and_then(|value| value.as_bool())
                        .unwrap_or(false),
                ),
            };
            if label.minlen > 1024 {
                return Err(failure(&format!(
                    "edge {:?}->{:?} minlen {} exceeds 1024",
                    edge.v, edge.w, label.minlen
                )));
            }
            budget.charge(label.minlen)?;
            let constraint = label
                .extras
                .get("constraint")
                .and_then(|value| value.as_bool())
                .unwrap_or(true);
            let weight = if matches!(flavor, LayoutFlavor::PlantUmlGenerated) {
                1.
            } else {
                label.weight.max(0.)
            };
            let source_port = record_port_offset(label, "tail");
            let target_port = record_port_offset(label, "head");
            let mut source = anchor(&units[a], &edge.v);
            let mut target = anchor(&units[b], &edge.w);
            if let Some(offset) = source_port {
                source.x += offset.x;
                source.y += offset.y;
            }
            if let Some(offset) = target_port {
                target.x += offset.x;
                target.y += offset.y;
            }
            links.push(Link {
                from: a,
                to: b,
                source,
                target,
                label_width,
                label_height,
                label_present,
                minlen: label.minlen,
                weight,
                constraint,
                port: source_port.is_some() && target_port.is_some(),
            });
        }
    }
    Ok(links)
}

fn anchor(unit: &Unit, id: &str) -> Point {
    if let Some(node) = unit.nodes.get(id) {
        return Point {
            x: node.x + node.size.width / 2.,
            y: node.y + node.size.height / 2.,
        };
    }
    Point {
        x: unit.size.width / 2.,
        y: unit.size.height / 2.,
    }
}

fn record_port_offset(label: &EdgeLabel, endpoint: &str) -> Option<Point> {
    let port_key = format!("dot_{endpoint}_port");
    let fields_key = format!("dot_{endpoint}_record_fields");
    let (name, compass) = label.extras.get(&port_key)?.as_str()?.split_once(':')?;
    let field = label
        .extras
        .get(&fields_key)?
        .as_array()?
        .iter()
        .find(|field| field.get("port").and_then(|value| value.as_str()) == Some(name))?;
    let x = field.get("x")?.as_f64()?;
    let y = field.get("y")?.as_f64()?;
    let width = field.get("width")?.as_f64()?;
    let height = field.get("height")?.as_f64()?;
    match compass {
        "e" => Some(Point {
            x: x + width / 2.,
            y,
        }),
        "w" => Some(Point {
            x: x - width / 2.,
            y,
        }),
        "n" => Some(Point {
            x,
            y: y - height / 2.,
        }),
        "s" => Some(Point {
            x,
            y: y + height / 2.,
        }),
        _ => None,
    }
}

fn ranks(
    count: usize,
    links: &[Link],
    budget: &mut dyn LayoutWork,
) -> Result<Vec<usize>, RenderError> {
    let mut remaining = vec![true; count];
    let mut indegree = vec![0usize; count];
    let mut rank = vec![0usize; count];
    for link in links.iter().filter(|link| link.constraint) {
        indegree[link.to] += 1;
    }
    for _ in 0..count {
        budget.charge(count + links.len())?;
        let chosen = (0..count)
            .filter(|&i| remaining[i])
            .min_by_key(|&i| (indegree[i], i))
            .ok_or_else(|| failure("rank assignment lost a node"))?;
        remaining[chosen] = false;
        for link in links.iter().filter(|link| link.constraint) {
            if link.from == chosen && remaining[link.to] {
                indegree[link.to] = indegree[link.to].saturating_sub(1);
                rank[link.to] = rank[link.to].max(rank[link.from].saturating_add(link.minlen));
            }
        }
    }
    Ok(rank)
}

fn order(
    rows: &mut [Vec<usize>],
    links: &[Link],
    count: usize,
    budget: &mut dyn LayoutWork,
) -> Result<(), RenderError> {
    budget.charge(count + 2 * links.len())?;
    let mut neighbors = vec![Vec::new(); count];
    for link in links {
        neighbors[link.from].push((link.to, link.weight));
        neighbors[link.to].push((link.from, link.weight));
    }
    let mut positions = vec![0.; count];
    for sweep in 0..4 {
        let sequence: Vec<_> = if sweep % 2 == 0 {
            (0..rows.len()).collect()
        } else {
            (0..rows.len()).rev().collect()
        };
        for rank in sequence {
            let comparison_work = rows[rank].len() * (rows[rank].len().max(2).ilog2() as usize + 1);
            let neighbor_work = rows[rank]
                .iter()
                .map(|&node| neighbors[node].len())
                .sum::<usize>();
            budget.charge(comparison_work + neighbor_work)?;
            let mut scored = rows[rank]
                .iter()
                .map(|&node| {
                    let adjacent = &neighbors[node];
                    let total = adjacent.iter().map(|&(_, weight)| weight).sum::<f64>();
                    let score = if total > 0. {
                        adjacent
                            .iter()
                            .map(|&(next, weight)| positions[next] * weight)
                            .sum::<f64>()
                            / total
                    } else {
                        positions[node]
                    };
                    (node, score)
                })
                .collect::<Vec<_>>();
            scored.sort_by(|a, b| a.1.total_cmp(&b.1).then_with(|| a.0.cmp(&b.0)));
            for (position, (node, _)) in scored.into_iter().enumerate() {
                rows[rank][position] = node;
                positions[node] = position as f64;
            }
        }
    }
    Ok(())
}

fn place(
    graph: &LayoutGraph,
    parent: Option<&str>,
    units: Vec<Unit>,
    budget: &mut dyn LayoutWork,
    flavor: LayoutFlavor,
) -> Result<Group, RenderError> {
    if units.is_empty() {
        return Ok(Group {
            size: Size {
                width: 0.,
                height: 0.,
            },
            nodes: HashMap::new(),
        });
    }
    let links = links(graph, parent, &units, budget, flavor)?;
    let ranks = ranks(units.len(), &links, budget)?;
    let mut rows = vec![Vec::new(); ranks.iter().copied().max().unwrap_or(0) + 1];
    for (index, rank) in ranks.iter().enumerate() {
        rows[*rank].push(index);
    }
    order(&mut rows, &links, units.len(), budget)?;
    position(graph, units, rows, &links, budget, flavor)
}

fn position(
    graph: &LayoutGraph,
    units: Vec<Unit>,
    rows: Vec<Vec<usize>>,
    links: &[Link],
    budget: &mut dyn LayoutWork,
    flavor: LayoutFlavor,
) -> Result<Group, RenderError> {
    let direction = graph.graph().rankdir;
    let horizontal = matches!(direction, RankDir::LR | RankDir::RL);
    let primary = |size: Size| if horizontal { size.width } else { size.height };
    let secondary = |size: Size| if horizontal { size.height } else { size.width };
    let gap = match flavor {
        LayoutFlavor::PlantUmlGenerated => graph.graph().nodesep.max(16.),
        LayoutFlavor::RawDot => graph.graph().nodesep.max(0.),
    };
    let mut rank_of = vec![0; units.len()];
    for (rank, row) in rows.iter().enumerate() {
        for &node in row {
            rank_of[node] = rank;
        }
    }
    let ranksep = graph.graph().ranksep;
    let mut rank_gaps = vec![ranksep; rows.len().saturating_sub(1)];
    let adjusted_ranksep = ((ranksep + 1.) / 2.).floor() * 2.;
    for link in links {
        let start = rank_of[link.from].min(rank_of[link.to]);
        let end = rank_of[link.from].max(rank_of[link.to]);
        let span = end.saturating_sub(start);
        if span == 0 {
            continue;
        }
        if !graph.is_directed() && span > 1 {
            continue;
        }
        let label_size = if horizontal {
            link.label_width
        } else {
            link.label_height
        };
        let total = if link.label_present {
            adjusted_ranksep + label_size + (span - 1) as f64 * (adjusted_ranksep + 2.)
        } else {
            ranksep + (span - 1) as f64 * (ranksep + 1.)
        };
        let per_gap = total / span as f64;
        for gap in rank_gaps.iter_mut().take(end).skip(start) {
            *gap = gap.max(per_gap);
        }
    }
    if matches!(flavor, LayoutFlavor::RawDot) && matches!(direction, RankDir::TB) {
        for rank in 1..rows.len() {
            let cluster_space = cluster_entry_space(graph, &units, &rows, rank, budget)?;
            let unlabeled_virtual_rank = links.iter().any(|link| {
                rows[rank - 1].contains(&link.from)
                    && rows[rank].contains(&link.to)
                    && !link.label_present
            });
            rank_gaps[rank - 1] +=
                cluster_space + f64::from(cluster_space > 0. && unlabeled_virtual_rank);
        }
    }
    budget.charge(graph.edges().count())?;
    let mut self_space = vec![0.; units.len()];
    let index: HashMap<_, _> = units
        .iter()
        .enumerate()
        .map(|(i, unit)| (unit.id.as_str(), i))
        .collect();
    for edge in graph.edges().filter(|edge| edge.v == edge.w) {
        if let (Some(&node), Some(label)) = (index.get(edge.v.as_str()), graph.edge_by_key(edge)) {
            let label_width = match flavor {
                LayoutFlavor::PlantUmlGenerated => {
                    let columns = (label.width / 7.).ceil().clamp(1., 80.);
                    let font_size = label.height.clamp(12., 80.);
                    columns * font_size * 0.6
                }
                LayoutFlavor::RawDot => label.width,
            };
            self_space[node] += (18. + label_width).round();
        }
    }
    let lengths: Vec<_> = rows
        .iter()
        .map(|row| {
            row.iter()
                .map(|&i| primary(units[i].size))
                .fold(0., f64::max)
        })
        .collect();
    let row_step = |current: usize, next: usize| {
        let width = secondary(units[current].size);
        let next_width = secondary(units[next].size);
        if matches!(flavor, LayoutFlavor::RawDot)
            && self_space[current] == 0.
            && self_space[next] == 0.
        {
            let separation = (width / 2. + next_width / 2. + gap).round();
            return width / 2. + separation - next_width / 2.;
        }
        width + self_space[current] + gap
    };
    let breadths: Vec<_> = rows
        .iter()
        .map(|row| {
            let Some(&last) = row.last() else {
                return 0.;
            };
            row.windows(2)
                .map(|pair| row_step(pair[0], pair[1]))
                .sum::<f64>()
                + secondary(units[last].size)
                + self_space[last]
        })
        .collect();
    let extent = lengths.iter().sum::<f64>() + rank_gaps.iter().sum::<f64>();
    let breadth = breadths.iter().copied().fold(0., f64::max);
    let mut coordinates = vec![(0., 0.); units.len()];
    let mut axis = 0.;
    for (rank, row) in rows.iter().enumerate() {
        budget.charge(row.len())?;
        let mut cross = (breadth - breadths[rank]) / 2.;
        for (position, &index) in row.iter().enumerate() {
            let size = units[index].size;
            let start = axis + (lengths[rank] - primary(size)) / 2.;
            let start = if matches!(direction, RankDir::BT | RankDir::RL) {
                extent - start - primary(size)
            } else {
                start
            };
            coordinates[index] = if horizontal {
                (start, cross)
            } else {
                (cross, start)
            };
            if let Some(&next) = row.get(position + 1) {
                cross += row_step(index, next);
            }
        }
        axis += lengths[rank] + rank_gaps.get(rank).copied().unwrap_or(0.);
    }
    align_connected_rows(&units, &rows, links, &mut coordinates, horizontal, budget)?;
    if matches!(flavor, LayoutFlavor::RawDot) && matches!(direction, RankDir::TB) {
        align_binary_forks(&units, &rows, links, &mut coordinates, budget)?;
    }
    if graph.is_directed() {
        place_feedback_channels(
            &units,
            &rows,
            links,
            &mut coordinates,
            horizontal,
            gap,
            budget,
        )?;
    } else {
        place_undirected_feedback_channels(
            &units,
            &rows,
            links,
            &mut coordinates,
            horizontal,
            budget,
        )?;
    }
    let (minimum, maximum) = coordinates.iter().enumerate().fold(
        (f64::INFINITY, f64::NEG_INFINITY),
        |(minimum, maximum), (index, &(x, y))| {
            let cross = if horizontal { y } else { x };
            (
                minimum.min(cross),
                maximum.max(cross + secondary(units[index].size) + self_space[index]),
            )
        },
    );
    let corridor_shift =
        parallel_label_corridor_shift(&units, links, &coordinates, flavor, direction, gap);
    let mut breadth = maximum - minimum + corridor_shift;
    for coordinate in &mut coordinates {
        if horizontal {
            coordinate.1 -= minimum;
        } else {
            coordinate.0 += corridor_shift - minimum;
        }
    }
    let virtual_shift =
        mixed_feedback_virtual_shift(graph, &units, &rows, links, &coordinates, flavor, budget)?;
    if virtual_shift > 0. {
        budget.charge(coordinates.len())?;
        for coordinate in &mut coordinates {
            coordinate.0 += virtual_shift;
        }
        breadth += virtual_shift;
    }
    let mut nodes = HashMap::new();
    for (unit, (x, y)) in units.into_iter().zip(coordinates) {
        for (id, mut child) in unit.nodes {
            child.x += x;
            child.y += y;
            nodes.insert(id, child);
        }
    }
    let size = if horizontal {
        Size {
            width: extent,
            height: breadth,
        }
    } else {
        Size {
            width: breadth,
            height: extent,
        }
    };
    Ok(Group { size, nodes })
}

fn mixed_feedback_virtual_shift(
    graph: &LayoutGraph,
    units: &[Unit],
    rows: &[Vec<usize>],
    links: &[Link],
    coordinates: &[(f64, f64)],
    flavor: LayoutFlavor,
    budget: &mut dyn LayoutWork,
) -> Result<f64, RenderError> {
    if !matches!(flavor, LayoutFlavor::RawDot)
        || !graph.is_directed()
        || !matches!(graph.graph().rankdir, RankDir::TB)
        || units.len() > 3
        || rows.len() != 2
        || links.len() != 3
        || graph.edges().count() != 4
    {
        return Ok(0.);
    }
    budget.charge(graph.edges().count() + units.len())?;
    let [left, center, reverse] = links else {
        return Ok(0.);
    };
    if left.from != center.from
        || left.to != center.to
        || reverse.from != center.to
        || reverse.to != center.from
        || !rows[0].contains(&center.from)
        || !rows[1].contains(&center.to)
        || !links.iter().all(|link| {
            link.label_present
                && link.minlen == 1
                && link.constraint
                && !link.port
                && link.weight == 1.
        })
        || graph
            .edges()
            .filter(|edge| edge.v == edge.w && edge.v == units[center.from].id)
            .count()
            != 1
        || (units[center.from].size.width - 54.).abs() > 1e-9
        || (units[center.to].size.width - 54.).abs() > 1e-9
    {
        return Ok(0.);
    }
    let label_rank = rows.len() - 1;
    let virtuals = links
        .iter()
        .map(|link| VirtualLabelUnit {
            rank: label_rank,
            left_width: graph.graph().nodesep,
            right_width: link.label_width,
        })
        .collect::<Vec<_>>();
    if virtuals.iter().any(|unit| unit.rank != 1) {
        return Ok(0.);
    }
    let center_x = virtuals[1].separation_after(virtuals[0]);
    let current_x = coordinates[center.from].0 + units[center.from].size.width / 2.;
    Ok((center_x - current_x).max(0.))
}

fn cluster_entry_space(
    graph: &LayoutGraph,
    units: &[Unit],
    rows: &[Vec<usize>],
    rank: usize,
    budget: &mut dyn LayoutWork,
) -> Result<f64, RenderError> {
    let mut space: f64 = 0.;
    for cluster in graph.nodes().filter(|id| is_dot_cluster(id)) {
        budget.charge(units.len())?;
        let entering = rows[rank]
            .iter()
            .any(|&index| cluster_contains(graph, cluster, &units[index].id));
        let occupied_before = rows
            .iter()
            .take(rank)
            .flatten()
            .any(|&index| cluster_contains(graph, cluster, &units[index].id));
        let disjoint_previous_cluster = rows[rank - 1].iter().any(|&index| {
            graph.parent(&units[index].id).is_some_and(|previous| {
                is_dot_cluster(previous)
                    && previous != cluster
                    && !cluster_contains(graph, previous, cluster)
                    && !cluster_contains(graph, cluster, previous)
            })
        });
        if entering && !occupied_before && !disjoint_previous_cluster {
            let owner = graph
                .node(cluster)
                .ok_or_else(|| failure("DOT cluster owner disappeared"))?;
            if owner.height > 0. {
                // Graphviz's clust_ht raises this rank by the label and two
                // CL_OFFSET borders; set_ycoords compares that with half of
                // the ordinary rank separation on the virtual intervening rank.
                let raised = owner.height + 3. * 8. - graph.graph().ranksep / 2.;
                space = space.max(raised.max(0.));
            }
        }
    }
    Ok(space)
}

fn align_binary_forks(
    units: &[Unit],
    rows: &[Vec<usize>],
    links: &[Link],
    coordinates: &mut [(f64, f64)],
    budget: &mut dyn LayoutWork,
) -> Result<(), RenderError> {
    for ranks in rows.windows(2) {
        let [parent_rank, child_rank] = ranks else {
            continue;
        };
        if parent_rank.len() != 1 || child_rank.len() != 2 {
            continue;
        }
        budget.charge(links.len())?;
        let parent = parent_rank[0];
        let children = [child_rank[0], child_rank[1]];
        if children.iter().any(|&child| {
            links
                .iter()
                .filter(|link| link.from == parent && link.to == child && !link.port)
                .count()
                != 1
        }) {
            continue;
        }
        let centers = children.map(|child| coordinates[child].0 + units[child].size.width / 2.);
        let left = centers[0].min(centers[1]);
        let right = centers[0].max(centers[1]);
        // Graphviz rounds horizontal auxiliary edge minimum lengths to integer
        // units (position.c make_aux_edge). An odd child-center span therefore
        // places the sole parent at the lower integer midpoint.
        let target = left + ((right - left) / 2.).floor();
        let current = coordinates[parent].0 + units[parent].size.width / 2.;
        let shift = target - current;
        if shift.abs() <= 0.51 {
            coordinates[parent].0 += shift;
        }
    }
    Ok(())
}

fn parallel_label_corridor_shift(
    units: &[Unit],
    links: &[Link],
    coordinates: &[(f64, f64)],
    flavor: LayoutFlavor,
    direction: RankDir,
    nodesep: f64,
) -> f64 {
    if !matches!(flavor, LayoutFlavor::RawDot)
        || !matches!(direction, RankDir::TB)
        || links.len() != 3
    {
        return 0.;
    }
    let first = links[0];
    if links.iter().any(|link| {
        link.from != first.from
            || link.to != first.to
            || !link.label_present
            || (link.label_width - first.label_width).abs() > 1e-9
    }) {
        return 0.;
    }
    let source = coordinates[first.from].0 + units[first.from].size.width / 2.;
    let target = coordinates[first.to].0 + units[first.to].size.width / 2.;
    if (source - target).abs() > 1e-9 {
        return 0.;
    }
    // class2.c creates three label virtual nodes; position.c separates them
    // by integer auxiliary-edge minlen (nodesep + label width + 5). The first
    // corridor's right wall is its virtual x + 10 - label width.
    let spacing = (nodesep + first.label_width + 5.).floor();
    let left_wall = source - spacing + 10. - first.label_width;
    let minimum = coordinates
        .iter()
        .map(|point| point.0)
        .fold(f64::INFINITY, f64::min);
    (minimum - left_wall).max(0.)
}

fn align_connected_rows(
    units: &[Unit],
    rows: &[Vec<usize>],
    links: &[Link],
    coordinates: &mut [(f64, f64)],
    horizontal: bool,
    budget: &mut dyn LayoutWork,
) -> Result<(), RenderError> {
    let mut rank_of = vec![0; units.len()];
    for (rank, row) in rows.iter().enumerate() {
        for &node in row {
            rank_of[node] = rank;
        }
    }
    for (rank, row) in rows.iter().enumerate().skip(1) {
        budget.charge(links.len() + row.len())?;
        let cross = |node: usize, anchor: Point| {
            if horizontal {
                coordinates[node].1 + anchor.y
            } else {
                coordinates[node].0 + anchor.x
            }
        };
        let mut displacement = 0.;
        let mut total_weight = 0.;
        let mut contributing = 0;
        let mut port = false;
        for link in links {
            if rank_of[link.from] == rank && rank_of[link.to] < rank {
                displacement +=
                    (cross(link.to, link.target) - cross(link.from, link.source)) * link.weight;
                total_weight += link.weight;
                contributing += 1;
                port = link.port;
            } else if rank_of[link.to] == rank && rank_of[link.from] < rank {
                displacement +=
                    (cross(link.from, link.source) - cross(link.to, link.target)) * link.weight;
                total_weight += link.weight;
                contributing += 1;
                port = link.port;
            }
        }
        if total_weight > 0. {
            let mut shift = displacement / total_weight;
            if horizontal && contributing == 1 && port {
                // DOT's auxiliary position solver uses integer port offsets.
                shift = shift.floor();
            }
            for &node in row {
                if horizontal {
                    coordinates[node].1 += shift;
                } else {
                    coordinates[node].0 += shift;
                }
            }
        }
    }
    Ok(())
}

fn place_feedback_channels(
    units: &[Unit],
    rows: &[Vec<usize>],
    links: &[Link],
    coordinates: &mut [(f64, f64)],
    horizontal: bool,
    gap: f64,
    budget: &mut dyn LayoutWork,
) -> Result<(), RenderError> {
    let mut rank_of = vec![0; units.len()];
    for (rank, row) in rows.iter().enumerate() {
        for &node in row {
            rank_of[node] = rank;
        }
    }
    for link in links {
        budget.charge(units.len())?;
        let top = rank_of[link.to];
        let bottom = rank_of[link.from];
        if bottom <= top + 1 {
            continue;
        }
        let mut right = f64::NEG_INFINITY;
        for row in rows.iter().take(bottom).skip(top + 1) {
            for &node in row {
                let size = units[node].size;
                let half = if horizontal {
                    size.height / 2.
                } else {
                    size.width / 2.
                };
                let center = if horizontal {
                    coordinates[node].1
                } else {
                    coordinates[node].0
                };
                right = right.max(center + 2. * half + gap / 2.);
            }
        }
        if !right.is_finite() {
            continue;
        }
        for (node, anchor) in [(link.to, link.target), (link.from, link.source)] {
            let current = if horizontal {
                coordinates[node].1 + anchor.y
            } else {
                coordinates[node].0 + anchor.x
            };
            let shift = (right - current).max(0.);
            for &member in &rows[rank_of[node]] {
                if horizontal {
                    coordinates[member].1 += shift;
                } else {
                    coordinates[member].0 += shift;
                }
            }
        }
    }
    Ok(())
}

fn place_undirected_feedback_channels(
    units: &[Unit],
    rows: &[Vec<usize>],
    links: &[Link],
    coordinates: &mut [(f64, f64)],
    horizontal: bool,
    budget: &mut dyn LayoutWork,
) -> Result<(), RenderError> {
    let mut rank_of = vec![0; units.len()];
    for (rank, row) in rows.iter().enumerate() {
        for &node in row {
            rank_of[node] = rank;
        }
    }
    for link in links {
        let top = rank_of[link.from].min(rank_of[link.to]);
        let bottom = rank_of[link.from].max(rank_of[link.to]);
        if bottom <= top + 1 {
            continue;
        }
        budget.charge(units.len())?;
        let corridor = rows
            .iter()
            .take(bottom)
            .skip(top + 1)
            .flat_map(|row| row.iter().copied())
            .map(|index| {
                let cross = if horizontal {
                    coordinates[index].1
                } else {
                    coordinates[index].0
                };
                cross
                    + if horizontal {
                        units[index].size.height
                    } else {
                        units[index].size.width
                    }
            })
            .fold(f64::NEG_INFINITY, f64::max);
        if !corridor.is_finite() {
            continue;
        }
        for (node, anchor) in [(link.from, link.source), (link.to, link.target)] {
            let current = if horizontal {
                coordinates[node].1 + anchor.y
            } else {
                coordinates[node].0 + anchor.x
            };
            let shift = (corridor - current).max(0.);
            for &member in &rows[rank_of[node]] {
                if horizontal {
                    coordinates[member].1 += shift;
                } else {
                    coordinates[member].0 += shift;
                }
            }
        }
    }
    Ok(())
}

fn route(
    graph: &mut LayoutGraph,
    budget: &mut dyn LayoutWork,
    flavor: LayoutFlavor,
) -> Result<(), RenderError> {
    let keys = graph.edges().cloned().collect::<Vec<_>>();
    let mixed = mixed_feedback_keys(graph, &keys);
    for (index, key) in keys.iter().enumerate() {
        budget.charge(1)?;
        let source = graph
            .node(&key.v)
            .ok_or_else(|| failure("edge source disappeared"))?;
        let target = graph
            .node(&key.w)
            .ok_or_else(|| failure("edge target disappeared"))?;
        let start = Point {
            x: source.x.ok_or_else(|| failure("source has no position"))?,
            y: source.y.ok_or_else(|| failure("source has no position"))?,
        };
        let end = Point {
            x: target.x.ok_or_else(|| failure("target has no position"))?,
            y: target.y.ok_or_else(|| failure("target has no position"))?,
        };
        let (points, label_at) = if key.v == key.w {
            let exact_self = matches!(flavor, LayoutFlavor::RawDot)
                && matches!(graph.graph().rankdir, RankDir::TB)
                && keys
                    .iter()
                    .filter(|other| other.v == key.v && other.w == key.w)
                    .count()
                    == 1;
            if exact_self {
                budget.charge(graph.nodes().count() + keys.len() + 8)?;
                let label = graph
                    .edge_by_key(key)
                    .ok_or_else(|| failure("self edge disappeared"))?;
                let arrow = label
                    .extras
                    .get("dot_arrow_present")
                    .and_then(|value| value.as_bool())
                    .unwrap_or_else(|| graph.is_directed());
                let nearest_rank = graph
                    .nodes()
                    .filter_map(|id| graph.node(id).and_then(|node| node.y))
                    .map(|y| (y - start.y).abs())
                    .filter(|distance| *distance > 1e-9)
                    .reduce(f64::min);
                // Graphviz doubles every edge rank when any edge has a label;
                // the closest rank to this node is then a virtual label rank.
                let virtual_rank_factor = if keys.iter().any(|edge| {
                    graph.edge_by_key(edge).is_some_and(|label| {
                        label
                            .extras
                            .get("dot_label_present")
                            .and_then(|value| value.as_bool())
                            .unwrap_or(false)
                    })
                }) {
                    2.
                } else {
                    1.
                };
                let rank_distance = nearest_rank
                    .map(|distance| distance / virtual_rank_factor)
                    .unwrap_or(source.height);
                let points = route_self_right(SelfRoute {
                    center: start,
                    size: Size {
                        width: source.width,
                        height: source.height,
                    },
                    nodesep: graph.graph().nodesep,
                    rank_distance,
                    shape: edge_clip_shape(graph, key, "dot_tail_shape"),
                    arrow: if arrow {
                        ArrowMode::Head
                    } else {
                        ArrowMode::None
                    },
                });
                let label_at = Point {
                    x: start.x + source.width / 2. + graph.graph().nodesep + label.width / 2.,
                    y: start.y,
                };
                (points, label_at)
            } else {
                let outside = source.width / 2. + 40. + (index % 3) as f64 * 12.;
                let upper = Point {
                    x: start.x + source.width / 2.,
                    y: start.y - source.height / 4.,
                };
                let lower = Point {
                    x: start.x + source.width / 2.,
                    y: start.y + source.height / 4.,
                };
                let right = start.x + outside;
                (
                    vec![
                        upper,
                        Point {
                            x: right,
                            y: upper.y - 20.,
                        },
                        Point {
                            x: right,
                            y: lower.y + 20.,
                        },
                        lower,
                    ],
                    Point {
                        x: right,
                        y: start.y,
                    },
                )
            }
        } else {
            let rank_separation = match graph.graph().rankdir {
                RankDir::TB | RankDir::BT => (start.y - end.y).abs(),
                RankDir::LR | RankDir::RL => (start.x - end.x).abs(),
            };
            let mixed_role = mixed.as_ref().and_then(|edges| {
                if edges.left == *key {
                    Some(MixedCorridorRole::Left)
                } else if edges.center == *key {
                    Some(MixedCorridorRole::Center)
                } else if edges.reverse == *key {
                    Some(MixedCorridorRole::Reverse)
                } else {
                    None
                }
            });
            let clear = if mixed_role == Some(MixedCorridorRole::Center) {
                true
            } else if matches!(flavor, LayoutFlavor::RawDot) && rank_separation > 1e-9 {
                budget.charge(graph.nodes().count() + graph.edges().count())?;
                has_clear_shortest_path(graph, (&key.v, &key.w), (start, end))
                    || is_clear_parallel_center(graph, key, (start, end))
            } else {
                false
            };
            let parallel_lane = if matches!(flavor, LayoutFlavor::RawDot) {
                parallel_side_lane(graph, key, (start, end))
            } else {
                None
            };
            let undirected_lane = if matches!(flavor, LayoutFlavor::RawDot) {
                undirected_feedback_lane(graph, key, (start, end))
            } else {
                None
            };
            let cluster_crossing = if matches!(flavor, LayoutFlavor::RawDot) {
                cluster_crossing_waypoint(graph, key, (start, end))
            } else {
                None
            };
            let port_segment = graph.edge_by_key(key).and_then(|label| {
                let source_offset = record_port_offset(label, "tail")?;
                let target_offset = record_port_offset(label, "head")?;
                Some((
                    Point {
                        x: start.x + source_offset.x,
                        y: start.y + source_offset.y,
                    },
                    Point {
                        x: end.x + target_offset.x,
                        y: end.y + target_offset.y,
                    },
                ))
            });
            let has_arrow = graph
                .edge_by_key(key)
                .and_then(|edge| edge.extras.get("dot_arrow_present"))
                .and_then(|value| value.as_bool())
                .unwrap_or_else(|| graph.is_directed());
            let arrow = if has_arrow {
                ArrowMode::Head
            } else {
                ArrowMode::None
            };
            let mixed_curve = if matches!(
                mixed_role,
                Some(MixedCorridorRole::Left | MixedCorridorRole::Reverse)
            ) {
                let label = graph
                    .edge_by_key(key)
                    .ok_or_else(|| failure("mixed feedback edge disappeared"))?;
                Some(route_mixed_corridor(
                    MixedCorridorRoute {
                        upper: if matches!(mixed_role, Some(MixedCorridorRole::Reverse)) {
                            end
                        } else {
                            start
                        },
                        lower: if matches!(mixed_role, Some(MixedCorridorRole::Reverse)) {
                            start
                        } else {
                            end
                        },
                        upper_size: Size {
                            width: if matches!(mixed_role, Some(MixedCorridorRole::Reverse)) {
                                target.width
                            } else {
                                source.width
                            },
                            height: if matches!(mixed_role, Some(MixedCorridorRole::Reverse)) {
                                target.height
                            } else {
                                source.height
                            },
                        },
                        lower_size: Size {
                            width: if matches!(mixed_role, Some(MixedCorridorRole::Reverse)) {
                                source.width
                            } else {
                                target.width
                            },
                            height: if matches!(mixed_role, Some(MixedCorridorRole::Reverse)) {
                                source.height
                            } else {
                                target.height
                            },
                        },
                        label_width: label.width,
                        label_height: label.height,
                        center_label_width: mixed
                            .as_ref()
                            .and_then(|edges| graph.edge_by_key(&edges.center))
                            .map(|edge| edge.width)
                            .unwrap_or(0.),
                        nodesep: graph.graph().nodesep,
                        role: mixed_role.unwrap(),
                    },
                    budget,
                )?)
            } else {
                None
            };
            let points = if let Some((points, _)) = &mixed_curve {
                points.clone()
            } else if let Some((tail, head)) = port_segment {
                route_record_ports(tail, head, arrow)
            } else if let Some(waypoint) = cluster_crossing {
                route_cluster_crossing(
                    (start, end),
                    waypoint,
                    (
                        Size {
                            width: source.width,
                            height: source.height,
                        },
                        Size {
                            width: target.width,
                            height: target.height,
                        },
                    ),
                    (
                        edge_clip_shape(graph, key, "dot_tail_shape"),
                        edge_clip_shape(graph, key, "dot_head_shape"),
                    ),
                    arrow,
                )
            } else if let Some((lane, obstacle_height)) = undirected_lane {
                let label = graph
                    .edge_by_key(key)
                    .ok_or_else(|| failure("feedback edge disappeared"))?;
                let reverse = label
                    .extras
                    .get("dot_original_tail")
                    .and_then(|value| value.as_str())
                    == Some(key.w.as_str());
                let (segment, sizes, shapes) = if reverse {
                    (
                        (end, start),
                        (
                            Size {
                                width: target.width,
                                height: target.height,
                            },
                            Size {
                                width: source.width,
                                height: source.height,
                            },
                        ),
                        (
                            edge_clip_shape(graph, key, "dot_head_shape"),
                            edge_clip_shape(graph, key, "dot_tail_shape"),
                        ),
                    )
                } else {
                    (
                        (start, end),
                        (
                            Size {
                                width: source.width,
                                height: source.height,
                            },
                            Size {
                                width: target.width,
                                height: target.height,
                            },
                        ),
                        (
                            edge_clip_shape(graph, key, "dot_tail_shape"),
                            edge_clip_shape(graph, key, "dot_head_shape"),
                        ),
                    )
                };
                route_parallel_polyline(
                    segment,
                    sizes,
                    lane,
                    obstacle_height,
                    ArrowMode::None,
                    shapes,
                )
            } else if let Some(lane) = parallel_lane {
                route_parallel_polyline(
                    (start, end),
                    (
                        Size {
                            width: source.width,
                            height: source.height,
                        },
                        Size {
                            width: target.width,
                            height: target.height,
                        },
                    ),
                    lane,
                    graph
                        .edge_by_key(key)
                        .ok_or_else(|| failure("parallel edge disappeared"))?
                        .height,
                    arrow,
                    (
                        edge_clip_shape(graph, key, "dot_tail_shape"),
                        edge_clip_shape(graph, key, "dot_head_shape"),
                    ),
                )
            } else if clear {
                let tail_arrow = graph
                    .edge_by_key(key)
                    .and_then(|edge| edge.extras.get("dot_arrow_tail_present"))
                    .and_then(|value| value.as_bool())
                    .unwrap_or(false);
                let arrow_size = graph
                    .edge_by_key(key)
                    .and_then(|edge| edge.extras.get("dot_arrow_size"))
                    .and_then(|value| value.as_f64())
                    .unwrap_or(1.);
                let arrow_penwidth = graph
                    .edge_by_key(key)
                    .and_then(|edge| edge.extras.get("dot_arrow_penwidth"))
                    .and_then(|value| value.as_f64())
                    .unwrap_or(1.);
                let arrow_length = |name: &str| {
                    let shape = graph
                        .edge_by_key(key)
                        .and_then(|edge| edge.extras.get(name))
                        .and_then(|value| value.as_str())
                        .unwrap_or("normal");
                    dot_arrow_stem_length(shape, arrow_size, arrow_penwidth)
                };
                route_straight_cubic(StraightRoute {
                    start,
                    end,
                    source: Size {
                        width: source.width,
                        height: source.height,
                    },
                    target: Size {
                        width: target.width,
                        height: target.height,
                    },
                    source_shape: edge_clip_shape(graph, key, "dot_tail_shape"),
                    target_shape: edge_clip_shape(graph, key, "dot_head_shape"),
                    arrow,
                    tail_arrow,
                    head_arrow_length: arrow_length("dot_arrow_head_shape"),
                    tail_arrow_length: arrow_length("dot_arrow_tail_shape"),
                    rankdir: graph.graph().rankdir,
                    flow: match graph.graph().rankdir {
                        RankDir::TB if end.y < start.y => RouteFlow::Backward,
                        RankDir::BT if end.y > start.y => RouteFlow::Backward,
                        RankDir::LR if end.x < start.x => RouteFlow::Backward,
                        RankDir::RL if end.x > start.x => RouteFlow::Backward,
                        _ => RouteFlow::Forward,
                    },
                })
            } else {
                vec![
                    clip(start, source.width, source.height, end),
                    clip(end, target.width, target.height, start),
                ]
            };
            let label_at = match flavor {
                LayoutFlavor::RawDot => {
                    let label = graph
                        .edge_by_key(key)
                        .ok_or_else(|| failure("edge disappeared before label placement"))?;
                    let horizontal = matches!(graph.graph().rankdir, RankDir::LR | RankDir::RL);
                    let label_present = label
                        .extras
                        .get("dot_label_present")
                        .and_then(|value| value.as_bool())
                        .unwrap_or(false);
                    let enters_labeled_cluster =
                        graph.parent(&key.w).is_some_and(|target_parent| {
                            is_dot_cluster(target_parent)
                                && graph.parent(&key.v) != Some(target_parent)
                                && graph.parent(&key.v).is_none_or(|source_parent| {
                                    cluster_contains(graph, source_parent, target_parent)
                                })
                        });
                    let virtual_x = if let Some((_, recovered_x)) = &mixed_curve {
                        *recovered_x
                    } else if mixed.as_ref().is_some_and(|edges| edges.left == *key) {
                        // maximal_bbox reserves the label's right extent to
                        // the left of the neighboring middle-rank virtual node.
                        // recover_slack retains this right bound for the first
                        // labeled edge in the bounded mixed feedback corridor.
                        start.x - graph.graph().nodesep - 4. - label.width
                    } else if let Some(lane) = parallel_lane {
                        lane
                    } else if clear && label_present && matches!(graph.graph().rankdir, RankDir::TB)
                    {
                        budget.charge(51)?;
                        virtual_label_x(start, end, label.height)
                    } else {
                        (start.x + end.x) / 2.
                    };
                    if let Some((tail, head)) = port_segment {
                        Point {
                            x: (tail.x + head.x) / 2.,
                            y: virtual_port_label_y(tail, head, label.width, label.height),
                        }
                    } else {
                        // Graphviz keeps the edge-label virtual rank at its
                        // ordinary offset when the target cluster expands.
                        let ordinary_label_y = start.y
                            + (source.height / 2.
                                + target.height / 2.
                                + graph.graph().ranksep
                                + label.height)
                                / 2.;
                        let label_y = if label_present
                            && enters_labeled_cluster
                            && matches!(graph.graph().rankdir, RankDir::TB)
                            && end.y > ordinary_label_y
                        {
                            ordinary_label_y
                        } else {
                            (start.y + end.y) / 2.
                        };
                        Point {
                            x: virtual_x + if horizontal { 0. } else { label.width / 2. },
                            y: label_y - if horizontal { label.height / 2. } else { 0. },
                        }
                    }
                }
                LayoutFlavor::PlantUmlGenerated => Point {
                    x: (points[0].x + points[points.len() - 1].x) / 2.,
                    y: (points[0].y + points[points.len() - 1].y) / 2.,
                },
            };
            (points, label_at)
        };
        let label = graph
            .edge_mut_by_key(key)
            .ok_or_else(|| failure("edge disappeared"))?;
        label.x = Some(label_at.x);
        label.y = Some(label_at.y);
        label.points = points;
    }
    if let Some(edges) = &mixed {
        normalize_mixed_spline_extents(graph, &edges.left, budget)?;
    }
    Ok(())
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum MixedCorridorRole {
    Left,
    Center,
    Reverse,
}

struct MixedCorridorRoute {
    upper: Point,
    lower: Point,
    upper_size: Size,
    lower_size: Size,
    label_width: f64,
    label_height: f64,
    center_label_width: f64,
    nodesep: f64,
    role: MixedCorridorRole,
}

struct MixedFeedbackEdges {
    left: dugong::graphlib::EdgeKey,
    center: dugong::graphlib::EdgeKey,
    reverse: dugong::graphlib::EdgeKey,
}

fn mixed_feedback_keys(
    graph: &LayoutGraph,
    keys: &[dugong::graphlib::EdgeKey],
) -> Option<MixedFeedbackEdges> {
    let [left, center, reverse, loop_edge] = keys else {
        return None;
    };
    if !graph.is_directed()
        || !matches!(graph.graph().rankdir, RankDir::TB)
        || left.v != center.v
        || left.w != center.w
        || reverse.v != center.w
        || reverse.w != center.v
        || loop_edge.v != left.v
        || loop_edge.w != left.v
        || graph.nodes().count() > 3
        || graph.nodes().any(|id| graph.parent(id).is_some())
    {
        return None;
    }
    let labels = [left, center, reverse]
        .map(|key| graph.edge_by_key(key))
        .into_iter()
        .collect::<Option<Vec<_>>>()?;
    if labels.iter().any(|label| {
        label.minlen != 1
            || label.weight != 1.
            || (label.height - 16.8).abs() > 1e-9
            || label.width <= 0.
            || !label
                .extras
                .get("dot_label_present")
                .and_then(|value| value.as_bool())
                .unwrap_or(false)
            || label
                .extras
                .get("dot_tail_shape")
                .and_then(|value| value.as_str())
                != Some("box")
            || label
                .extras
                .get("dot_head_shape")
                .and_then(|value| value.as_str())
                != Some("box")
            || label
                .extras
                .get("dot_arrow_head_shape")
                .and_then(|value| value.as_str())
                != Some("normal")
            || label
                .extras
                .get("dot_arrow_size")
                .and_then(|value| value.as_f64())
                != Some(1.)
            || label
                .extras
                .get("dot_arrow_penwidth")
                .and_then(|value| value.as_f64())
                != Some(1.)
            || label
                .extras
                .get("dot_arrow_present")
                .and_then(|value| value.as_bool())
                != Some(true)
            || label
                .extras
                .get("dot_arrow_tail_present")
                .and_then(|value| value.as_bool())
                != Some(false)
            || label.extras.contains_key("dot_tail_port")
            || label.extras.contains_key("dot_head_port")
            || label
                .extras
                .get("dot_tail_penwidth")
                .and_then(|value| value.as_f64())
                != Some(1.)
            || label
                .extras
                .get("dot_head_penwidth")
                .and_then(|value| value.as_f64())
                != Some(1.)
    }) {
        return None;
    }
    let nodes = [left.v.as_str(), left.w.as_str()]
        .map(|id| graph.node(id))
        .into_iter()
        .collect::<Option<Vec<_>>>()?;
    if nodes
        .iter()
        .any(|node| (node.width - 54.).abs() > 1e-9 || (node.height - 36.).abs() > 1e-9)
    {
        return None;
    }
    Some(MixedFeedbackEdges {
        left: left.clone(),
        center: center.clone(),
        reverse: reverse.clone(),
    })
}

fn route_mixed_corridor(
    route: MixedCorridorRoute,
    budget: &mut dyn LayoutWork,
) -> Result<(Vec<Point>, f64), RenderError> {
    let middle = (route.upper.y + route.lower.y) / 2.;
    let corner_x = match route.role {
        MixedCorridorRole::Left => route.upper.x - route.nodesep - 4. - route.label_width,
        MixedCorridorRole::Reverse => route.upper.x + (route.center_label_width + 1.).round(),
        MixedCorridorRole::Center => return Err(failure("center mixed corridor has no side path")),
    };
    let upper = Point {
        x: route.upper.x,
        y: route.upper.y + 1.,
    };
    let lower = Point {
        x: route.lower.x,
        y: route.lower.y - 1.,
    };
    let first = Point {
        x: corner_x,
        y: middle - route.label_height / 2.,
    };
    let second = Point {
        x: corner_x,
        y: middle + route.label_height / 2.,
    };
    let first_tangent = bisect_path_tangent(upper, first, second);
    let second_tangent = bisect_path_tangent(first, second, lower);
    let mut splines = match route.role {
        MixedCorridorRole::Left => {
            let first_span = distance(upper, first) * 4. / 9.;
            let second_span = distance(first, lower) * 4. / 9.;
            vec![
                [
                    upper,
                    upper,
                    move_along(first, first_tangent, -first_span),
                    first,
                ],
                [
                    first,
                    move_along(first, first_tangent, second_span),
                    lower,
                    lower,
                ],
            ]
        }
        MixedCorridorRole::Reverse => {
            let first_span = distance(upper, first) * 4. / 9.;
            let middle_span = distance(first, second) * 4. / 9.;
            let last_span = distance(second, lower) * 4. / 9.;
            vec![
                [
                    upper,
                    upper,
                    move_along(first, first_tangent, -first_span),
                    first,
                ],
                [
                    first,
                    move_along(first, first_tangent, middle_span),
                    move_along(second, second_tangent, -middle_span),
                    second,
                ],
                [
                    second,
                    move_along(second, second_tangent, last_span),
                    lower,
                    lower,
                ],
            ]
        }
        MixedCorridorRole::Center => unreachable!(),
    };
    budget.charge(splines.len() * 55)?;
    splines[0] = clip_cubic(splines[0], ClipEnd::Start, |point| {
        inside_shape(point, route.upper, route.upper_size, ClipShape::Box(1.))
    });
    let last = splines.len() - 1;
    splines[last] = clip_cubic(splines[last], ClipEnd::End, |point| {
        inside_shape(point, route.lower, route.lower_size, ClipShape::Box(1.))
    });
    let tip = match route.role {
        MixedCorridorRole::Left => {
            let tip = splines[last][3];
            splines[last] = clip_cubic(splines[last], ClipEnd::End, |point| {
                distance(point, tip) <= normal_arrow_stem_length()
            });
            tip
        }
        MixedCorridorRole::Reverse => {
            let tip = splines[0][0];
            splines[0] = clip_cubic(splines[0], ClipEnd::Start, |point| {
                distance(point, tip) <= normal_arrow_stem_length()
            });
            tip
        }
        MixedCorridorRole::Center => unreachable!(),
    };
    let virtual_x = match route.role {
        MixedCorridorRole::Left => corner_x,
        MixedCorridorRole::Reverse => recover_corridor_right(&splines, middle, route.label_height),
        MixedCorridorRole::Center => unreachable!(),
    };
    let mut points = Vec::with_capacity(splines.len() * 3 + 2);
    points.extend(splines[0]);
    for spline in splines.iter().skip(1) {
        points.extend(spline.iter().skip(1).copied());
    }
    if route.role == MixedCorridorRole::Reverse {
        points.reverse();
    }
    points.push(tip);
    Ok((points, virtual_x))
}

fn move_along(point: Point, direction: Point, length: f64) -> Point {
    Point {
        x: point.x + direction.x * length,
        y: point.y + direction.y * length,
    }
}

fn bisect_path_tangent(before: Point, at: Point, after: Point) -> Point {
    let incoming = Point {
        x: at.x - before.x,
        y: at.y - before.y,
    };
    let outgoing = Point {
        x: after.x - at.x,
        y: after.y - at.y,
    };
    let first = move_along(Point { x: 0., y: 0. }, incoming, 1. / distance(before, at));
    let second = move_along(Point { x: 0., y: 0. }, outgoing, 1. / distance(at, after));
    let sum = Point {
        x: first.x + second.x,
        y: first.y + second.y,
    };
    move_along(Point { x: 0., y: 0. }, sum, 1. / sum.x.hypot(sum.y))
}

fn recover_corridor_right(splines: &[[Point; 4]], middle: f64, height: f64) -> f64 {
    let mut right = f64::NEG_INFINITY;
    for spline in splines {
        for sample in 0..=50 {
            let point = split_cubic(*spline, sample as f64 / 50.).0;
            if (point.y - middle).abs() <= height / 2. + 0.0001 {
                right = right.max(point.x);
            }
        }
    }
    right
}

fn normalize_mixed_spline_extents(
    graph: &mut LayoutGraph,
    key: &dugong::graphlib::EdgeKey,
    budget: &mut dyn LayoutWork,
) -> Result<(), RenderError> {
    let points = &graph
        .edge_by_key(key)
        .ok_or_else(|| failure("mixed left edge disappeared"))?
        .points;
    let mut left: f64 = 0.;
    for control in points[..points.len() - 1].windows(4).step_by(3) {
        let cubic = [control[0], control[1], control[2], control[3]];
        left = left.min(refined_cubic_min_x(cubic, budget)?);
    }
    if left >= 0. {
        return Ok(());
    }
    let shift = -left;
    let node_ids = graph.nodes().map(str::to_owned).collect::<Vec<_>>();
    let edge_keys = graph.edges().cloned().collect::<Vec<_>>();
    budget.charge(node_ids.len() + edge_keys.len())?;
    for id in node_ids {
        if let Some(node) = graph.node_mut(&id) {
            node.x = node.x.map(|x| x + shift);
        }
    }
    for key in edge_keys {
        if let Some(edge) = graph.edge_mut_by_key(&key) {
            edge.x = edge.x.map(|x| x + shift);
            for point in &mut edge.points {
                point.x += shift;
            }
        }
    }
    graph.graph_mut().width += shift;
    Ok(())
}

fn refined_cubic_min_x(cubic: [Point; 4], budget: &mut dyn LayoutWork) -> Result<f64, RenderError> {
    let mut stack = vec![(cubic, 0_u8)];
    let mut left = f64::INFINITY;
    while let Some((curve, depth)) = stack.pop() {
        budget.charge(1)?;
        let chord = (curve[0], curve[3]);
        let flat = [curve[1], curve[2]].iter().all(|point| {
            let dx = chord.1.x - chord.0.x;
            let dy = chord.1.y - chord.0.y;
            let cross = dx * (point.y - chord.0.y) - dy * (point.x - chord.0.x);
            let squared = dx * dx + dy * dy;
            squared < 1e-10 || cross * cross / squared < 4.
        });
        if flat || depth == 12 {
            left = curve.iter().fold(left, |min, point| min.min(point.x));
        } else {
            let (_, first, second) = split_cubic(curve, 0.5);
            stack.push((second, depth + 1));
            stack.push((first, depth + 1));
        }
    }
    Ok(left)
}

fn has_clear_shortest_path(
    graph: &LayoutGraph,
    endpoints: (&str, &str),
    segment: (Point, Point),
) -> bool {
    let parallel = graph
        .edges()
        .filter(|edge| {
            (edge.v == endpoints.0 && edge.w == endpoints.1)
                || (edge.v == endpoints.1 && edge.w == endpoints.0)
        })
        .count();
    if parallel > 1 {
        return false;
    }
    has_clear_segment(graph, endpoints, segment)
}

fn has_clear_segment(
    graph: &LayoutGraph,
    endpoints: (&str, &str),
    segment: (Point, Point),
) -> bool {
    graph
        .nodes()
        .filter(|id| *id != endpoints.0 && *id != endpoints.1 && !is_dot_cluster(id))
        .all(|id| {
            graph
                .node(id)
                .is_some_and(|node| !segment_crosses_box(segment, node))
        })
}

fn is_clear_parallel_center(
    graph: &LayoutGraph,
    key: &dugong::graphlib::EdgeKey,
    segment: (Point, Point),
) -> bool {
    // Each labeled edge has its own virtual rank node in class2.c. For an odd
    // number of parallel edges, the middle corridor remains a direct line.
    if !matches!(graph.graph().rankdir, RankDir::TB | RankDir::BT)
        || (segment.0.x - segment.1.x).abs() > 1e-9
    {
        return false;
    }
    let parallel = graph
        .edges()
        .filter(|edge| edge.v == key.v && edge.w == key.w)
        .collect::<Vec<_>>();
    if parallel.len() < 3 || parallel.len() % 2 == 0 {
        return false;
    }
    if parallel.iter().any(|edge| {
        graph.edge_by_key(edge).is_none_or(|label| {
            !label
                .extras
                .get("dot_label_present")
                .and_then(|value| value.as_bool())
                .unwrap_or(false)
        })
    }) {
        return false;
    }
    parallel[parallel.len() / 2] == key && has_clear_segment(graph, (&key.v, &key.w), segment)
}

fn parallel_side_lane(
    graph: &LayoutGraph,
    key: &dugong::graphlib::EdgeKey,
    segment: (Point, Point),
) -> Option<f64> {
    if !matches!(graph.graph().rankdir, RankDir::TB)
        || (segment.0.x - segment.1.x).abs() > 1e-9
        || !has_clear_segment(graph, (&key.v, &key.w), segment)
    {
        return None;
    }
    let parallel = graph
        .edges()
        .filter(|edge| edge.v == key.v && edge.w == key.w)
        .collect::<Vec<_>>();
    if parallel.len() != 3 {
        return None;
    }
    let labels = parallel
        .iter()
        .map(|edge| graph.edge_by_key(edge))
        .collect::<Option<Vec<_>>>()?;
    let width = labels[0].width;
    if labels.iter().any(|label| {
        (label.width - width).abs() > 1e-9
            || !label
                .extras
                .get("dot_label_present")
                .and_then(|value| value.as_bool())
                .unwrap_or(false)
    }) {
        return None;
    }
    let spacing = (graph.graph().nodesep + width + 5.).floor();
    match parallel.iter().position(|edge| *edge == key)? {
        0 => Some(segment.0.x - spacing + 10. - width),
        2 => Some(segment.0.x + (width + 2.).round() - 1.),
        _ => None,
    }
}

fn undirected_feedback_lane(
    graph: &LayoutGraph,
    key: &dugong::graphlib::EdgeKey,
    segment: (Point, Point),
) -> Option<(f64, f64)> {
    if graph.is_directed()
        || !matches!(graph.graph().rankdir, RankDir::TB)
        || (segment.0.x - segment.1.x).abs() > 1e-9
    {
        return None;
    }
    let low = segment.0.y.min(segment.1.y);
    let high = segment.0.y.max(segment.1.y);
    let obstacles = graph
        .nodes()
        .filter(|id| *id != key.v && *id != key.w && !is_dot_cluster(id))
        .filter_map(|id| graph.node(id))
        .filter(|node| node.y.is_some_and(|y| y > low && y < high))
        .collect::<Vec<_>>();
    if obstacles.len() != 1 {
        return None;
    }
    let obstacle = obstacles[0];
    Some((
        obstacle.x? + obstacle.width / 2. + graph.graph().nodesep / 2.,
        obstacle.height,
    ))
}

fn cluster_crossing_waypoint(
    graph: &LayoutGraph,
    key: &dugong::graphlib::EdgeKey,
    segment: (Point, Point),
) -> Option<Point> {
    if !matches!(graph.graph().rankdir, RankDir::TB) || (segment.0.x - segment.1.x).abs() < 1e-9 {
        return None;
    }
    let source_cluster = graph.parent(&key.v)?;
    let target_cluster = graph.parent(&key.w)?;
    if source_cluster == target_cluster
        || !is_dot_cluster(source_cluster)
        || !is_dot_cluster(target_cluster)
    {
        return None;
    }
    let target = graph.node(target_cluster)?;
    Some(Point {
        x: (segment.0.x + segment.1.x) / 2.,
        y: target.y? - target.height / 2.,
    })
}

fn route_cluster_crossing(
    segment: (Point, Point),
    waypoint: Point,
    sizes: (Size, Size),
    shapes: (ClipShape, ClipShape),
    arrow: ArrowMode,
) -> Vec<Point> {
    // Proutespline splits the three-point shortest path at its middle vertex.
    // With unconstrained endpoints, mkspline/splinefits use a shared average
    // tangent and a 4/9 chord-length control offset on each side.
    let (start, end) = segment;
    let first = Point {
        x: start.x,
        y: start.y + 1.,
    };
    let last = Point {
        x: end.x,
        y: end.y - 1.,
    };
    let first_vector = normalize(Point {
        x: waypoint.x - first.x,
        y: waypoint.y - first.y,
    });
    let last_vector = normalize(Point {
        x: last.x - waypoint.x,
        y: last.y - waypoint.y,
    });
    let tangent = normalize(Point {
        x: first_vector.x + last_vector.x,
        y: first_vector.y + last_vector.y,
    });
    let first_length = distance(first, waypoint) * 4. / 9.;
    let last_length = distance(waypoint, last) * 4. / 9.;
    let first_control = Point {
        x: waypoint.x - tangent.x * first_length,
        y: waypoint.y - tangent.y * first_length,
    };
    let last_control = Point {
        x: waypoint.x + tangent.x * last_length,
        y: waypoint.y + tangent.y * last_length,
    };
    let first_curve = clip_cubic(
        [first, first, first_control, waypoint],
        ClipEnd::Start,
        |point| inside_shape(point, start, sizes.0, shapes.0),
    );
    let mut last_curve = clip_cubic(
        [waypoint, last_control, last, last],
        ClipEnd::End,
        |point| inside_shape(point, end, sizes.1, shapes.1),
    );
    let tip = last_curve[3];
    if matches!(arrow, ArrowMode::Head) {
        last_curve = clip_cubic(last_curve, ClipEnd::End, |point| {
            distance(point, tip) <= normal_arrow_stem_length()
        });
    }
    let mut points = first_curve.to_vec();
    points.extend(last_curve.into_iter().skip(1));
    if matches!(arrow, ArrowMode::Head) {
        points.push(tip);
    }
    points
}

fn normalize(vector: Point) -> Point {
    let length = vector.x.hypot(vector.y);
    if length == 0. {
        return Point { x: 0., y: 0. };
    }
    Point {
        x: vector.x / length,
        y: vector.y / length,
    }
}

fn route_parallel_polyline(
    segment: (Point, Point),
    sizes: (Size, Size),
    lane: f64,
    label_height: f64,
    arrow: ArrowMode,
    shapes: (ClipShape, ClipShape),
) -> Vec<Point> {
    // routepolylines follows a shortest path through the rank boxes, then
    // pathplan::make_polyline duplicates each internal waypoint three times.
    let (start, end) = segment;
    let (source, target) = sizes;
    let direction = (end.y - start.y).signum();
    let middle = (start.y + end.y) / 2.;
    let first_waypoint = Point {
        x: lane,
        y: middle - direction * label_height / 2.,
    };
    let second_waypoint = Point {
        x: lane,
        y: middle + direction * label_height / 2.,
    };
    let start_port = Point {
        x: start.x,
        y: start.y + direction,
    };
    let end_port = Point {
        x: end.x,
        y: end.y - direction,
    };
    let first = clip_cubic(
        [start_port, start_port, first_waypoint, first_waypoint],
        ClipEnd::Start,
        |point| inside_shape(point, start, source, shapes.0),
    );
    let middle_curve = [
        first_waypoint,
        first_waypoint,
        second_waypoint,
        second_waypoint,
    ];
    let mut last = clip_cubic(
        [second_waypoint, second_waypoint, end_port, end_port],
        ClipEnd::End,
        |point| inside_shape(point, end, target, shapes.1),
    );
    let tip = last[3];
    if matches!(arrow, ArrowMode::Head) {
        last = clip_cubic(last, ClipEnd::End, |point| {
            distance(point, tip) <= normal_arrow_stem_length()
        });
    }
    let mut points = Vec::with_capacity(11);
    points.extend(first);
    points.extend(middle_curve.into_iter().skip(1));
    points.extend(last.into_iter().skip(1));
    if matches!(arrow, ArrowMode::Head) {
        points.push(tip);
    }
    points
}

fn route_record_ports(tail: Point, head: Point, arrow: ArrowMode) -> Vec<Point> {
    // A record compass port is already on the field boundary. Graphviz's
    // polyline route starts there and clips only the arrow stem at the head.
    let mut curve = [tail, tail, head, head];
    if matches!(arrow, ArrowMode::Head) {
        curve = clip_cubic(curve, ClipEnd::End, |point| {
            distance(point, head) <= normal_arrow_stem_length()
        });
        let mut points = curve.to_vec();
        points.push(head);
        return points;
    }
    curve.to_vec()
}

fn segment_crosses_box(segment: (Point, Point), node: &NodeLabel) -> bool {
    let (Some(cx), Some(cy)) = (node.x, node.y) else {
        return true;
    };
    let mut enter: f64 = 0.;
    let mut exit: f64 = 1.;
    for (origin, delta, center, extent) in [
        (segment.0.x, segment.1.x - segment.0.x, cx, node.width / 2.),
        (segment.0.y, segment.1.y - segment.0.y, cy, node.height / 2.),
    ] {
        if delta.abs() < 1e-9 {
            if origin < center - extent || origin > center + extent {
                return false;
            }
            continue;
        }
        let first = (center - extent - origin) / delta;
        let last = (center + extent - origin) / delta;
        enter = enter.max(first.min(last));
        exit = exit.min(first.max(last));
        if enter > exit {
            return false;
        }
    }
    true
}

#[derive(Clone, Copy)]
enum ClipEnd {
    Start,
    End,
}

#[derive(Clone, Copy)]
enum ArrowMode {
    Head,
    None,
}

#[derive(Clone, Copy)]
enum ClipShape {
    Box(f64),
    Ellipse(f64),
    Diamond(f64),
}

struct SelfRoute {
    center: Point,
    size: Size,
    nodesep: f64,
    rank_distance: f64,
    shape: ClipShape,
    arrow: ArrowMode,
}

fn route_self_right(route: SelfRoute) -> Vec<Point> {
    // splines.c::selfRight constructs two cubics. clip_and_install clips both
    // ends against the stroked node outline, then shortens the head for an arrow.
    let middle_x = route.center.x + route.size.width / 2. + route.nodesep;
    let control_x = route.center.x + (route.size.width / 2. + route.nodesep) / 3.;
    let step_y = (route.rank_distance / 4.).max(2.);
    let raw = [
        route.center,
        Point {
            x: control_x,
            y: route.center.y - step_y,
        },
        Point {
            x: middle_x,
            y: route.center.y - step_y,
        },
        Point {
            x: middle_x,
            y: route.center.y,
        },
        Point {
            x: middle_x,
            y: route.center.y + step_y,
        },
        Point {
            x: control_x,
            y: route.center.y + step_y,
        },
        route.center,
    ];
    let first = clip_cubic(
        raw[..4].try_into().expect("four controls"),
        ClipEnd::Start,
        |point| inside_shape(point, route.center, route.size, route.shape),
    );
    let mut last = clip_cubic(
        raw[3..].try_into().expect("four controls"),
        ClipEnd::End,
        |point| inside_shape(point, route.center, route.size, route.shape),
    );
    let tip = last[3];
    if matches!(route.arrow, ArrowMode::Head) {
        last = clip_cubic(last, ClipEnd::End, |point| {
            distance(point, tip) <= normal_arrow_stem_length()
        });
    }
    let mut points = first.to_vec();
    points.extend_from_slice(&last[1..]);
    if matches!(route.arrow, ArrowMode::Head) {
        points.push(tip);
    }
    points
}

#[derive(Clone, Copy)]
enum RouteFlow {
    Forward,
    Backward,
}

struct StraightRoute {
    start: Point,
    end: Point,
    source: Size,
    target: Size,
    source_shape: ClipShape,
    target_shape: ClipShape,
    arrow: ArrowMode,
    tail_arrow: bool,
    head_arrow_length: f64,
    tail_arrow_length: f64,
    rankdir: RankDir,
    flow: RouteFlow,
}

fn route_straight_cubic(route: StraightRoute) -> Vec<Point> {
    // DOT's polyline router converts the straight shortest path to a cubic, then
    // clips the cubic to both node outlines and the arrow stem (splines.c).
    let (start, end, source, target, source_shape, target_shape) = match route.flow {
        RouteFlow::Forward => (
            route.start,
            route.end,
            route.source,
            route.target,
            route.source_shape,
            route.target_shape,
        ),
        RouteFlow::Backward => (
            route.end,
            route.start,
            route.target,
            route.source,
            route.target_shape,
            route.source_shape,
        ),
    };
    let mut cubic = raw_edge_cubic(start, end, route.rankdir);
    cubic = clip_cubic(cubic, ClipEnd::Start, |point| {
        inside_shape(point, start, source, source_shape)
    });
    cubic = clip_cubic(cubic, ClipEnd::End, |point| {
        inside_shape(point, end, target, target_shape)
    });
    let source_end = match route.flow {
        RouteFlow::Forward => ClipEnd::Start,
        RouteFlow::Backward => ClipEnd::End,
    };
    let target_end = match route.flow {
        RouteFlow::Forward => ClipEnd::End,
        RouteFlow::Backward => ClipEnd::Start,
    };
    let tail_tip = if route.tail_arrow {
        let tip = match source_end {
            ClipEnd::Start => cubic[0],
            ClipEnd::End => cubic[3],
        };
        cubic = clip_cubic(cubic, source_end, |point| {
            distance(point, tip) <= route.tail_arrow_length
        });
        Some(tip)
    } else {
        None
    };
    let head_tip = if matches!(route.arrow, ArrowMode::Head) {
        let tip = match target_end {
            ClipEnd::Start => cubic[0],
            ClipEnd::End => cubic[3],
        };
        cubic = clip_cubic(cubic, target_end, |point| {
            distance(point, tip) <= route.head_arrow_length
        });
        Some(tip)
    } else {
        None
    };
    let mut points =
        Vec::with_capacity(4 + usize::from(tail_tip.is_some()) + usize::from(head_tip.is_some()));
    points.extend(tail_tip);
    match route.flow {
        RouteFlow::Forward => points.extend(cubic),
        RouteFlow::Backward => points.extend(cubic.into_iter().rev()),
    }
    points.extend(head_tip);
    points
}

fn raw_edge_cubic(start: Point, end: Point, rankdir: RankDir) -> [Point; 4] {
    let offset = match rankdir {
        RankDir::TB | RankDir::BT => Point {
            x: 0.,
            y: (end.y - start.y).signum(),
        },
        RankDir::LR | RankDir::RL => Point {
            x: (end.x - start.x).signum(),
            y: 0.,
        },
    };
    let start_port = Point {
        x: start.x + offset.x,
        y: start.y + offset.y,
    };
    let end_port = Point {
        x: end.x - offset.x,
        y: end.y - offset.y,
    };
    [start_port, start_port, end_port, end_port]
}

fn virtual_label_x(start: Point, end: Point, label_height: f64) -> f64 {
    // routespl.c::limitBoxes samples each path box at ten points; a regular
    // adjacent-rank edge with one label virtual node has five path boxes.
    // dotsplines.c::recover_slack uses the sampled right bound as the
    // virtual label node's x coordinate.
    let cubic = raw_edge_cubic(start, end, RankDir::TB);
    let center = (start.y + end.y) / 2.;
    let half = label_height / 2.;
    let mut right = f64::NEG_INFINITY;
    for sample in 0..=50 {
        let point = split_cubic(cubic, sample as f64 / 50.).0;
        if (point.y - center).abs() <= half + 0.0001 {
            right = right.max(point.x);
        }
    }
    if right.is_finite() {
        return right;
    }
    (start.x + end.x) / 2.
}

fn virtual_port_label_y(tail: Point, head: Point, width: f64, height: f64) -> f64 {
    // In LR layout, recover_slack uses the upper route bound across the label
    // rank. limitBoxes samples the five path boxes at ten points each.
    let cubic = [tail, tail, head, head];
    let middle = (tail.x + head.x) / 2.;
    let mut upper = f64::INFINITY;
    for sample in 0..=50 {
        let point = split_cubic(cubic, sample as f64 / 50.).0;
        if (point.x - middle).abs() <= width / 2. + 0.0001 {
            upper = upper.min(point.y);
        }
    }
    if upper.is_finite() {
        return upper - height / 2.;
    }
    (tail.y + head.y) / 2. - height / 2.
}

fn edge_clip_shape(
    graph: &LayoutGraph,
    key: &dugong::graphlib::EdgeKey,
    shape_key: &str,
) -> ClipShape {
    let edge = graph.edge_by_key(key);
    let shape = edge
        .and_then(|edge| edge.extras.get(shape_key))
        .and_then(|value| value.as_str());
    let penwidth = shape_key
        .strip_suffix("_shape")
        .map(|prefix| format!("{prefix}_penwidth"))
        .and_then(|key| {
            edge.and_then(|edge| edge.extras.get(&key))
                .and_then(|value| value.as_f64())
        })
        .unwrap_or(1.);
    match shape {
        Some("plaintext") => ClipShape::Box(0.),
        Some("ellipse" | "oval" | "circle") => ClipShape::Ellipse(penwidth),
        Some("diamond") => ClipShape::Diamond(penwidth),
        _ => ClipShape::Box(penwidth),
    }
}

fn inside_box_pen(point: Point, center: Point, size: Size, pen: f64) -> bool {
    // Graphviz's polygon outline includes half the default 1 pt node pen width.
    (point.x - center.x).abs() <= (size.width + pen) / 2.
        && (point.y - center.y).abs() <= (size.height + pen) / 2.
}

fn inside_shape(point: Point, center: Point, size: Size, shape: ClipShape) -> bool {
    match shape {
        ClipShape::Box(pen) => inside_box_pen(point, center, size, pen),
        ClipShape::Ellipse(pen) => {
            let x = (point.x - center.x) / ((size.width + pen) / 2.);
            let y = (point.y - center.y) / ((size.height + pen) / 2.);
            x.hypot(y) < 1.
        }
        ClipShape::Diamond(pen) => {
            let half_width = size.width / 2.;
            let half_height = size.height / 2.;
            let diagonal = half_width.hypot(half_height);
            let outline_x = half_width + pen / 2. * diagonal / half_height;
            let outline_y = half_height + pen / 2. * diagonal / half_width;
            (point.x - center.x).abs() / outline_x + (point.y - center.y).abs() / outline_y < 1.
        }
    }
}

fn normal_arrow_stem_length() -> f64 {
    // arrows.c's mitered normal-arrow tip extends the 10 pt nominal length.
    let nominal = 10_f64;
    let half_base = nominal * 0.35;
    nominal + 0.5 * nominal.hypot(half_base) / half_base
}

fn dot_arrow_stem_length(shape: &str, size: f64, penwidth: f64) -> f64 {
    let half_stroke = 0.5;
    match shape {
        "vee" => 10. * size + half_stroke * 10_f64.hypot(4.5) / 4.5,
        "diamond" => {
            let miter = half_stroke * 6_f64.hypot(4.) / 4.;
            let half_length = 6. * size + miter;
            let full_base_width = 8. * size * half_length / (6. * size);
            2. * half_length - half_length / full_base_width
        }
        "box" => 10. * size + half_stroke,
        _ => 10. * size + penwidth / 2. * 10_f64.hypot(3.5) / 3.5,
    }
}

fn distance(a: Point, b: Point) -> f64 {
    (a.x - b.x).hypot(a.y - b.y)
}

fn clip_cubic(cubic: [Point; 4], end: ClipEnd, inside: impl Fn(Point) -> bool) -> [Point; 4] {
    let mut low = 0.;
    let mut high = 1.;
    let mut previous = match end {
        ClipEnd::Start => cubic[0],
        ClipEnd::End => cubic[3],
    };
    let mut best = None;
    loop {
        let t = (low + high) / 2.;
        let (point, left, right) = split_cubic(cubic, t);
        let is_inside = inside(point);
        let segment = match end {
            ClipEnd::Start => right,
            ClipEnd::End => left,
        };
        if is_inside {
            best = Some(segment);
        }
        match (end, is_inside) {
            (ClipEnd::Start, true) | (ClipEnd::End, false) => low = t,
            (ClipEnd::Start, false) | (ClipEnd::End, true) => high = t,
        }
        if (point.x - previous.x).abs() <= 0.5 && (point.y - previous.y).abs() <= 0.5 {
            return best.unwrap_or(segment);
        }
        previous = point;
    }
}

fn split_cubic(cubic: [Point; 4], t: f64) -> (Point, [Point; 4], [Point; 4]) {
    let first = [
        lerp(cubic[0], cubic[1], t),
        lerp(cubic[1], cubic[2], t),
        lerp(cubic[2], cubic[3], t),
    ];
    let second = [lerp(first[0], first[1], t), lerp(first[1], first[2], t)];
    let point = lerp(second[0], second[1], t);
    (
        point,
        [cubic[0], first[0], second[0], point],
        [point, second[1], first[2], cubic[3]],
    )
}

fn lerp(a: Point, b: Point, t: f64) -> Point {
    Point {
        x: a.x * (1. - t) + b.x * t,
        y: a.y * (1. - t) + b.y * t,
    }
}

fn clip(center: Point, width: f64, height: f64, toward: Point) -> Point {
    let dx = toward.x - center.x;
    let dy = toward.y - center.y;
    let scale = (dx.abs() / (width / 2.).max(1.)).max(dy.abs() / (height / 2.).max(1.));
    if scale <= f64::EPSILON {
        return center;
    }
    Point {
        x: center.x + dx / scale,
        y: center.y + dy / scale,
    }
}

fn failure(message: &str) -> RenderError {
    RenderError::Backend(format!("Rust DOT layout: {message}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GraphLayout, RenderOptions};
    use dugong::graphlib::GraphOptions;

    fn graph() -> LayoutGraph {
        let mut graph = Graph::new(GraphOptions {
            directed: true,
            multigraph: true,
            compound: true,
        });
        graph.set_graph(GraphLabel {
            ranksep: 60.,
            nodesep: 40.,
            ..Default::default()
        });
        for id in ["a", "b", "c"] {
            graph.set_node(
                id,
                NodeLabel {
                    width: 80.,
                    height: 40.,
                    ..Default::default()
                },
            );
        }
        graph
    }

    fn budget() -> Budget {
        Budget::new(&RenderOptions {
            graph_layout: GraphLayout::Layered,
            ..Default::default()
        })
    }

    #[test]
    fn should_round_graphviz_label_virtual_rank_separation() {
        // given: Graphviz 16.1.0 class2.c and position.c use asymmetric
        // label widths and a 5 pt separator on the doubled label rank.
        let center = VirtualLabelUnit {
            rank: 1,
            left_width: 18.,
            right_width: 21.,
        };
        let left_labels = [20.2138671875, 18.6552734375, 22.5517578125];

        // when
        let separations = left_labels.map(|width| {
            center.separation_after(VirtualLabelUnit {
                rank: 1,
                left_width: 18.,
                right_width: width,
            })
        });

        // then
        assert_eq!(separations, [43., 42., 46.]);
    }

    #[test]
    fn should_recover_left_label_from_path_box_for_varied_feedback_labels() {
        // given: independent Graphviz 16.1.0 JSON0 oracles; the remaining
        // absolute x offset belongs to spline extent normalization.
        let cases = [
            (
                include_str!("goldens/mixed_feedback_labeled.dot"),
                include_str!("goldens/mixed_feedback_labeled.json"),
            ),
            (
                include_str!("goldens/mixed_feedback_varied.dot"),
                include_str!("goldens/mixed_feedback_varied.json"),
            ),
        ];
        let options = RenderOptions {
            language: crate::Language::Dot,
            graph_layout: GraphLayout::Layered,
            ..Default::default()
        };
        let x = |value: &serde_json::Value| -> f64 {
            value
                .as_str()
                .unwrap()
                .split(',')
                .next()
                .unwrap()
                .parse()
                .unwrap()
        };

        // when and then
        for (source, oracle) in cases {
            let actual: serde_json::Value =
                serde_json::from_str(&crate::render_dot_json(source, &options).unwrap()).unwrap();
            let expected: serde_json::Value = serde_json::from_str(oracle).unwrap();
            let actual_node = x(&actual["objects"][0]["pos"]);
            let expected_node = x(&expected["objects"][0]["pos"]);
            let actual_label = x(&actual["edges"][0]["lp"]);
            let expected_label = x(&expected["edges"][0]["lp"]);
            assert!((actual_node - actual_label - (expected_node - expected_label)).abs() < 0.0001);
            assert!(actual_label < actual_node);
        }
    }

    #[test]
    fn should_match_mixed_feedback_json_for_varied_labels() {
        // given
        let cases = [
            (
                include_str!("goldens/mixed_feedback_labeled.dot"),
                include_str!("goldens/mixed_feedback_labeled.json"),
            ),
            (
                include_str!("goldens/mixed_feedback_varied.dot"),
                include_str!("goldens/mixed_feedback_varied.json"),
            ),
        ];
        let options = RenderOptions {
            language: crate::Language::Dot,
            graph_layout: GraphLayout::Layered,
            ..Default::default()
        };

        // when and then
        for (source, oracle) in cases {
            let actual: serde_json::Value =
                serde_json::from_str(&crate::render_dot_json(source, &options).unwrap()).unwrap();
            let expected: serde_json::Value = serde_json::from_str(oracle).unwrap();
            let mut actual = actual;
            let mut expected = expected;
            for document in [&mut actual, &mut expected] {
                for edge in document["edges"].as_array_mut().unwrap() {
                    edge.as_object_mut().unwrap().remove("_gvid");
                }
            }
            assert_eq!(actual, expected, "{source}");
        }
    }

    #[test]
    fn should_place_acyclic_nodes_in_separate_ranks_and_route_edges() {
        // given
        let mut graph = graph();
        graph.set_edge("a", "b");
        graph.set_edge("b", "c");

        // when
        layout(&mut graph, &mut budget()).unwrap();

        // then
        assert!(graph.node("a").unwrap().y.unwrap() < graph.node("b").unwrap().y.unwrap());
        assert!(graph.node("b").unwrap().y.unwrap() < graph.node("c").unwrap().y.unwrap());
        assert_eq!(graph.edge("a", "b", None).unwrap().points.len(), 2);
    }

    #[test]
    fn should_include_dot_edge_label_in_graph_right_bound() {
        // given
        let source = "digraph uml_object { graph [rankdir=TB]; node [shape=box]; n0 [label=\"basket : Cart\"]; n1 [label=\"line : Item\"]; n0 -> n1 [label=contains]; }";
        let options = RenderOptions {
            language: crate::Language::Dot,
            graph_layout: GraphLayout::Layered,
            ..Default::default()
        };

        // when
        let json = crate::render_dot_json(source, &options).unwrap();
        let geometry: serde_json::Value = serde_json::from_str(&json).unwrap();

        // then
        assert_eq!(geometry["bb"], "0,0,90.035,124.8");
    }

    #[test]
    fn should_align_class_fork_with_integer_auxiliary_constraints() {
        // given
        let source = "digraph uml_class { graph [rankdir=TB]; node [shape=box]; n0 [label=\"Account\"]; n1 [label=\"Premium\"]; n2 [label=\"Ledger\"]; n0 -> n1; n0 -> n2 [label=\"records\"]; }";
        let options = RenderOptions {
            language: crate::Language::Dot,
            graph_layout: GraphLayout::Layered,
            ..Default::default()
        };

        // when
        let json = crate::render_dot_json(source, &options).unwrap();
        let geometry: serde_json::Value = serde_json::from_str(&json).unwrap();
        let nodes = geometry["objects"].as_array().unwrap();
        let account = nodes.iter().find(|node| node["name"] == "n0").unwrap();

        // then
        assert_eq!(account["pos"], "72.666,106.8");
        assert_eq!(
            geometry["edges"][0]["pos"],
            "e,41.488,36.41 64.96,88.651 59.6,76.722 52.336,60.554 46.155,46.798"
        );
        assert_eq!(geometry["edges"][1]["lp"], "116.85,62.4");
    }

    #[test]
    fn should_reserve_nested_cluster_label_space_before_first_rank() {
        // given
        let source = "digraph uml_deployment { graph [rankdir=TB]; node [shape=box]; subgraph cluster_0 { label=\"Cluster\"; n0 [label=\"Gateway\"]; subgraph cluster_1 { label=\"Worker pool\"; n1 [label=\"Worker\"]; } n2 [label=\"Store\"]; } n0 -> n1 [label=\"dispatch\"]; n1 -> n2 [label=\"save\"]; }";
        let options = RenderOptions {
            language: crate::Language::Dot,
            graph_layout: GraphLayout::Layered,
            ..Default::default()
        };

        // when
        let json = crate::render_dot_json(source, &options).unwrap();
        let geometry: serde_json::Value = serde_json::from_str(&json).unwrap();
        let nodes = geometry["objects"].as_array().unwrap();
        let worker = nodes.iter().find(|node| node["name"] == "n1").unwrap();
        let store = nodes.iter().find(|node| node["name"] == "n2").unwrap();
        let outer = nodes
            .iter()
            .find(|node| node["name"] == "cluster_0")
            .unwrap();
        let inner = nodes
            .iter()
            .find(|node| node["name"] == "cluster_1")
            .unwrap();

        // then
        assert_eq!(geometry["bb"], "0,0,119,293.2");
        assert_eq!(outer["bb"], "8,8,111,285.2");
        assert_eq!(inner["bb"], "16,96.8,103,173.6");
        assert_eq!(worker["pos"], "54,122.8");
        assert_eq!(store["pos"], "54,34");
        assert_eq!(geometry["edges"][0]["lp"], "77.328,190");
    }

    #[test]
    fn should_keep_integer_column_for_nested_cluster_with_wider_leaf() {
        // given
        let source = "digraph G { graph [rankdir=TB]; node [shape=box]; subgraph cluster_0 { label=\"Cluster\"; n0 [label=\"Gateway\"]; subgraph cluster_1 { label=\"Worker pool\"; n1 [label=\"Worker\"]; } n2 [label=\"Storage\"]; } n0 -> n1 [label=\"dispatch\"]; n1 -> n2 [label=\"save\"]; }";
        let options = RenderOptions {
            language: crate::Language::Dot,
            graph_layout: GraphLayout::Layered,
            ..Default::default()
        };

        // when
        let json = crate::render_dot_json(source, &options).unwrap();
        let geometry: serde_json::Value = serde_json::from_str(&json).unwrap();
        let nodes = geometry["objects"].as_array().unwrap();
        let centers: Vec<_> = nodes
            .iter()
            .filter(|node| {
                node["name"]
                    .as_str()
                    .is_some_and(|name| name.starts_with('n'))
            })
            .map(|node| node["pos"].as_str().unwrap().split(',').next().unwrap())
            .collect();

        // then
        assert_eq!(geometry["bb"], "0,0,119,293.2");
        assert_eq!(centers, ["54", "54", "54"]);
    }

    #[test]
    fn should_reserve_outer_cluster_label_space_before_first_rank() {
        // given
        let source = "digraph uml_usecase { graph [rankdir=TB]; node [shape=box]; n0 [label=\"Shopper\"]; subgraph cluster_0 { label=\"Store\"; n1 [label=\"Place order\"]; n2 [label=\"Pay invoice\"]; } n0 -> n1; n1 -> n2 [label=\"includes\",style=dashed]; }";
        let options = RenderOptions {
            language: crate::Language::Dot,
            graph_layout: GraphLayout::Layered,
            ..Default::default()
        };

        // when
        let json = crate::render_dot_json(source, &options).unwrap();
        let geometry: serde_json::Value = serde_json::from_str(&json).unwrap();
        let nodes = geometry["objects"].as_array().unwrap();
        let order = nodes.iter().find(|node| node["name"] == "n1").unwrap();
        let invoice = nodes.iter().find(|node| node["name"] == "n2").unwrap();
        let cluster = nodes
            .iter()
            .find(|node| node["name"] == "cluster_0")
            .unwrap();

        // then
        assert_eq!(geometry["bb"], "0,0,114,236.6");
        assert_eq!(cluster["bb"], "8,8,106,173.6");
        assert_eq!(cluster["lp"], "57,161.2");
        assert_eq!(order["pos"], "56,122.8");
        assert_eq!(invoice["pos"], "57,34");
        assert_eq!(
            geometry["edges"][0]["pos"],
            "e,56,141.21 56,200.4 56,186.99 56,168.09 56,152.42"
        );
        assert_eq!(geometry["edges"][1]["lp"], "79.917,78.4");
        assert_eq!(
            geometry["edges"][1]["pos"],
            "e,56.799,52.41 56.198,104.65 56.332,92.958 56.514,77.192 56.67,63.618"
        );
    }

    #[test]
    fn should_match_integer_cluster_chain_with_independent_labels() {
        // given
        let source = "digraph G { graph [rankdir=TB]; node [shape=box]; visitor [label=\"Visitor\"]; subgraph cluster_0 { label=\"Area\"; first [label=\"Supplier\"]; second [label=\"Dispatch\"]; } visitor -> first; first -> second [label=\"includes\",style=dashed]; }";
        let options = RenderOptions {
            language: crate::Language::Dot,
            graph_layout: GraphLayout::Layered,
            ..Default::default()
        };

        // when
        let json = crate::render_dot_json(source, &options).unwrap();
        let geometry: serde_json::Value = serde_json::from_str(&json).unwrap();
        let nodes = geometry["objects"].as_array().unwrap();
        let by_name = |name| nodes.iter().find(|node| node["name"] == name).unwrap();

        // then
        assert_eq!(geometry["bb"], "0,0,98,236.6");
        assert_eq!(by_name("visitor")["pos"], "48,218.6");
        assert_eq!(by_name("first")["pos"], "48,122.8");
        assert_eq!(by_name("second")["pos"], "49,34");
        assert_eq!(
            geometry["edges"][0]["pos"],
            "e,48,141.21 48,200.4 48,186.99 48,168.09 48,152.42"
        );
        assert_eq!(
            geometry["edges"][1]["pos"],
            "e,48.799,52.41 48.198,104.65 48.332,92.958 48.514,77.192 48.67,63.618"
        );
        assert_eq!(geometry["edges"][1]["lp"], "71.917,78.4");
    }

    #[test]
    fn should_render_compound_uml_and_state_with_rust_layout() {
        // given
        let options = RenderOptions {
            language: crate::Language::PlantUml,
            graph_layout: GraphLayout::Layered,
            ..Default::default()
        };
        let sources = [
            "@startuml\npackage Shop {\nclass Customer\nclass Order\nCustomer --> Order\n}\n@enduml",
            "@startuml\nstate Ready\nstate Busy\nReady --> Busy\nBusy --> Ready\n@enduml",
        ];

        // when
        let rendered = sources.map(|source| crate::render(source, &options).unwrap());

        // then
        assert!(rendered.iter().all(|diagram| diagram.svg.contains("<svg")));
        assert!(rendered[0].svg.contains("Customer"));
        assert!(rendered[1].svg.contains("Ready"));
    }

    #[test]
    fn should_align_external_edges_to_their_descendant_anchor() {
        // given
        let mut graph = Graph::new(GraphOptions {
            directed: true,
            multigraph: true,
            compound: true,
        });
        graph.set_graph(GraphLabel {
            ranksep: 60.,
            nodesep: 40.,
            ..Default::default()
        });
        for id in ["p", "caption", "a", "b"] {
            graph.set_node(
                id,
                NodeLabel {
                    width: 80.,
                    height: 40.,
                    ..Default::default()
                },
            );
        }
        graph
            .try_set_unparented_parents_ix(&[
                (
                    graph.node_ix("caption").unwrap(),
                    graph.node_ix("p").unwrap(),
                ),
                (graph.node_ix("a").unwrap(), graph.node_ix("p").unwrap()),
            ])
            .unwrap();
        graph.set_edge("a", "b");

        // when
        layout(&mut graph, &mut budget()).unwrap();

        // then
        let a = graph.node("a").unwrap().x.unwrap();
        let b = graph.node("b").unwrap().x.unwrap();
        assert_eq!(a, b);
        assert_eq!(graph.node("p").unwrap().width, 344.);
    }

    #[test]
    fn should_include_cluster_owner_and_children_in_cluster_bounds() {
        // given
        let mut graph = graph();
        graph.set_node(
            "p",
            NodeLabel {
                width: 80.,
                height: 40.,
                ..Default::default()
            },
        );
        graph
            .try_set_unparented_parents_ix(&[
                (graph.node_ix("a").unwrap(), graph.node_ix("p").unwrap()),
                (graph.node_ix("b").unwrap(), graph.node_ix("p").unwrap()),
            ])
            .unwrap();

        // when
        layout(&mut graph, &mut budget()).unwrap();

        // then
        let cluster = graph.node("p").unwrap();
        assert_eq!(cluster.width, 344.);
        assert_eq!(cluster.height, 64.);
    }

    #[test]
    fn should_match_dot_cycle_node_centers_with_feedback_channel() {
        // given
        let mut graph = graph();
        graph.set_node(
            "d",
            NodeLabel {
                width: 80.,
                height: 40.,
                ..Default::default()
            },
        );
        for (from, to) in [("a", "b"), ("b", "c"), ("c", "a")] {
            graph.set_edge(from, to);
        }

        // when
        layout(&mut graph, &mut budget()).unwrap();

        // then
        let positions = ["a", "b", "c", "d"].map(|id| {
            let node = graph.node(id).unwrap();
            (node.x.unwrap(), node.y.unwrap())
        });
        assert_eq!(
            positions,
            [(100., 20.), (40., 134.4), (100., 248.8), (220., 20.)]
        );
    }

    #[test]
    fn should_match_dot_self_loop_clearance_between_peer_nodes() {
        // given
        let mut graph = graph();
        graph.set_node(
            "d",
            NodeLabel {
                width: 80.,
                height: 40.,
                ..Default::default()
            },
        );
        graph.set_edge("a", "a");

        // when
        layout(&mut graph, &mut budget()).unwrap();

        // then
        let centers = ["a", "b", "c", "d"].map(|id| graph.node(id).unwrap().x.unwrap());
        assert_eq!(centers, [40., 185., 305., 425.]);
    }

    #[test]
    fn should_match_dot_node_centers_for_a_diamond() {
        // given
        let mut graph = graph();
        graph.set_node(
            "d",
            NodeLabel {
                width: 80.,
                height: 40.,
                ..Default::default()
            },
        );
        for (from, to) in [("a", "b"), ("a", "c"), ("b", "d"), ("c", "d")] {
            graph.set_edge(from, to);
        }

        // when
        layout(&mut graph, &mut budget()).unwrap();

        // then
        let positions = ["a", "b", "c", "d"].map(|id| {
            let node = graph.node(id).unwrap();
            (node.x.unwrap(), node.y.unwrap())
        });
        assert_eq!(
            positions,
            [(100., 20.), (40., 134.4), (160., 134.4), (100., 248.8)]
        );
    }

    #[test]
    fn should_match_dot_horizontal_label_rank_spacing() {
        // given
        let mut graph = graph();
        graph.graph_mut().rankdir = RankDir::LR;
        graph
            .try_set_edge_named(
                "a",
                "b",
                Some("wide".to_owned()),
                Some(EdgeLabel {
                    height: 30.,
                    ..Default::default()
                }),
            )
            .unwrap();

        // when
        layout(&mut graph, &mut budget()).unwrap();

        // then
        let left = graph.node("a").unwrap().x.unwrap();
        let right = graph.node("b").unwrap().x.unwrap();
        assert!((right - left - 158.).abs() < 0.01);
    }

    #[test]
    fn should_match_dot_separate_label_heights_on_adjacent_ranks() {
        // given
        let mut graph = graph();
        graph.set_edge("a", "b");
        graph
            .try_set_edge_named(
                "b",
                "c",
                Some("tall".to_owned()),
                Some(EdgeLabel {
                    height: 30.,
                    ..Default::default()
                }),
            )
            .unwrap();

        // when
        layout(&mut graph, &mut budget()).unwrap();

        // then
        let y = ["a", "b", "c"].map(|id| graph.node(id).unwrap().y.unwrap());
        assert!((y[1] - y[0] - 114.4).abs() < 0.01);
        assert!((y[2] - y[1] - 136.).abs() < 0.01);
    }

    #[test]
    fn should_match_dot_alignment_when_a_chain_has_an_isolated_peer() {
        // given
        let mut graph = graph();
        graph.set_node(
            "d",
            NodeLabel {
                width: 80.,
                height: 40.,
                ..Default::default()
            },
        );
        graph.set_edge("a", "b");
        graph.set_edge("b", "c");

        // when
        layout(&mut graph, &mut budget()).unwrap();

        // then
        let positions = ["a", "b", "c", "d"].map(|id| graph.node(id).unwrap().x.unwrap());
        assert_eq!(positions, [40., 40., 40., 160.]);
    }

    #[test]
    fn should_match_dot_label_rank_spacing_for_a_chain() {
        // given
        let mut graph = graph();
        graph.set_edge("a", "b");
        graph.set_edge("b", "c");

        // when
        layout(&mut graph, &mut budget()).unwrap();

        // then
        let top = graph.node("a").unwrap().y.unwrap();
        let middle = graph.node("b").unwrap().y.unwrap();
        let bottom = graph.node("c").unwrap().y.unwrap();
        assert!((middle - top - 114.4).abs() < 0.01);
        assert!((bottom - middle - 114.4).abs() < 0.01);
    }

    #[test]
    fn should_respect_raw_dot_minlen_and_measured_label_height() {
        // given
        let mut graph = graph();
        graph
            .try_set_edge_named(
                "a",
                "b",
                Some("labeled".to_owned()),
                Some(EdgeLabel {
                    height: 14.4,
                    minlen: 2,
                    extras: [("dot_label_present".to_owned(), true.into())].into(),
                    ..Default::default()
                }),
            )
            .unwrap();

        // when
        layout_with_control(&mut graph, &mut budget(), LayoutFlavor::RawDot).unwrap();

        // then
        let separation = graph.node("b").unwrap().y.unwrap() - graph.node("a").unwrap().y.unwrap();
        assert!((separation - 176.4).abs() < 0.01);
    }

    #[test]
    fn should_ignore_raw_dot_edges_without_rank_constraint() {
        // given
        let mut graph = graph();
        graph
            .try_set_edge_named(
                "a",
                "b",
                Some("flat".to_owned()),
                Some(EdgeLabel {
                    extras: [("constraint".to_owned(), false.into())].into(),
                    ..Default::default()
                }),
            )
            .unwrap();

        // when
        layout_with_control(&mut graph, &mut budget(), LayoutFlavor::RawDot).unwrap();

        // then
        assert_eq!(graph.node("a").unwrap().y, graph.node("b").unwrap().y);
    }

    #[test]
    fn should_match_graphviz_minimum_fixed_node_size() {
        // given
        let mut graph = graph();
        for id in ["b", "c"] {
            graph.remove_node(id);
        }
        let node = graph.node_mut("a").unwrap();
        node.width = 0.;
        node.height = 0.;

        // when
        layout_with_control(&mut graph, &mut budget(), LayoutFlavor::RawDot).unwrap();

        // then
        let node = graph.node("a").unwrap();
        assert_eq!((node.width, node.height), (0.72, 1.44));
        assert_eq!((node.x, node.y), (Some(0.36), Some(0.72)));
    }

    fn raw_dot_chain_fixture() -> LayoutGraph {
        // tools/dot-parity/fixtures/chain.dot, Graphviz 16.1.0 -Tjson0
        let mut graph = graph();
        graph.set_node(
            "d",
            NodeLabel {
                width: 80.,
                height: 40.,
                ..Default::default()
            },
        );
        for (from, to) in [("a", "b"), ("b", "c")] {
            graph
                .try_set_edge_named(
                    from,
                    to,
                    None::<String>,
                    Some(EdgeLabel {
                        width: 7.202,
                        height: 14.4,
                        weight: 1.,
                        extras: [("dot_label_present".to_owned(), true.into())].into(),
                        ..Default::default()
                    }),
                )
                .unwrap();
        }
        graph
    }

    fn graphviz_five_significant(value: f64) -> f64 {
        if value == 0. {
            return 0.;
        }
        let decimals = 4 - value.abs().log10().floor() as i32;
        let scale = 10_f64.powi(decimals);
        (value * scale).round() / scale
    }

    #[test]
    fn should_match_graphviz_chain_polyline_control_points_exactly() {
        // given
        let mut graph = raw_dot_chain_fixture();

        // when
        layout_with_control(&mut graph, &mut budget(), LayoutFlavor::RawDot).unwrap();

        // then: Graphviz 16.1.0 -Tjson0 serializes bottom-up coordinates with %.5g.
        for (from, to, expected, label_y) in [
            ("a", "b", [228.48, 211.35, 186.04, 166.13, 154.85], 191.6),
            ("b", "c", [114.08, 96.948, 71.635, 51.729, 40.446], 77.2),
        ] {
            let edge = graph.edge(from, to, None).unwrap();
            assert_eq!(edge.points.len(), expected.len());
            for (point, oracle_y) in edge.points.iter().zip(expected) {
                assert_eq!(graphviz_five_significant(point.x), 40.);
                assert_eq!(
                    graphviz_five_significant(graph.graph().height - point.y),
                    oracle_y
                );
            }
            assert_eq!(graphviz_five_significant(edge.x.unwrap()), 43.601);
            assert_eq!(
                graphviz_five_significant(graph.graph().height - edge.y.unwrap()),
                label_y
            );
        }
    }

    #[test]
    fn should_match_graphviz_chain_edge_without_arrow_exactly() {
        // given: chain.dot a->b with arrowhead=none, Graphviz 16.1.0 -Tjson0
        let mut graph = raw_dot_chain_fixture();
        graph
            .edge_mut("a", "b", None)
            .unwrap()
            .extras
            .insert("dot_arrow_present".to_owned(), false.into());

        // when
        layout_with_control(&mut graph, &mut budget(), LayoutFlavor::RawDot).unwrap();

        // then
        let points = &graph.edge("a", "b", None).unwrap().points;
        assert_eq!(points.len(), 4);
        for (point, oracle_y) in points.iter().zip([228.48, 207.89, 175.48, 154.85]) {
            assert_eq!(graphviz_five_significant(point.x), 40.);
            assert_eq!(
                graphviz_five_significant(graph.graph().height - point.y),
                oracle_y
            );
        }
    }

    #[test]
    fn should_match_graphviz_parallel_middle_curve_controls_exactly() {
        // given: tools/dot-parity/fixtures/parallel.dot, Graphviz 16.1.0 -Tjson0
        let mut graph = graph();
        for id in ["e0", "e1", "e2"] {
            graph
                .try_set_edge_named(
                    "a",
                    "b",
                    Some(id.to_owned()),
                    Some(EdgeLabel {
                        width: 7.201171875,
                        height: 14.4,
                        weight: 1.,
                        extras: [("dot_label_present".to_owned(), true.into())].into(),
                        ..Default::default()
                    }),
                )
                .unwrap();
        }

        // when
        layout_with_control(&mut graph, &mut budget(), LayoutFlavor::RawDot).unwrap();

        // then: the label rank's left corridor sets the graph's x origin.
        let middle = graph.edge("a", "b", Some("e1")).unwrap();
        assert_eq!(graphviz_five_significant(graph.graph().width), 209.2);
        assert_eq!(
            graphviz_five_significant(graph.node("a").unwrap().x.unwrap()),
            49.201
        );
        assert_eq!(
            graphviz_five_significant(graph.node("c").unwrap().x.unwrap()),
            169.2
        );
        assert_eq!(middle.points.len(), 5);
        for (point, oracle_y) in middle
            .points
            .iter()
            .zip([114.08, 96.948, 71.635, 51.729, 40.446])
        {
            assert_eq!(graphviz_five_significant(point.x), 49.201);
            assert_eq!(
                graphviz_five_significant(graph.graph().height - point.y),
                oracle_y
            );
        }
        assert_eq!(graphviz_five_significant(middle.x.unwrap()), 52.802);
        assert_eq!(
            graphviz_five_significant(graph.graph().height - middle.y.unwrap()),
            77.2
        );
        for (id, expected, label_x) in [
            (
                "e0",
                [
                    (29.756, 114.03),
                    (16.003, 100.34),
                    (0., 84.4),
                    (0., 84.4),
                    (0., 84.4),
                    (0., 70.),
                    (0., 70.),
                    (0., 70.),
                    (10.564, 59.479),
                    (21.903, 48.187),
                    (29.756, 40.366),
                ],
                3.6006,
            ),
            (
                "e2",
                [
                    (52.363, 114.03),
                    (54.599, 100.34),
                    (57.201, 84.4),
                    (57.201, 84.4),
                    (57.201, 84.4),
                    (57.201, 70.),
                    (57.201, 70.),
                    (57.201, 70.),
                    (55.798, 61.405),
                    (54.167, 51.414),
                    (52.363, 40.366),
                ],
                60.802,
            ),
        ] {
            let edge = graph.edge("a", "b", Some(id)).unwrap();
            assert_eq!(edge.points.len(), expected.len(), "{id}");
            for (point, (x, y)) in edge.points.iter().zip(expected) {
                assert_eq!(graphviz_five_significant(point.x), x, "{id} x");
                assert_eq!(
                    graphviz_five_significant(graph.graph().height - point.y),
                    y,
                    "{id} y"
                );
            }
            assert_eq!(
                graphviz_five_significant(edge.x.unwrap()),
                label_x,
                "{id} lp"
            );
        }
    }

    #[test]
    fn should_align_lr_record_ports_and_route_from_field_boundaries() {
        // given: tools/dot-parity/fixtures/ports.dot, Graphviz 16.1.0 -Tjson0
        let mut graph = Graph::new(GraphOptions {
            directed: true,
            multigraph: true,
            compound: true,
        });
        graph.set_graph(GraphLabel {
            rankdir: RankDir::LR,
            ranksep: 36.,
            nodesep: 18.,
            ..Default::default()
        });
        graph.set_node(
            "a",
            NodeLabel {
                width: 54.,
                height: 50.6,
                ..Default::default()
            },
        );
        graph.set_node(
            "b",
            NodeLabel {
                width: 55.67,
                height: 50.6,
                ..Default::default()
            },
        );
        graph.try_set_edge_named("a", "b", Some("e0".to_owned()), Some(EdgeLabel {
            width: 22.55,
            height: 16.8,
            weight: 1.,
            extras: [
                ("dot_label_present".to_owned(), true.into()),
                ("dot_tail_port".to_owned(), "right:e".into()),
                ("dot_head_port".to_owned(), "top:w".into()),
                ("dot_tail_record_fields".to_owned(), serde_json::json!([{"port":"right","x":0.,"y":12.4,"width":54.,"height":24.8}])),
                ("dot_head_record_fields".to_owned(), serde_json::json!([{"port":"top","x":0.,"y":-12.4,"width":55.67,"height":24.8}])),
            ].into(),
            ..Default::default()
        })).unwrap();

        // when
        layout_with_control(&mut graph, &mut budget(), LayoutFlavor::RawDot).unwrap();

        // then
        assert_eq!(graphviz_five_significant(graph.graph().height), 74.6);
        assert_eq!(
            graphviz_five_significant(graph.node("a").unwrap().y.unwrap()),
            25.3
        );
        assert_eq!(
            graphviz_five_significant(graph.node("b").unwrap().y.unwrap()),
            49.3
        );
        let edge = graph.edge("a", "b", Some("e0")).unwrap();
        assert_eq!(edge.points.len(), 5);
        assert_eq!(edge.points[0].x, 54.);
        assert_eq!(edge.points[0].y, 37.7);
        assert_eq!(graphviz_five_significant(edge.points[4].x), 112.55);
        assert_eq!(edge.points[4].y, 36.9);
        assert_eq!(graphviz_five_significant(edge.x.unwrap()), 83.275);
        assert_eq!(
            graphviz_five_significant(graph.graph().height - edge.y.unwrap()),
            45.841
        );
    }

    #[test]
    fn should_match_graphviz_cluster_crossing_spline_exactly() {
        // given: subgraph_edge.dot a->d, Graphviz 16.1.0 -Tjson0
        let source = Point { x: 43., y: 58.8 };
        let target = Point { x: 115., y: 143.6 };
        let waypoint = Point { x: 79., y: 92.8 };
        let size = Size {
            width: 54.,
            height: 36.,
        };

        // when
        let points = route_cluster_crossing(
            (source, target),
            waypoint,
            (size, size),
            (ClipShape::Ellipse(1.), ClipShape::Ellipse(1.)),
            ArrowMode::Head,
        );

        // then
        let expected = [
            (59.44, 104.),
            (65.825, 98.324),
            (73.024, 91.513),
            (79., 84.8),
            (85.758, 77.209),
            (92.471, 68.371),
            (98.238, 60.266),
            (104.74, 50.876),
        ];
        assert_eq!(points.len(), expected.len());
        for (point, (x, y)) in points.iter().zip(expected) {
            assert_eq!(graphviz_five_significant(point.x), x);
            assert_eq!(graphviz_five_significant(177.6 - point.y), y);
        }
    }

    #[test]
    fn should_match_graphviz_undirected_feedback_route_exactly() {
        // given: undirected.dot c--a, Graphviz 16.1.0 -Tjson0
        let source = Point { x: 54., y: 162. };
        let target = Point { x: 54., y: 18. };
        let size = Size {
            width: 54.,
            height: 36.,
        };

        // when
        let points = route_parallel_polyline(
            (source, target),
            (size, size),
            63.,
            36.,
            ArrowMode::None,
            (ClipShape::Ellipse(1.), ClipShape::Ellipse(1.)),
        );

        // then
        let expected = [
            (56.947, 36.354),
            (59.572, 51.811),
            (63., 72.),
            (63., 72.),
            (63., 72.),
            (63., 108.),
            (63., 108.),
            (63., 108.),
            (59.572, 128.19),
            (56.947, 143.65),
        ];
        assert_eq!(points.len(), expected.len());
        for (point, (x, y)) in points.iter().zip(expected) {
            assert_eq!(graphviz_five_significant(point.x), x);
            assert_eq!(graphviz_five_significant(180. - point.y), y);
        }
    }

    #[test]
    fn should_match_graphviz_diamond_polyline_controls_exactly() {
        // given: tools/dot-parity/fixtures/diamond.dot, Graphviz 16.1.0 -Tjson0
        let mut graph = graph();
        graph.set_node(
            "d",
            NodeLabel {
                width: 80.,
                height: 40.,
                ..Default::default()
            },
        );
        for (from, to) in [("a", "b"), ("a", "c"), ("b", "d"), ("c", "d")] {
            graph
                .try_set_edge_named(
                    from,
                    to,
                    None::<String>,
                    Some(EdgeLabel {
                        width: 7.202,
                        height: 14.4,
                        weight: 1.,
                        extras: [("dot_label_present".to_owned(), true.into())].into(),
                        ..Default::default()
                    }),
                )
                .unwrap();
        }

        // when
        layout_with_control(&mut graph, &mut budget(), LayoutFlavor::RawDot).unwrap();

        // then: each point is (x, bottom-up y) as serialized by Graphviz JSON0.
        for (from, to, expected) in [
            (
                "a",
                "b",
                [
                    (89.686, 228.48),
                    (80.327, 210.95),
                    (66.393, 184.84),
                    (55.66, 164.74),
                    (50.381, 154.85),
                ],
            ),
            (
                "a",
                "c",
                [
                    (110.31, 228.48),
                    (119.67, 210.95),
                    (133.61, 184.84),
                    (144.34, 164.74),
                    (149.62, 154.85),
                ],
            ),
            (
                "b",
                "d",
                [
                    (50.314, 114.08),
                    (59.673, 96.546),
                    (73.607, 70.442),
                    (84.34, 50.336),
                    (89.619, 40.446),
                ],
            ),
            (
                "c",
                "d",
                [
                    (149.69, 114.08),
                    (140.33, 96.546),
                    (126.39, 70.442),
                    (115.66, 50.336),
                    (110.38, 40.446),
                ],
            ),
        ] {
            let points = &graph.edge(from, to, None).unwrap().points;
            assert_eq!(points.len(), expected.len(), "{from}->{to}");
            for (point, (oracle_x, oracle_y)) in points.iter().zip(expected) {
                assert_eq!(
                    graphviz_five_significant(point.x),
                    oracle_x,
                    "{from}->{to} x"
                );
                assert_eq!(
                    graphviz_five_significant(graph.graph().height - point.y),
                    oracle_y,
                    "{from}->{to} y"
                );
            }
        }
        for (from, to, oracle_x, oracle_y) in [
            ("a", "b", 77.193, 191.6),
            ("a", "c", 137.19, 191.6),
            ("b", "d", 77.193, 77.2),
            ("c", "d", 137.19, 77.2),
        ] {
            let edge = graph.edge(from, to, None).unwrap();
            assert_eq!(graphviz_five_significant(edge.x.unwrap()), oracle_x);
            assert_eq!(
                graphviz_five_significant(graph.graph().height - edge.y.unwrap()),
                oracle_y
            );
        }
    }

    #[test]
    fn should_match_graphviz_cycle_back_edge_exactly() {
        // given: tools/dot-parity/fixtures/cycle.dot, Graphviz 16.1.0 -Tjson0
        let mut graph = graph();
        graph.set_node(
            "d",
            NodeLabel {
                width: 80.,
                height: 40.,
                ..Default::default()
            },
        );
        for (from, to) in [("a", "b"), ("b", "c"), ("c", "a")] {
            graph
                .try_set_edge_named(
                    from,
                    to,
                    None::<String>,
                    Some(EdgeLabel {
                        width: 7.202,
                        height: 14.4,
                        weight: 1.,
                        extras: [("dot_label_present".to_owned(), true.into())].into(),
                        ..Default::default()
                    }),
                )
                .unwrap();
        }

        // when
        layout_with_control(&mut graph, &mut budget(), LayoutFlavor::RawDot).unwrap();

        // then
        let edge = graph.edge("c", "a", None).unwrap();
        assert_eq!(edge.points.len(), 5);
        for (point, oracle_y) in edge
            .points
            .iter()
            .zip([40.288, 79.664, 169.58, 217.08, 228.46])
        {
            assert_eq!(graphviz_five_significant(point.x), 100.);
            assert_eq!(
                graphviz_five_significant(graph.graph().height - point.y),
                oracle_y
            );
        }
        assert_eq!(graphviz_five_significant(edge.x.unwrap()), 103.6);
        assert_eq!(
            graphviz_five_significant(graph.graph().height - edge.y.unwrap()),
            134.4
        );
    }

    #[test]
    fn should_match_graphviz_lr_label_side_exactly() {
        // given: tools/dot-parity/fixtures/rankdir_lr.dot, Graphviz 16.1.0 -Tjson0
        let mut graph = graph();
        graph.graph_mut().rankdir = RankDir::LR;
        for (from, to) in [("a", "b"), ("b", "c")] {
            graph
                .try_set_edge_named(
                    from,
                    to,
                    None::<String>,
                    Some(EdgeLabel {
                        width: 7.202,
                        height: 14.4,
                        weight: 1.,
                        extras: [("dot_label_present".to_owned(), true.into())].into(),
                        ..Default::default()
                    }),
                )
                .unwrap();
        }

        // when
        layout_with_control(&mut graph, &mut budget(), LayoutFlavor::RawDot).unwrap();

        // then
        for (from, to, label_x) in [("a", "b", 113.6), ("b", "c", 260.8)] {
            let edge = graph.edge(from, to, None).unwrap();
            assert_eq!(edge.points.len(), 5);
            assert_eq!(graphviz_five_significant(edge.x.unwrap()), label_x);
            assert_eq!(
                graphviz_five_significant(graph.graph().height - edge.y.unwrap()),
                27.2
            );
        }
    }

    #[test]
    fn should_match_graphviz_self_loop_controls_and_label_exactly() {
        // given: tools/dot-parity/fixtures/self.dot, Graphviz 16.1.0 -Tjson0
        let mut graph = graph();
        graph.set_node(
            "d",
            NodeLabel {
                width: 80.,
                height: 40.,
                ..Default::default()
            },
        );
        graph
            .try_set_edge_named(
                "a",
                "a",
                None::<String>,
                Some(EdgeLabel {
                    width: 7.202,
                    height: 14.4,
                    weight: 1.,
                    extras: [("dot_label_present".to_owned(), true.into())].into(),
                    ..Default::default()
                }),
            )
            .unwrap();

        // when
        layout_with_control(&mut graph, &mut budget(), LayoutFlavor::RawDot).unwrap();

        // then: seven cubic controls and a separate head-arrow tip
        let edge = graph.edge("a", "a", None).unwrap();
        let expected = [
            (80.339, 27.236),
            (101.2, 28.35),
            (120., 25.938),
            (120., 20.),
            (120., 15.176),
            (107.59, 12.679),
            (91.745, 12.509),
            (80.339, 12.764),
        ];
        assert_eq!(edge.points.len(), expected.len());
        for (point, (oracle_x, oracle_y)) in edge.points.iter().zip(expected) {
            assert_eq!(graphviz_five_significant(point.x), oracle_x);
            assert_eq!(
                graphviz_five_significant(graph.graph().height - point.y),
                oracle_y
            );
        }
        assert_eq!(graphviz_five_significant(edge.x.unwrap()), 123.6);
        assert_eq!(
            graphviz_five_significant(graph.graph().height - edge.y.unwrap()),
            20.
        );
    }

    #[test]
    fn should_match_graphviz_ranked_self_loop_controls_with_label_rank() {
        // given: Graphviz 16.1.0, a -> b; a -> a [label="self"]
        let route = SelfRoute {
            center: Point { x: 27., y: 18. },
            size: Size {
                width: 54.,
                height: 36.,
            },
            nodesep: 18.,
            rank_distance: 73. / 2.,
            shape: ClipShape::Box(1.),
            arrow: ArrowMode::Head,
        };

        // when
        let points = route_self_right(route);

        // then: two clipped cubics and a separate head-arrow tip.
        let expected = [
            (54.241, 97.837),
            (64.024, 97.984),
            (72., 95.705),
            (72., 91.),
            (72., 88.353),
            (69.476, 86.474),
            (65.527, 85.363),
            (54.241, 84.163),
        ];
        assert_eq!(points.len(), expected.len());
        for (point, (oracle_x, oracle_y)) in points.iter().zip(expected) {
            assert_eq!(graphviz_five_significant(point.x), oracle_x);
            assert_eq!(graphviz_five_significant(109. - point.y), oracle_y);
        }
    }

    #[test]
    fn should_match_graphviz_tail_and_double_arrow_straight_controls() {
        // given: Graphviz 16.1.0 fixed 1 x 0.5 inch boxes, dir=back/both
        let route = |arrow, tail_arrow| StraightRoute {
            start: Point { x: 36., y: 18. },
            end: Point { x: 36., y: 90. },
            source: Size {
                width: 72.,
                height: 36.,
            },
            target: Size {
                width: 72.,
                height: 36.,
            },
            source_shape: ClipShape::Box(1.),
            target_shape: ClipShape::Box(1.),
            arrow,
            tail_arrow,
            head_arrow_length: normal_arrow_stem_length(),
            tail_arrow_length: normal_arrow_stem_length(),
            rankdir: RankDir::TB,
            flow: RouteFlow::Forward,
        };

        // when
        let back = route_straight_cubic(route(ArrowMode::None, true));
        let both = route_straight_cubic(route(ArrowMode::Head, true));

        // then: tip-first/tip-last point convention and exact clipped controls.
        for (points, expected) in [
            (back, vec![71.697, 60.24, 52.041, 43.369, 36.104]),
            (both, vec![71.697, 60.24, 56.012, 51.659, 47.438, 36.104]),
        ] {
            assert_eq!(points.len(), expected.len());
            for (point, oracle_y) in points.iter().zip(expected) {
                assert_eq!(graphviz_five_significant(point.x), 36.);
                assert_eq!(graphviz_five_significant(108. - point.y), oracle_y);
            }
        }
    }

    #[test]
    fn should_place_horizontal_chain_from_left_to_right() {
        // given
        let mut graph = graph();
        graph.graph_mut().rankdir = RankDir::LR;
        graph.set_edge("a", "b");
        graph.set_edge("b", "c");

        // when
        layout(&mut graph, &mut budget()).unwrap();

        // then
        assert!(graph.node("a").unwrap().x.unwrap() < graph.node("b").unwrap().x.unwrap());
        assert!(graph.node("b").unwrap().x.unwrap() < graph.node("c").unwrap().x.unwrap());
    }

    #[test]
    fn should_stop_layout_when_work_budget_is_exhausted() {
        // given
        let mut graph = graph();
        graph.set_edge("a", "b");
        let mut options = RenderOptions {
            graph_layout: GraphLayout::Layered,
            ..Default::default()
        };
        options.limits.layout_work = 1;
        let mut budget = Budget::new(&options);

        // when
        let result = layout(&mut graph, &mut budget);

        // then
        assert!(
            matches!(result, Err(RenderError::Backend(message)) if message.contains("work units"))
        );
    }

    #[test]
    fn should_place_cycle_and_self_edge_without_missing_geometry() {
        // given
        let mut graph = graph();
        graph.set_edge("a", "b");
        graph.set_edge("b", "a");
        graph.set_edge("c", "c");

        // when
        layout(&mut graph, &mut budget()).unwrap();

        // then
        assert!(
            graph
                .nodes()
                .all(|id| graph.node(id).unwrap().x.unwrap().is_finite())
        );
        assert!(
            graph
                .edges()
                .all(|key| graph.edge_by_key(key).unwrap().points.len() >= 2)
        );
        assert_eq!(graph.edge("c", "c", None).unwrap().points.len(), 4);
    }
}
