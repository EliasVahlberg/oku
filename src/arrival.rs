//! Arrival ordering — controls the temporal structure of city growth.

use serde::{Deserialize, Serialize};

use crate::catalog::Category;

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
    pub count_min: usize,
    pub count_max: usize,
}
