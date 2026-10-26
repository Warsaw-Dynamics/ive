use super::budget::Budget;
use crate::{DotRunner, RenderError};
use dugong::{EdgeLabel, GraphLabel, NodeLabel, Point, RankDir, graphlib::Graph};
use serde_json::Value;

type LayoutGraph = Graph<NodeLabel, EdgeLabel, GraphLabel>;

pub(super) fn layout(
    graph: &mut LayoutGraph,
    runner: &dyn DotRunner,
    budget: &mut Budget,
) -> Result<(), RenderError> {
    let source = write_source(graph, budget)?;
    let output = runner.run(&source).map_err(|error| {
        RenderError::Backend(format!("PlantUML DOT layout process failed: {error}"))
    })?;
    if output.len() > 8 * 1024 * 1024 {
        return Err(RenderError::Backend(format!(
            "PlantUML DOT output has {} bytes, exceeding 8 MiB",
            output.len()
        )));
    }
    budget.charge(output.len() / 32 + 1)?;
    let document: Value = serde_json::from_str(&output)
        .map_err(|error| RenderError::Backend(format!("PlantUML DOT JSON is invalid: {error}")))?;
    apply_layout(graph, &document, budget)
}

fn write_source(graph: &LayoutGraph, budget: &mut Budget) -> Result<String, RenderError> {
    let mut source = format!(
        "digraph G {{ graph [rankdir={},nodesep={},ranksep={},margin=0,splines=polyline]; node [shape=box,label=\"\",fixedsize=true];\n",
        match graph.graph().rankdir {
            RankDir::LR => "LR",
            RankDir::RL => "RL",
            RankDir::BT => "BT",
            RankDir::TB => "TB",
        },
        graph.graph().nodesep / 72.,
        graph.graph().ranksep / 72.,
    );
    for id in graph.children_root() {
        write_node(graph, id, 0, &mut source, budget)?;
    }
    for (index, edge) in graph.edges().enumerate() {
        budget.charge(1)?;
        let label = graph
            .edge_by_key(edge)
            .ok_or_else(|| backend("DOT input edge disappeared"))?;
        let columns = (label.width / 7.).ceil().clamp(1., 80.) as usize;
        let text = " ".repeat(columns);
        source.push_str(&format!(
            "\"{}\" -> \"{}\" [id=\"edge{index}\",label=\"{text}\",fontname=\"Courier\",fontsize={}];\n",
            edge.v, edge.w, label.height.clamp(12., 80.)
        ));
    }
    source.push('}');
    if source.len() > 2 * 1024 * 1024 {
        return Err(RenderError::Backend(format!(
            "PlantUML DOT input has {} bytes, exceeding 2 MiB",
            source.len()
        )));
    }
    Ok(source)
}

fn write_node(
    graph: &LayoutGraph,
    id: &str,
    depth: usize,
    source: &mut String,
    budget: &mut Budget,
) -> Result<(), RenderError> {
    budget.depth(depth)?;
    budget.charge(1)?;
    if !id
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    {
        return Err(RenderError::Backend(format!(
            "PlantUML DOT node ID {id:?} is invalid"
        )));
    }
    let node = graph
        .node(id)
        .ok_or_else(|| RenderError::Backend(format!("PlantUML DOT node {id:?} is missing")))?;
    let children = graph.children(id);
    if !children.is_empty() {
        source.push_str(&format!(
            "subgraph \"cluster_{id}\" {{ graph [label=\"\",margin=12];\n"
        ));
    }
    source.push_str(&format!(
        "\"{id}\" [width={:.6},height={:.6}];\n",
        (node.width / 72.).max(0.01),
        (node.height / 72.).max(0.01)
    ));
    for child in children {
        write_node(graph, child, depth + 1, source, budget)?;
    }
    if graph.children(id).is_empty() {
        return Ok(());
    }
    source.push_str("}\n");
    Ok(())
}

