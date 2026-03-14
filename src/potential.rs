//! Urban potential function — category interaction weights.
//!
//! Data-driven attraction/repulsion rules between building categories.
//! Ships with a default matrix; users can load custom weights from JSON.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::catalog::Category;

/// A single interaction rule between two categories.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionRule {
    pub a: Category,
    pub b: Category,
    pub weight: f32,
}

/// The full set of category interaction weights.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionMatrix {
    pub weights: Vec<InteractionRule>,
    #[serde(skip)]
    lookup: HashMap<(Category, Category), f32>,
}

impl InteractionMatrix {
    /// Build the lookup cache. Call after deserializing.
    pub fn build(mut self) -> Self {
        self.lookup.clear();
        for rule in &self.weights {
            self.lookup.insert((rule.a, rule.b), rule.weight);
            self.lookup.insert((rule.b, rule.a), rule.weight);
        }
        self
    }

    /// Interaction weight between two categories. Returns 0.0 for undefined pairs.
    pub fn weight(&self, a: Category, b: Category) -> f32 {
        self.lookup.get(&(a, b)).copied().unwrap_or(0.0)
    }

    /// Default urban interaction matrix.
    pub fn default_urban() -> Self {
        serde_json::from_str(include_str!("../data/default_weights.json"))
            .expect("embedded default_weights.json is valid")
    }
}

impl Default for InteractionMatrix {
    fn default() -> Self {
        Self::default_urban()
    }
}
