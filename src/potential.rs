//! Urban interaction potential — data-driven attraction/repulsion with spacing.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::catalog::Category;

/// Describes the spatial relationship between two building categories.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct InteractionFn {
    /// Positive = attract (ogun edge weight), negative/zero = no edge.
    pub attraction: f32,
    /// Minimum gap in grid cells between buildings of these categories.
    /// Enforced by inflating node radii in the translate layer.
    pub gap: f32,
}

/// A single interaction rule between two categories.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionRule {
    pub a: Category,
    pub b: Category,
    #[serde(flatten)]
    pub interaction: InteractionFn,
}

/// Matrix of pairwise category interactions, loaded from JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionMatrix {
    pub weights: Vec<InteractionRule>,
    #[serde(skip)]
    lookup: HashMap<(Category, Category), InteractionFn>,
}

impl Default for InteractionMatrix {
    fn default() -> Self {
        Self::default_urban()
    }
}

impl InteractionMatrix {
    /// Build the lookup cache. Call after deserializing.
    pub fn build(mut self) -> Self {
        self.lookup.clear();
        for rule in &self.weights {
            self.lookup.insert((rule.a, rule.b), rule.interaction);
            self.lookup.insert((rule.b, rule.a), rule.interaction);
        }
        self
    }

    /// Interaction function between two categories.
    pub fn get(&self, a: Category, b: Category) -> InteractionFn {
        self.lookup.get(&(a, b)).copied().unwrap_or(InteractionFn {
            attraction: 0.0,
            gap: 0.0,
        })
    }

    /// Maximum gap/2 for a category across all its interactions.
    /// Used to inflate node radii in the translate layer.
    pub fn padding(&self, cat: Category) -> f32 {
        self.lookup
            .iter()
            .filter(|&(&(a, _), _)| a == cat)
            .map(|(_, f)| f.gap / 2.0)
            .fold(0.0f32, f32::max)
    }

    /// Default urban interaction matrix.
    pub fn default_urban() -> Self {
        let m: Self = serde_json::from_str(include_str!("../data/default_weights.json"))
            .expect("embedded default_weights.json is valid");
        m.build()
    }
}
