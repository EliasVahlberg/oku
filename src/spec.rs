//! City specification — what to generate.

use serde::{Deserialize, Serialize};

use crate::arrival::ArrivalStrategy;
use crate::erosion::ErosionSpec;
use crate::potential::InteractionMatrix;

/// Top-level specification for a city generation request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitySpec {
    pub width: u32,
    pub height: u32,
    pub city_type: CityType,
    pub era: Era,
    pub beta: f32,
    pub seed: u64,
    #[serde(default)]
    pub erosion: Option<ErosionSpec>,
    /// Per-cell placement/routing cost (flat vec, width × height, row-major).
    /// High values discourage placement and increase routing cost.
    #[serde(default)]
    pub terrain_costs: Option<Vec<f32>>,
    /// Rectangular obstacles fully blocked for placement and routing.
    #[serde(default)]
    pub obstacles: Vec<(u32, u32, u32, u32)>,
    /// Override the default arrival strategy for this city type.
    #[serde(default)]
    pub arrival_strategy: Option<ArrivalStrategy>,
    /// Override the default interaction matrix.
    #[serde(default)]
    pub interaction_matrix: Option<InteractionMatrix>,
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
