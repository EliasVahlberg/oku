//! Interpretation layer: ogun::Layout → CityLayout.

use ogun::{Layout, NodeId, Pos};
use serde::{Deserialize, Serialize};

use crate::catalog::AgentCatalog;

/// A fully generated city layout with urban semantics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CityLayout {
    pub buildings: Vec<PlacedBuilding>,
    pub roads: Vec<Road>,
    pub score: f32,
}

/// A building placed at a specific position.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlacedBuilding {
    pub template_index: usize,
    pub name: String,
    pub x: u32,
    pub y: u32,
    pub radius: u32,
}

/// A routed road between two buildings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Road {
    pub from: usize,
    pub to: usize,
    pub path: Vec<(u32, u32)>,
}

/// Convert an abstract ogun layout into a domain-specific city layout.
pub fn interpret(layout: &Layout, catalog: &AgentCatalog) -> CityLayout {
    let buildings: Vec<PlacedBuilding> = layout
        .positions
        .iter()
        .map(|(&NodeId(id), &Pos { x, y })| {
            let t = &catalog.templates[id as usize];
            PlacedBuilding {
                template_index: id as usize,
                name: t.name.clone(),
                x,
                y,
                radius: t.radius,
            }
        })
        .collect();

    let roads: Vec<Road> = layout
        .paths
        .iter()
        .map(|(edge_id, path)| {
            Road {
                from: edge_id.0 as usize,
                to: edge_id.0 as usize,
                path: path.iter().map(|p| (p.x, p.y)).collect(),
            }
        })
        .collect();

    CityLayout {
        buildings,
        roads,
        score: layout.score,
    }
}
