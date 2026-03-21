//! Interpretation layer: ogun::Layout → CityLayout.
//!
//! Maps abstract positions and paths back to urban domain types,
//! using the graph edges to resolve road endpoints.

use std::collections::{HashMap, HashSet};

use ogun::{Graph, Layout, NodeId, Pos, ScoreBreakdown};
use serde::{Deserialize, Serialize};

use crate::catalog::AgentCatalog;

/// Cardinal direction for building facing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

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
    pub width: u32,
    pub height: u32,
    /// Direction the building entrance faces (toward nearest road).
    pub facing: Option<Direction>,
}

/// A routed road between two buildings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Road {
    pub from: usize,
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
                width: t.width,
                height: t.height,
                facing: None, // computed after roads are built
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

    // Compute facing: direction from building center to nearest road cell.
    compute_facing(&mut buildings, &roads, width, height);

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

/// Compute facing direction for each building based on nearest road cell.
fn compute_facing(buildings: &mut [PlacedBuilding], roads: &[Road], width: u32, height: u32) {
    // Collect all road cells into a set for fast lookup.
    let road_cells: HashSet<(u32, u32)> =
        roads.iter().flat_map(|r| r.path.iter().copied()).collect();

    for b in buildings.iter_mut() {
        let (cx, cy) = (b.x as i64, b.y as i64);
        let hw = b.width as i64 / 2;
        let hh = b.height as i64 / 2;

        // Find nearest road cell by scanning a perimeter ring just outside the footprint.
        let mut best_dist = i64::MAX;
        let mut best_dir = None;

        // Check cells adjacent to each edge of the building footprint.
        for d in 1..=3i64 {
            // North edge (y = cy - hh - d)
            let ny = cy - hh - d;
            if ny >= 0 {
                for dx in -hw..=hw {
                    let nx = cx + dx;
                    if nx >= 0
                        && (nx as u32) < width
                        && road_cells.contains(&(nx as u32, ny as u32))
                    {
                        let dist = dx.abs() + d;
                        if dist < best_dist {
                            best_dist = dist;
                            best_dir = Some(Direction::North);
                        }
                    }
                }
            }
            // South edge
            let sy = cy + hh + d;
            if (sy as u32) < height {
                for dx in -hw..=hw {
                    let sx = cx + dx;
                    if sx >= 0
                        && (sx as u32) < width
                        && road_cells.contains(&(sx as u32, sy as u32))
                    {
                        let dist = dx.abs() + d;
                        if dist < best_dist {
                            best_dist = dist;
                            best_dir = Some(Direction::South);
                        }
                    }
                }
            }
            // West edge
            let wx = cx - hw - d;
            if wx >= 0 {
                for dy in -hh..=hh {
                    let wy = cy + dy;
                    if wy >= 0
                        && (wy as u32) < height
                        && road_cells.contains(&(wx as u32, wy as u32))
                    {
                        let dist = dy.abs() + d;
                        if dist < best_dist {
                            best_dist = dist;
                            best_dir = Some(Direction::West);
                        }
                    }
                }
            }
            // East edge
            let ex = cx + hw + d;
            if (ex as u32) < width {
                for dy in -hh..=hh {
                    let ey = cy + dy;
                    if ey >= 0
                        && (ey as u32) < height
                        && road_cells.contains(&(ex as u32, ey as u32))
                    {
                        let dist = dy.abs() + d;
                        if dist < best_dist {
                            best_dist = dist;
                            best_dir = Some(Direction::East);
                        }
                    }
                }
            }
            if best_dir.is_some() {
                break;
            }
        }

        // Fallback: face toward grid center.
        if best_dir.is_none() {
            let gcx = width as i64 / 2;
            let gcy = height as i64 / 2;
            let dx = gcx - cx;
            let dy = gcy - cy;
            best_dir = Some(if dx.abs() >= dy.abs() {
                if dx > 0 {
                    Direction::East
                } else {
                    Direction::West
                }
            } else if dy > 0 {
                Direction::South
            } else {
                Direction::North
            });
        }

        b.facing = best_dir;
    }
}

impl CityLayout {
    /// Merge overlapping roads into a unified network.
    ///
    /// Deduplicates road cells, removes cells inside building footprints,
    /// thins thick blobs, and rebuilds as connected components.
    pub fn merge_roads(&mut self) {
        let mut road_cells: HashSet<(u32, u32)> = HashSet::new();
        for road in &self.roads {
            road_cells.extend(&road.path);
        }

        // Remove cells inside building footprints (rectangular).
        for b in &self.buildings {
            let hw = b.width / 2;
            let hh = b.height / 2;
            let x0 = b.x.saturating_sub(hw);
            let y0 = b.y.saturating_sub(hh);
            let x1 = b.x + hw;
            let y1 = b.y + hh;
            for y in y0..=y1 {
                for x in x0..=x1 {
                    road_cells.remove(&(x, y));
                }
            }
        }

        // Thin: iteratively remove fully-interior cells.
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

        // Rebuild roads from 4-connected components.
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
                    let hw = b.width / 2 + 1;
                    let hh = b.height / 2 + 1;
                    if cx.abs_diff(b.x) <= hw && cy.abs_diff(b.y) <= hh && !serves.contains(&bi) {
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
