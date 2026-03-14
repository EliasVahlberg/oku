//! # Oku
//!
//! PCB-inspired procedural city generation — a domain-specific facade over
//! [ogun](https://github.com/EliasVahlberg/ogun).
//!
//! Named after the Yoruba concept encompassing death and the afterlife —
//! fitting for a generator that builds cities meant to be found as ruins.
//!
//! ## Architecture
//!
//! ```text
//! CitySpec + AgentCatalog
//!       │
//!   translate  →  ogun::Graph + Space + Config
//!                       │
//!                 ogun::generate()
//!                       │
//!   interpret  ←  ogun::Layout
//!       │
//!   CityLayout
//!       │
//!   erosion (optional)
//!       │
//!   CityLayout (eroded)
//! ```
//!
//! Oku translates urban concepts (building types, road demands, growth phases)
//! into ogun's abstract inputs (nodes, edges, potential functions), then
//! interprets the output back into urban terms.

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
pub use catalog::{AgentCatalog, BuildingTemplate, Category};
pub use config::OkuConfig;
pub use erosion::ErosionSpec;
pub use interpret::CityLayout;
pub use output::{SemanticGrid, TileMap};
pub use spec::{CitySpec, CityType, Era};

/// Generate a city layout from a specification and agent catalog.
///
/// Deterministic: same `spec` + `catalog` = same output.
pub fn generate(spec: &CitySpec, catalog: &AgentCatalog) -> CityLayout {
    let (graph, space, config) = translate::translate(spec, catalog);
    let layout = ogun::generate(&graph, &space, &config);
    let mut city = interpret::interpret(&layout, catalog);

    if let Some(ref erosion) = spec.erosion {
        erosion::erode(&mut city, erosion);
    }

    city
}
