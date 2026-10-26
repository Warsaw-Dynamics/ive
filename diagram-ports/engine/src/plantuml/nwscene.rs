//! Original sparse measured buses/columns with MIT nwdiag membership/group semantics.
//! Copyright 2009-2024 Arnaud Roques; exact source traces/notices in provenance.
use super::{
    budget::{Budget, invalid},
    nwdiag::{Fill, GroupAssignment, NwDiagram, Width},
    scene::{Dash, Decoration, Element, Paint, Point, Rect, Scene, Shape, text_at},
    text::{Context, Label},
};
use crate::RenderError;
use std::collections::BTreeSet;

struct Column {
    label: Label,
    rect: Rect,
    left: f64,
    lane_count: usize,
    width: f64,
}
struct Bus {
    label: Label,
    address: Label,
    y: f64,
    start: f64,
    end: f64,
    members: Vec<usize>,
}
struct Cable {
    address: Label,
    x: f64,
    lane: usize,
}
struct Enclosure {
    label: Label,
    rect: Rect,
}
struct Layout {
    columns: Vec<Column>,
    buses: Vec<Bus>,
    cables: Vec<Cable>,
    groups: Vec<Enclosure>,
    order: Vec<usize>,
    label_right: f64,
}

pub(super) fn layout(model: &NwDiagram, context: &mut Context<'_>) -> Result<Scene, RenderError> {
    let measured = measure(model, context)?;
    let mut scene = Scene::new();
    paint_groups(model, &measured, &mut scene, context.budget)?;
    paint_buses(model, &measured, &mut scene, context.budget)?;
    paint_cables(model, &measured, &mut scene, context.budget)?;
    paint_servers(model, &measured, &mut scene, context.budget)?;
    Ok(scene)
}

fn measure(model: &NwDiagram, context: &mut Context<'_>) -> Result<Layout, RenderError> {
    let mut columns = Vec::new();
    for server in &model.servers {
        context.budget.item()?;
        let label = server_label(&server.display, context)?;
        let width = label.width.max(64.) + 24.;
        columns.push(Column {
            rect: Rect {
                x: 0.,
                y: 0.,
                width,
                height: label.height.max(24.) + 24.,
            },
            label,
            left: 0.,
            lane_count: 0,
            width,
        });
    }
    let mut buses = Vec::new();
    for network in &model.networks {
        context.budget.item()?;
        buses.push(Bus {
            label: context.label(&network.display)?,
            address: context.label(&network.address)?,
            y: 0.,
            start: 0.,
            end: 0.,
            members: Vec::new(),
        });
    }
    let mut cables = Vec::new();
    for (index, membership) in model.memberships.iter().enumerate() {
        context.budget.item()?;
        let address = server_label(&membership.address, context)?;
        let column = &mut columns[membership.server.0];
        column.width = column.width.max(address.width + 24.);
        let lane = column.lane_count;
        column.lane_count += 1;
        cables.push(Cable {
            address,
            x: 0.,
            lane,
        });
        buses[membership.network.0].members.push(index);
    }
    let mut groups = Vec::new();
    for group in &model.groups {
        context.budget.item()?;
        let label = context.label(&group.description)?;
        let first = group.members[0].0;
        columns[first].width = columns[first].width.max(label.width + 24.);
        groups.push(Enclosure {
            label,
            rect: Rect {
                x: 0.,
                y: 0.,
                width: 0.,
                height: 0.,
            },
        });
    }
    let order = column_order(model, context.budget)?;
    let mut result = Layout {
        columns,
        buses,
        cables,
        groups,
        order,
        label_right: 0.,
    };
    place(model, &mut result, context.budget)?;
    Ok(result)
}

fn server_label(text: &str, context: &mut Context<'_>) -> Result<Label, RenderError> {
    context.budget.text(text.len())?;
    context.label(&text.replace(", ", "\n"))
}

fn column_order(model: &NwDiagram, budget: &mut Budget) -> Result<Vec<usize>, RenderError> {
    budget.charge(model.groups.len() + model.servers.len())?;
    let mut emitted = vec![false; model.groups.len()];
    let mut order = Vec::new();
    for server in &model.servers {
        budget.charge(1)?;
        match server.group {
            GroupAssignment::Ungrouped => order.push(server.id.0),
            GroupAssignment::Group(group) if emitted[group.0] => {}
            GroupAssignment::Group(group) => {
                emitted[group.0] = true;
                for member in &model.groups[group.0].members {
                    budget.charge(1)?;
                    order.push(member.0);
                }
            }
        }
    }
    Ok(order)
}

