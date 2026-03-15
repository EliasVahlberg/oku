//! Arrival ordering — controls the temporal structure of city growth.
//!
//! Ogun processes nodes in vector order, so reordering controls which
//! buildings get placed first (with more freedom) vs last (more constrained).

use rand::seq::SliceRandom;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use crate::catalog::{BuildingTemplate, Category};

/// Strategy for ordering agent placement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArrivalStrategy {
    /// Founding structures first, then growth phases.
    Phased { phases: Vec<Phase> },
    /// Highest priority first.
    Priority,
    /// Random order (for ruins / chaotic settlements).
    Random,
}

/// A named generation phase filtering by category.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phase {
    pub name: String,
    pub categories: Vec<Category>,
}

/// Return template indices in arrival order.
pub fn order_agents(
    templates: &[BuildingTemplate],
    strategy: &ArrivalStrategy,
    rng: &mut ChaCha8Rng,
) -> Vec<usize> {
    match strategy {
        ArrivalStrategy::Priority => {
            let mut idx: Vec<usize> = (0..templates.len()).collect();
            idx.sort_by(|&a, &b| {
                templates[b]
                    .priority
                    .partial_cmp(&templates[a].priority)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            idx
        }
        ArrivalStrategy::Phased { phases } => {
            let mut seen = vec![false; templates.len()];
            let mut idx = Vec::with_capacity(templates.len());
            for phase in phases {
                for (i, t) in templates.iter().enumerate() {
                    if !seen[i] && phase.categories.contains(&t.category) {
                        idx.push(i);
                        seen[i] = true;
                    }
                }
            }
            // Append any templates not covered by phases.
            for (i, &s) in seen.iter().enumerate() {
                if !s {
                    idx.push(i);
                }
            }
            idx
        }
        ArrivalStrategy::Random => {
            let mut idx: Vec<usize> = (0..templates.len()).collect();
            idx.shuffle(rng);
            idx
        }
    }
}
