//! Interpretation layer: ogun::Layout → CityLayout.
//!
//! Maps abstract positions and paths back to urban domain types,
//! using the graph edges to resolve road endpoints.

use std::collections::HashSet;

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

impl CityLayout {
    /// Merge overlapping roads into a unified network.
    ///
    /// Deduplicates road cells, removes cells inside building footprints,
    /// thins thick blobs, and rebuilds as connected components.
    pub fn merge_roads(&mut self) {
        // 1. Collect unique road cells.
        let mut road_cells: HashSet<(u32, u32)> = HashSet::new();
        for road in &self.roads {
            road_cells.extend(&road.path);
        }

        // 2. Remove cells inside building footprints.
        for b in &self.buildings {
            let r = b.radius as i32;
            for dy in -r..=r {
                for dx in -r..=r {
                    let bx = b.x as i32 + dx;
                    let by = b.y as i32 + dy;
                    if bx >= 0 && by >= 0 {
                        road_cells.remove(&(bx as u32, by as u32));
                    }
                }
            }
        }

        // 3. Thin: iteratively remove fully-interior cells
        //    (all 4 cardinal neighbors are road → safe to remove).
        loop {
            let removable: Vec<_> = road_cells
                .iter()
                .copied()
                .filter(|&(x, y)| {
                    x > 0
                        && y > 0
                        && road_cells.contains(&(x - 1, y))
                        && road_cells.contains(&(x + 1, y))
                        && road_cells.contains(&(x, y - 1))
                        && road_cells.contains(&(x, y + 1))
                })
                .collect();
            if removable.is_empty() {
                break;
            }
            for cell in removable {
                road_cells.remove(&cell);
            }
        }

        // 4. Rebuild roads from 4-connected components.
        let mut visited: HashSet<(u32, u32)> = HashSet::new();
        let mut merged = Vec::new();

        for &start in &road_cells {
            if !visited.insert(start) {
                continue;
            }
            let mut component = vec![start];
            let mut queue = vec![start];
            while let Some((x, y)) = queue.pop() {
                for (nx, ny) in [
                    (x + 1, y),
                    (x.wrapping_sub(1), y),
                    (x, y + 1),
                    (x, y.wrapping_sub(1)),
                ] {
                    if road_cells.contains(&(nx, ny)) && visited.insert((nx, ny)) {
                        component.push((nx, ny));
                        queue.push((nx, ny));
                    }
                }
            }
            merged.push(Road { from: 0, to: 0, path: component });
        }

        self.roads = merged;
    }
}
