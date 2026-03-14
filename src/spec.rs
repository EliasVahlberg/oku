//! City specification — what to generate.

use serde::{Deserialize, Serialize};

use crate::erosion::ErosionSpec;

/// Top-level specification for a city generation request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitySpec {
    pub width: u32,
    pub height: u32,
    pub city_type: CityType,
    pub era: Era,
    pub beta: f32,
    pub seed: u64,
    pub erosion: Option<ErosionSpec>,
}

/// The kind of settlement to generate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CityType {
    PlannedCapital,
    FrontierOutpost,
    TradeHub,
    Ruin,
}

/// The temporal era — influences agent availability and arrival order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Era {
    Founding,
    Growth,
    Decline,
    PostCollapse,
}
