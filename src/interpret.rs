//! Interpretation layer: ogun::Layout → CityLayout.
//!
//! Maps abstract positions and paths back to urban domain types,
//! using the graph edges to resolve road endpoints.

use std::collections::{HashMap, HashSet};

use ogun::{Graph, Layout, NodeId, Pos, ScoreBreakdown};
use serde::{Deserialize, Serialize};

use crate::catalog::AgentCatalog;

/// A fully generated city layout with urban semantics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CityLayout {
    pub buildings: Vec<PlacedBuilding>,
    pub roads: Vec<Road>,
    pub width: u32,
    pub height: u32,
    pub score: ScoreBreakdown,
    /// Template indices of buildings that couldn't be placed.
    pub unplaced: Vec<usize>,
    /// Per-building accessibility (fraction of edges routed). Indexed by buildings vec.
    pub accessibility: Vec<f32>,
    /// Per-edge routing cost. Indexed by roads vec (before merge).
    pub route_costs: Vec<f32>,
    /// Per-cell route overlap count, row-major `width * height`.
    pub congestion_grid: Vec<u32>,
}

/// A building placed at a specific position.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlacedBuilding {
    pub template_index: usize,
    pub name: String,
    pub category: crate::catalog::Category,
    pub material: crate::catalog::Material,
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
    /// Building indices adjacent to this road component (populated after merge).
    #[serde(default)]
    pub serves: Vec<usize>,
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
    // Map NodeId → index in the buildings vec (only placed nodes get an entry).
    let mut node_to_building: HashMap<u32, usize> = HashMap::new();
    let mut buildings: Vec<PlacedBuilding> = Vec::with_capacity(layout.positions.len());

    for (i, &orig) in order.iter().enumerate() {
        let nid = NodeId(i as u32);
        if let Some(&Pos { x, y }) = layout.positions.get(&nid) {
            let t = &catalog.templates[orig];
            node_to_building.insert(i as u32, buildings.len());
            buildings.push(PlacedBuilding {
                template_index: orig,
                name: t.name.clone(),
                category: t.category,
                material: t.material,
                x,
                y,
                radius: t.radius,
            });
        }
    }

    // Per-building accessibility from ogun's per-node metric.
    let mut accessibility: Vec<f32> = vec![0.0; buildings.len()];
    for (&nid_raw, &bi) in &node_to_building {
        if bi < accessibility.len() {
            accessibility[bi] = layout
                .node_accessibility
                .get(&NodeId(nid_raw))
                .copied()
                .unwrap_or(0.0);
        }
    }

    // Build roads from routed paths.
    let mut roads = Vec::new();
    let mut route_costs = Vec::new();
    for edge in &graph.edges {
        if let Some(path) = layout.paths.get(&edge.id) {
            let from = node_to_building.get(&edge.src.0).copied().unwrap_or(0);
            let to = node_to_building.get(&edge.dst.0).copied().unwrap_or(0);
            roads.push(Road {
                from,
                to,
                path: path.iter().map(|p| (p.x, p.y)).collect(),
                serves: vec![],
            });
            route_costs.push(layout.route_costs.get(&edge.id).copied().unwrap_or(0.0));
        }
    }

    // Unplaced: map NodeIds back to template indices.
    let unplaced: Vec<usize> = layout
        .unplaced
        .iter()
        .map(|nid| order[nid.0 as usize])
        .collect();

    CityLayout {
        buildings,
        roads,
        width,
        height,
        score: layout.score.clone(),
        unplaced,
        accessibility,
        route_costs,
        congestion_grid: layout.congestion_grid.clone(),
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
            // Find buildings adjacent to this road component.
            let mut serves = Vec::new();
            for &(cx, cy) in &component {
                for (bi, b) in self.buildings.iter().enumerate() {
                    if cx.abs_diff(b.x) <= b.radius + 1
                        && cy.abs_diff(b.y) <= b.radius + 1
                        && !serves.contains(&bi)
                    {
                        serves.push(bi);
                    }
                }
            }
            merged.push(Road {
                from: 0,
                to: 0,
                path: component,
                serves,
            });
        }

        self.roads = merged;
        self.route_costs.clear();
    }
}
