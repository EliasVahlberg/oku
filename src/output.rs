//! Output formats — tile map and semantic grid representations.

use serde::{Deserialize, Serialize};

/// A 2D tile map suitable for rendering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileMap {
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<Tile>,
}

/// A single tile in the output map.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tile {
    Empty,
    Building,
    Road,
    Wall,
}

/// A semantic grid with richer per-cell information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticGrid {
    pub width: u32,
    pub height: u32,
    pub cells: Vec<SemanticCell>,
}

/// Semantic information for a single grid cell.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticCell {
    pub tile: Tile,
    pub building_index: Option<usize>,
    pub road_index: Option<usize>,
}
