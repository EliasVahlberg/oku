//! Output formats — tile map and semantic grid representations.
//!
//! Stamps building footprints and road paths onto 2D grids.

use serde::{Deserialize, Serialize};

use crate::interpret::CityLayout;

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

impl CityLayout {
    /// Convert to a simple tile map.
    pub fn to_tilemap(&self) -> TileMap {
        let (w, h) = (self.width, self.height);
        let mut tiles = vec![Tile::Empty; (w * h) as usize];

        let idx = |x: u32, y: u32| (y * w + x) as usize;

        // Stamp roads first so buildings overwrite on overlap.
        for road in &self.roads {
            for &(rx, ry) in &road.path {
                if rx < w && ry < h {
                    tiles[idx(rx, ry)] = Tile::Road;
                }
            }
        }

        // Stamp building footprints.
        for b in &self.buildings {
            let r = b.radius as i32;
            for dy in -r..=r {
                for dx in -r..=r {
                    let bx = b.x as i32 + dx;
                    let by = b.y as i32 + dy;
                    if bx >= 0 && by >= 0 && (bx as u32) < w && (by as u32) < h {
                        tiles[idx(bx as u32, by as u32)] = Tile::Building;
                    }
                }
            }
        }

        TileMap {
            width: w,
            height: h,
            tiles,
        }
    }

    /// Convert to a semantic grid with per-cell references.
    pub fn to_semantic_grid(&self) -> SemanticGrid {
        let (w, h) = (self.width, self.height);
        let mut cells = vec![
            SemanticCell {
                tile: Tile::Empty,
                building_index: None,
                road_index: None
            };
            (w * h) as usize
        ];

        let idx = |x: u32, y: u32| (y * w + x) as usize;

        for (ri, road) in self.roads.iter().enumerate() {
            for &(rx, ry) in &road.path {
                if rx < w && ry < h {
                    let c = &mut cells[idx(rx, ry)];
                    c.tile = Tile::Road;
                    c.road_index = Some(ri);
                }
            }
        }

        for (bi, b) in self.buildings.iter().enumerate() {
            let r = b.radius as i32;
            for dy in -r..=r {
                for dx in -r..=r {
                    let bx = b.x as i32 + dx;
                    let by = b.y as i32 + dy;
                    if bx >= 0 && by >= 0 && (bx as u32) < w && (by as u32) < h {
                        let c = &mut cells[idx(bx as u32, by as u32)];
                        c.tile = Tile::Building;
                        c.building_index = Some(bi);
                    }
                }
            }
        }

        SemanticGrid {
            width: w,
            height: h,
            cells,
        }
    }
}
