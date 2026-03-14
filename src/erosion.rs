//! Functional erosion — cascading degradation of city layouts.

use serde::{Deserialize, Serialize};

use crate::interpret::CityLayout;

/// Erosion parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErosionSpec {
    /// 0.0 = pristine, 1.0 = total ruin.
    pub severity: f32,
    pub seed: u64,
}

/// Apply functional erosion to a city layout.
///
/// Degrades buildings and roads based on severity, producing ruins that
/// tell a story through their pattern of collapse.
pub fn erode(city: &mut CityLayout, _spec: &ErosionSpec) {
    // TODO: Implement cascading degradation.
    // 1. Select weakest link (lowest durability road/building)
    // 2. Degrade it
    // 3. Propagate: dependent structures lose accessibility → accelerated decay
    // 4. Repeat until severity target reached
    let _ = city;
}