fn place(model: &NwDiagram, measured: &mut Layout, budget: &mut Budget) -> Result<(), RenderError> {
    let mut left: f64 = 24.;
    for bus in &measured.buses {
        budget.charge(1)?;
        left = left.max(bus.label.width.max(bus.address.width) + 48.);
    }
    let mut right = left;
    measured.label_right = left - 24.;
    for id in &measured.order {
        budget.charge(1)?;
        let column = &mut measured.columns[*id];
        column.left = right;
        column.rect.x = right + column.lane_count as f64 * 12. + 16.;
        right = column.rect.x + column.width + 36.;
    }
    for (index, membership) in model.memberships.iter().enumerate() {
        budget.charge(1)?;
        measured.cables[index].x = measured.columns[membership.server.0].left
            + 12. * measured.cables[index].lane as f64
            + 8.;
    }
    let mut y = 24.;
    for (id, bus) in measured.buses.iter_mut().enumerate() {
        budget.charge(1)?;
        let label_height = bus.label.height + bus.address.height + 6.;
        bus.y = y + label_height / 2.;
        let mut address_height: f64 = 0.;
        let mut start = right;
        let mut end = left;
        for member in &bus.members {
            budget.charge(1)?;
            let cable = &measured.cables[*member];
            address_height = address_height.max(cable.address.height);
            start = start.min(cable.x - 12.);
            end = end.max(cable.x + 12.);
        }
        (bus.start, bus.end) = match model.networks[id].width {
            Width::Full => (left, right - 24.),
            Width::Members if bus.members.is_empty() => (left, left + 80.),
            Width::Members => (start, end),
        };
        y = (bus.y + 12. + address_height).max(y + label_height) + 32.;
    }
    let mut header_height: f64 = 0.;
    for group in &measured.groups {
        budget.charge(1)?;
        header_height = header_height.max(group.label.height);
    }
    let node_y = y + header_height + 32.;
    for column in &mut measured.columns {
        budget.charge(1)?;
        column.rect.y = node_y;
    }
    for (id, group) in model.groups.iter().enumerate() {
        budget.charge(1)?;
        let first = &measured.columns[group.members[0].0];
        let mut right = first.rect.x;
        let mut bottom = node_y;
        for member in &group.members {
            budget.charge(1)?;
            let column = &measured.columns[member.0];
            right = right.max(column.rect.x + column.width);
            bottom = bottom.max(column.rect.y + column.rect.height);
        }
        let top = node_y - measured.groups[id].label.height - 24.;
        measured.groups[id].rect = Rect {
            x: first.rect.x - 12.,
            y: top,
            width: right - first.rect.x + 24.,
            height: bottom + 12. - top,
        };
    }
    if !right.is_finite() || right > 999_800. || node_y > 999_800. {
        return Err(invalid(
            1,
            format!("excessive nwdiag measured geometry {right} by {node_y}"),
        ));
    }
    Ok(())
}

fn fill(value: Fill, default: Paint) -> Paint {
    match value {
        Fill::Default => default,
        Fill::Color(color) => Paint::Color(color),
    }
}
fn paint_groups(
    model: &NwDiagram,
    measured: &Layout,
    scene: &mut Scene,
    budget: &mut Budget,
) -> Result<(), RenderError> {
    for group in &model.groups {
        budget.charge(8)?;
        budget.text(group.name.len() + 96)?;
        let enclosure = &measured.groups[group.id.0];
        let mut element = Element::new(format!("nwdiag-group-{}", group.id.0), "nwdiag-group");
        element.name = group.name.clone();
        element.shapes.push(Shape::Rect(
            enclosure.rect,
            0.,
            fill(group.fill, Paint::Container),
        ));
        if group.description.is_empty() {
        } else {
            element.shapes.push(Shape::Text(
                Point::new(enclosure.rect.x + 12., enclosure.rect.y + 6.),
                enclosure.label.clone(),
                Decoration::Plain,
            ));
        }
        scene.push(element, budget)?;
        for member in &group.members {
            budget.charge(1)?;
            budget.text(64)?;
            let mut item = Element::new(
                format!("nwdiag-group-member-{}", member.0),
                "nwdiag-group-member",
            );
            item.owner = format!("nwdiag-group-{}", group.id.0);
            item.target = format!("nwdiag-server-{}", member.0);
            scene.push(item, budget)?;
        }
    }
    Ok(())
}

