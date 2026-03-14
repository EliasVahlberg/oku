//! # Oku
//!
//! PCB-inspired procedural city generation — a domain-specific facade over
//! [ogun](https://crates.io/crates/ogun).
//!
//! ## Quick start
//!
//! ```rust
//! use oku::*;
//!
//! let catalog = AgentCatalog {
//!     templates: vec![
//!         BuildingTemplate {
//!             name: "market".into(),
//!             category: Category::Commercial,
//!             radius: 2,
//!             priority: 0.8,
//!             connections: vec![],
//!         },
//!         BuildingTemplate {
//!             name: "house".into(),
//!             category: Category::Residential,
//!             radius: 1,
//!             priority: 0.3,
//!             connections: vec![],
//!         },
//!     ],
//! };
//!
//! let spec = CitySpec {
//!     width: 40,
//!     height: 40,
//!     city_type: CityType::TradeHub,
//!     era: Era::Growth,
//!     beta: 2.0,
//!     seed: 42,
//!     erosion: None,
//! };
//!
//! let city = generate(&spec, &catalog);
//! let tilemap = city.to_tilemap();
//! ```

mod arrival;
mod catalog;
mod config;
mod erosion;
mod hierarchy;
mod interpret;
mod output;
mod potential;
mod spec;
mod translate;

pub use arrival::{ArrivalStrategy, Phase};
pub use catalog::{AgentCatalog, BuildingTemplate, Category, ConnectionDemand};
pub use config::OkuConfig;
pub use erosion::ErosionSpec;
pub use interpret::{CityLayout, PlacedBuilding, Road};
pub use output::{SemanticCell, SemanticGrid, Tile, TileMap};
pub use potential::InteractionMatrix;
pub use spec::{CitySpec, CityType, Era};

/// Generate a city layout from a specification and agent catalog.
///
/// Deterministic: same `spec` + `catalog` = same output.
pub fn generate(spec: &CitySpec, catalog: &AgentCatalog) -> CityLayout {
    let (graph, space, config, order) = translate::translate(spec, catalog);
    let layout = ogun::generate(&graph, &space, &config);
    let mut city = interpret::interpret(&layout, &graph, catalog, &order, spec.width, spec.height);

    if let Some(ref erosion) = spec.erosion {
        erosion::erode(&mut city, erosion);
    }

    city
}