fn apply_layout(
    graph: &mut LayoutGraph,
    document: &Value,
    budget: &mut Budget,
) -> Result<(), RenderError> {
    let bbox = rectangle(document, "bb")?;
    let objects = document["objects"]
        .as_array()
        .ok_or_else(|| backend("DOT JSON has no objects array"))?;
    let mut clusters = Vec::new();
    for object in objects {
        budget.charge(1)?;
        let name = field(object, "name")?;
        if let Some(id) = name.strip_prefix("cluster_") {
            let bounds = rectangle(object, "bb")?;
            clusters.push((id.to_owned(), bounds));
            continue;
        }
        if let Some(node) = graph.node_mut(name) {
            let point = point(field(object, "pos")?)?;
            node.x = Some(point.0 - bbox.0);
            node.y = Some(bbox.3 - point.1);
        }
    }
    for (id, bounds) in clusters {
        let node = graph
            .node_mut(&id)
            .ok_or_else(|| backend("DOT cluster has no owner"))?;
        node.x = Some((bounds.0 + bounds.2) / 2. - bbox.0);
        node.y = Some(bbox.3 - (bounds.1 + bounds.3) / 2.);
        node.width = bounds.2 - bounds.0;
        node.height = bounds.3 - bounds.1;
    }
    let keys = graph.edges().cloned().collect::<Vec<_>>();
    let edges = document["edges"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    if edges.len() != keys.len() {
        return Err(backend("DOT JSON edge count differs from submitted graph"));
    }
    for edge in edges {
        budget.charge(1)?;
        let index = field(edge, "id")?
            .strip_prefix("edge")
            .ok_or_else(|| backend("DOT edge ID is invalid"))?
            .parse::<usize>()
            .map_err(|_| backend("DOT edge ID is invalid"))?;
        let key = keys
            .get(index)
            .ok_or_else(|| backend("DOT edge ID is out of range"))?;
        let route = route(field(edge, "pos")?, bbox)?;
        let placed = graph
            .edge_mut_by_key(key)
            .ok_or_else(|| backend("DOT edge disappeared"))?;
        let label = route_midpoint(&route);
        placed.points = route;
        placed.x = Some(label.x);
        placed.y = Some(label.y);
    }
    if graph.edges().any(|key| {
        graph
            .edge_by_key(key)
            .is_some_and(|edge| edge.points.len() < 2)
    }) {
        return Err(backend("DOT omitted an edge route"));
    }
    if graph.nodes().any(|id| {
        graph
            .node(id)
            .is_some_and(|node| node.x.is_none() || node.y.is_none())
    }) {
        return Err(backend("DOT omitted a node position"));
    }
    Ok(())
}

fn route(value: &str, bbox: (f64, f64, f64, f64)) -> Result<Vec<Point>, RenderError> {
    let mut endpoint = None;
    let mut points = Vec::new();
    for token in value.split_whitespace() {
        if let Some(value) = token.strip_prefix("e,") {
            endpoint = Some(point(value)?);
        } else if let Some(value) = token.strip_prefix("s,") {
            points.push(point(value)?);
        } else {
            points.push(point(token)?);
        }
    }
    if let Some(endpoint) = endpoint {
        points.push(endpoint);
    }
    if points.len() < 2 {
        return Err(backend("DOT edge route has fewer than two points"));
    }
    Ok(points
        .into_iter()
        .map(|(x, y)| Point {
            x: x - bbox.0,
            y: bbox.3 - y,
        })
        .collect())
}

fn route_midpoint(points: &[Point]) -> Point {
    let total = points
        .windows(2)
        .map(|pair| distance(pair[0], pair[1]))
        .sum::<f64>();
    let mut left = total / 2.;
    for pair in points.windows(2) {
        let length = distance(pair[0], pair[1]);
        if left <= length && length > 0. {
            let part = left / length;
            return Point {
                x: pair[0].x + (pair[1].x - pair[0].x) * part,
                y: pair[0].y + (pair[1].y - pair[0].y) * part,
            };
        }
        left -= length;
    }
    points[points.len() / 2]
}

fn distance(start: Point, end: Point) -> f64 {
    (end.x - start.x).hypot(end.y - start.y)
}

fn rectangle(value: &Value, key: &str) -> Result<(f64, f64, f64, f64), RenderError> {
    let text = field(value, key)?;
    let parts = text.split(',').map(number).collect::<Result<Vec<_>, _>>()?;
    if parts.len() != 4 || parts[2] <= parts[0] || parts[3] <= parts[1] {
        return Err(backend("DOT bounding box is invalid"));
    }
    Ok((parts[0], parts[1], parts[2], parts[3]))
}

fn point(value: &str) -> Result<(f64, f64), RenderError> {
    let parts = value
        .split(',')
        .map(number)
        .collect::<Result<Vec<_>, _>>()?;
    if parts.len() != 2 {
        return Err(backend("DOT point is invalid"));
    }
    Ok((parts[0], parts[1]))
}

fn number(value: &str) -> Result<f64, RenderError> {
    let parsed = value
        .parse::<f64>()
        .map_err(|_| backend("DOT coordinate is invalid"))?;
    if parsed.is_finite() && parsed.abs() <= 1_000_000. {
        return Ok(parsed);
    }
    Err(backend("DOT coordinate is nonfinite or exceeds 1000000"))
}

fn field<'a>(value: &'a Value, name: &str) -> Result<&'a str, RenderError> {
    value[name]
        .as_str()
        .ok_or_else(|| backend("DOT JSON field is missing"))
}

fn backend(message: &str) -> RenderError {
    RenderError::Backend(format!("PlantUML DOT layout: {message}"))
}