fn paint_buses(
    model: &NwDiagram,
    measured: &Layout,
    scene: &mut Scene,
    budget: &mut Budget,
) -> Result<(), RenderError> {
    // Active earlier cables are the only possible foreign crossings. Ordered
    // sparse sweep avoids a network-by-server matrix; each emitted gap is charged.
    let mut active: BTreeSet<usize> = BTreeSet::new();
    let mut ranks = Vec::new();
    budget.charge(
        measured.cables.len() * (measured.cables.len().checked_ilog2().unwrap_or(0) as usize + 1),
    )?;
    ranks.extend(0..measured.cables.len());
    ranks.sort_by(|a, b| measured.cables[*a].x.total_cmp(&measured.cables[*b].x));
    let mut positions = vec![0; ranks.len()];
    for (rank, index) in ranks.iter().enumerate() {
        budget.charge(1)?;
        positions[*index] = rank;
    }
    for network in &model.networks {
        budget.charge(8)?;
        budget.text(network.name.len() + 64)?;
        let bus = &measured.buses[network.id.0];
        let paint = fill(network.fill, Paint::Ink);
        let mut element =
            Element::new(format!("nwdiag-network-{}", network.id.0), "nwdiag-network");
        element.name = network.name.clone();
        let mut start = bus.start;
        for rank in &active {
            budget.charge(1)?;
            let x = measured.cables[ranks[*rank]].x;
            if x <= bus.start || x >= bus.end {
                continue;
            }
            add_segment(&mut element, (start, x - 5., bus.y), paint, budget)?;
            start = x + 5.;
        }
        add_segment(&mut element, (start, bus.end, bus.y), paint, budget)?;
        budget.charge(2)?;
        let label_x = measured.label_right;
        element.shapes.push(Shape::Text(
            Point::new(
                label_x - bus.label.width,
                bus.y - (bus.label.height + bus.address.height + 6.) / 2.,
            ),
            bus.label.clone(),
            Decoration::Plain,
        ));
        element.shapes.push(Shape::Text(
            Point::new(
                label_x - bus.address.width,
                bus.y + (bus.label.height - bus.address.height + 6.) / 2.,
            ),
            bus.address.clone(),
            Decoration::Plain,
        ));
        scene.push(element, budget)?;
        for index in &bus.members {
            budget.charge(positions.len().checked_ilog2().unwrap_or(0) as usize + 1)?;
            active.insert(positions[*index]);
        }
    }
    Ok(())
}
fn add_segment(
    element: &mut Element,
    segment: (f64, f64, f64),
    paint: Paint,
    budget: &mut Budget,
) -> Result<(), RenderError> {
    let (start, end, y) = segment;
    if end <= start {
        return Ok(());
    }
    budget.charge(4)?;
    element.shapes.push(Shape::Rect(
        Rect {
            x: start,
            y: y - 2.,
            width: end - start,
            height: 4.,
        },
        0.,
        paint,
    ));
    Ok(())
}

