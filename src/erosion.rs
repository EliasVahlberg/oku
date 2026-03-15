//! Functional erosion — cascading degradation of city layouts.
//!
//! Removes buildings by durability, then propagates connectivity loss:
//! structures that lose road access decay faster.

use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
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
/// Removes buildings (weakest first, with connectivity cascade),
/// then removes orphaned roads.
pub fn erode(city: &mut CityLayout, spec: &ErosionSpec) {
    if city.buildings.is_empty() || spec.severity <= 0.0 {
        return;
    }

    let mut rng = ChaCha8Rng::seed_from_u64(spec.seed);
    let target = (city.buildings.len() as f32 * spec.severity.clamp(0.0, 1.0)) as usize;

    for _ in 0..target {
        if city.buildings.is_empty() {
            break;
        }

        // Score each building: lower = more vulnerable.
        // Accessibility from ogun measures how well-connected each building is.
        let scores: Vec<f32> = city
            .buildings
            .iter()
            .enumerate()
            .map(|(i, b)| {
                let acc = city.accessibility.get(i).copied().unwrap_or(0.0);
                b.material.durability() * 0.4 + acc * 0.4 + rng.random::<f32>() * 0.2
            })
            .collect();

        // Remove the weakest.
        let victim = scores
            .iter()
            .enumerate()
            .min_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(i, _)| i)
            .unwrap();

        city.buildings.remove(victim);
        city.accessibility.remove(victim);

        // Reindex road references after removal.
        city.roads.retain(|r| r.from != victim && r.to != victim);
        for road in &mut city.roads {
            if road.from > victim {
                road.from -= 1;
            }
            if road.to > victim {
                road.to -= 1;
            }
        }
    }
}
