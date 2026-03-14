//! Interpretation layer: ogun::Layout → CityLayout.
//!
//! Maps abstract positions and paths back to urban domain types,
//! using the graph edges to resolve road endpoints.

use ogun::{Graph, Layout, NodeId, Pos};
use serde::{Deserialize, Serialize};

use crate::catalog::AgentCatalog;

/// A fully generated city layout with urban semantics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CityLayout {
    pub buildings: Vec<PlacedBuilding>,
    pub roads: Vec<Road>,
    pub width: u32,
    pub height: u32,
    pub score: f32,
}

/// A building placed at a specific position.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlacedBuilding {
    pub template_index: usize,
    pub name: String,
    pub category: crate::catalog::Category,
    pub x: u32,
    pub y: u32,
    pub radius: u32,
}

/// A routed road between two buildings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Road {
    /// Index into `buildings` vec for the source.
    pub from: usize,
    /// Index into `buildings` vec for the destination.
    pub to: usize,
    pub path: Vec<(u32, u32)>,
}

/// Convert an abstract ogun layout into a domain-specific city layout.
///
/// `order` maps node-index-in-graph → original template index in the catalog.
pub fn interpret(
    layout: &Layout,
    graph: &Graph,
    catalog: &AgentCatalog,
    order: &[usize],
    width: u32,
    height: u32,
) -> CityLayout {
    // Build buildings from placed positions.
    // NodeId(i) corresponds to order[i] in the catalog.
    let mut buildings: Vec<PlacedBuilding> = Vec::with_capacity(layout.positions.len());
    for i in 0..order.len() {
        let nid = NodeId(i as u32);
        if let Some(&Pos { x, y }) = layout.positions.get(&nid) {
            let orig = order[i];
            let t = &catalog.templates[orig];
            buildings.push(PlacedBuilding {
                template_index: orig,
                name: t.name.clone(),
                category: t.category,
                x,
                y,
                radius: t.radius,
            });
        }
    }

    // Build roads from routed paths, mapping EdgeId → (src NodeId, dst NodeId).
    let roads: Vec<Road> = layout
        .paths
        .iter()
        .filter_map(|(edge_id, path)| {
            let edge = graph.edges.iter().find(|e| e.id == *edge_id)?;
            Some(Road {
                from: edge.src.0 as usize,
                to: edge.dst.0 as usize,
                path: path.iter().map(|p| (p.x, p.y)).collect(),
            })
        })
        .collect();

    CityLayout {
        buildings,
        roads,
        width,
        height,
        score: layout.score,
    }
}