fn paint_cables(
    model: &NwDiagram,
    measured: &Layout,
    scene: &mut Scene,
    budget: &mut Budget,
) -> Result<(), RenderError> {
    for (id, membership) in model.memberships.iter().enumerate() {
        budget.charge(24)?;
        budget.text(membership.address.len() + 192)?;
        let cable = &measured.cables[id];
        let bus = &measured.buses[membership.network.0];
        let column = &measured.columns[membership.server.0];
        let center = column.rect.x + column.rect.width / 2.;
        let mut element = Element::new(format!("nwdiag-membership-{id}"), "nwdiag-membership");
        element.owner = format!("nwdiag-network-{}", membership.network.0);
        element.target = format!("nwdiag-server-{}", membership.server.0);
        element.shapes.push(Shape::Line(
            vec![
                Point::new(cable.x, bus.y),
                Point::new(cable.x, column.rect.y - 8.),
                Point::new(center, column.rect.y - 8.),
                Point::new(center, column.rect.y),
            ],
            Dash::Solid,
        ));
        element.shapes.push(Shape::Ellipse(
            Rect {
                x: cable.x - 2.,
                y: bus.y - 2.,
                width: 4.,
                height: 4.,
            },
            Paint::Ink,
        ));
        scene.push(element, budget)?;
        let mut label = Element::new(format!("nwdiag-interface-{id}"), "nwdiag-interface");
        label.name = membership.address.clone();
        label.owner = format!("nwdiag-membership-{id}");
        label.shapes.push(Shape::Text(
            Point::new(column.rect.x, bus.y + 10.),
            cable.address.clone(),
            Decoration::Plain,
        ));
        scene.push(label, budget)?;
    }
    Ok(())
}
fn paint_servers(
    model: &NwDiagram,
    measured: &Layout,
    scene: &mut Scene,
    budget: &mut Budget,
) -> Result<(), RenderError> {
    for server in &model.servers {
        budget.charge(8)?;
        budget.text(server.name.len() + 64)?;
        let column = &measured.columns[server.id.0];
        let mut element = Element::new(format!("nwdiag-server-{}", server.id.0), "nwdiag-server");
        element.name = server.name.clone();
        element
            .shapes
            .push(Shape::Rect(column.rect, 0., fill(server.fill, Paint::Node)));
        element
            .shapes
            .push(text_at(column.rect, column.label.clone()));
        scene.push(element, budget)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RenderOptions;
    use merman_render::environment::RenderEnvironment;

    fn inspect(source: &str, verify: impl FnOnce(NwDiagram, Layout, Scene)) {
        let mut budget = Budget::new(&RenderOptions::default());
        let (preamble, model) = crate::plantuml::parser::test_support::styled(
            source,
            &mut budget,
            crate::plantuml::parser::nwdiag::parse,
        )
        .unwrap();
        let session = RenderEnvironment::deterministic().begin_session().unwrap();
        let mut context = Context::new(&session, &mut budget).with_fonts(preamble.fonts);
        let measured = measure(&model, &mut context).unwrap();
        let scene = layout(&model, &mut context).unwrap();
        verify(model, measured, scene);
    }
    fn contains(outer: Rect, inner: Rect) -> bool {
        inner.x >= outer.x
            && inner.y >= outer.y
            && inner.x + inner.width <= outer.x + outer.width + 0.00001
            && inner.y + inner.height <= outer.y + outer.height + 0.00001
    }
    #[test]
    fn should_preserve_stable_model_ids_and_separate_network_interface_and_group_ownership() {
        // given
        let source = "@startnwdiag\nnwdiag {\nnetwork n {\naddress=cidr;\nr [address=old,description=Router];\nr [address=new];\nr;\ns;\n}\nnetwork core {\nr [address=other];\n}\ngroup internal {\ndescription=Zone;\ns;\n}\n}\n@endnwdiag";
        // when
        inspect(source, |model, _, _| {
            // then
            assert_eq!(
                model
                    .networks
                    .iter()
                    .map(|n| (n.id.0, n.name.as_str(), n.address.as_str()))
                    .collect::<Vec<_>>(),
                [(0, "n", "cidr"), (1, "core", "")]
            );
            assert_eq!(
                model
                    .servers
                    .iter()
                    .map(|s| (s.id.0, s.name.as_str(), s.display.as_str()))
                    .collect::<Vec<_>>(),
                [(0, "r", "Router"), (1, "s", "s")]
            );
            assert_eq!(
                model
                    .memberships
                    .iter()
                    .map(|m| (m.network.0, m.server.0, m.address.as_str()))
                    .collect::<Vec<_>>(),
                [(0, 0, "new"), (0, 1, ""), (1, 0, "other")]
            );
            assert_eq!(model.groups[0].members, [super::super::nwdiag::ServerId(1)]);
        });
    }
    #[test]
    fn should_connect_every_membership_to_its_bus_and_leave_foreign_crossings_open() {
        // given
        let source = "@startnwdiag\nnwdiag {\nnetwork a {\nwidth=full;\nr;\ns;\n}\nnetwork b {\nwidth=full;\ns;\n}\nnetwork c {\nwidth=full;\nr;\n}\n}\n@endnwdiag";
        // when
        inspect(source, |model, measured, scene| {
            // then
            assert_eq!(model.memberships.len(), 4);
            let cables = scene
                .elements
                .iter()
                .filter(|e| e.kind == "nwdiag-membership")
                .collect::<Vec<_>>();
            assert_eq!(cables.len(), 4);
            for (id, membership) in model.memberships.iter().enumerate() {
                let Shape::Line(points, _) = &cables[id].shapes[0] else {
                    panic!()
                };
                assert_eq!(points[0].y, measured.buses[membership.network.0].y);
                assert_eq!(points[3].y, measured.columns[membership.server.0].rect.y);
                assert_eq!(
                    cables[id].owner,
                    format!("nwdiag-network-{}", membership.network.0)
                );
                for (net, bus) in measured
                    .buses
                    .iter()
                    .enumerate()
                    .skip(membership.network.0 + 1)
                {
                    if points[0].x <= bus.start || points[0].x >= bus.end {
                        continue;
                    }
                    let network = scene
                        .elements
                        .iter()
                        .find(|e| e.id == format!("nwdiag-network-{net}"))
                        .unwrap();
                    assert!(!network.shapes.iter().any(|s|matches!(s,Shape::Rect(rect,_,_) if points[0].x>=rect.x && points[0].x<=rect.x+rect.width)),"foreign cable {id} touched bus {net}");
                }
            }
        });
    }
    #[test]
    fn should_reserve_measured_group_headers_interfaces_and_nodes_without_overlap() {
        // given
        for size in [6, 14, 96] {
            let source = format!(
                "@startnwdiag\nskinparam defaultFontSize {size}\nnwdiag {{\nnetwork n {{\ndescription=\"Long bus\\ncaption\";\naddress=\"CIDR value\";\na [description=\"Server A\\nsecond line\",address=\"Interface A\\naddress\"];\nb [description=\"Server B\"];\nc;\n}}\ngroup g {{\ndescription=\"A very wide measured group header\\nsecond header row\";\na;\nc;\n}}\n}}\n@endnwdiag"
            );
            // when
            inspect(&source, |model, measured, scene| {
                // then
                let enclosure = measured.groups[0].rect;
                for member in &model.groups[0].members {
                    assert!(contains(enclosure, measured.columns[member.0].rect));
                }
                assert!(!contains(enclosure, measured.columns[1].rect));
                let header = &measured.groups[0];
                assert!(header.rect.width >= header.label.width + 24.);
                assert!(header.rect.y + 6. + header.label.height < measured.columns[0].rect.y);
                for pair in measured.order.windows(2) {
                    let a = &measured.columns[pair[0]];
                    let b = &measured.columns[pair[1]];
                    assert!(a.rect.x + a.width < b.left);
                }
                for (index, membership) in model.memberships.iter().enumerate() {
                    let cable = &measured.cables[index];
                    let column = &measured.columns[membership.server.0];
                    let bus = &measured.buses[membership.network.0];
                    assert!(column.width >= cable.address.width + 24.);
                    assert!(bus.y + 10. + cable.address.height < column.rect.y);
                }
                for element in &scene.elements {
                    for shape in &element.shapes {
                        if let Shape::Text(p, label, _) = shape {
                            assert!(
                                p.x >= scene.min_x
                                    && p.y >= scene.min_y
                                    && p.x + label.width <= scene.width
                                    && p.y + label.height <= scene.height
                            );
                        }
                    }
                }
            });
        }
    }
    #[test]
    fn should_keep_bus_membership_storage_sparse() {
        // given
        let source = format!(
            "@startnwdiag\nnwdiag {{\n{}\n}}\n@endnwdiag",
            (0..120)
                .map(|id| format!("network n{id} {{\ns{id};\n}}"))
                .collect::<Vec<_>>()
                .join("\n")
        );
        // when
        inspect(&source, |model, measured, _| {
            // then
            assert_eq!(
                (
                    model.servers.len(),
                    model.networks.len(),
                    model.memberships.len()
                ),
                (120, 120, 120)
            );
            assert_eq!(
                measured
                    .buses
                    .iter()
                    .map(|b| b.members.len())
                    .sum::<usize>(),
                120
            );
            assert_eq!((measured.columns.len(), measured.cables.len()), (120, 120));
        });
    }
}
