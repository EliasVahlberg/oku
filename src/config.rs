//! Serializable generation configuration.

use serde::{Deserialize, Serialize};

/// Top-level configuration combining all Oku parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OkuConfig {
    pub beta: f32,
    pub seed: u64,
    pub repulsion_k: f32,
}

impl Default for OkuConfig {
    fn default() -> Self {
        Self {
            beta: 2.0,
            seed: 0,
            repulsion_k: 50.0,
        }
    }
}
