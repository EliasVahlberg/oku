//! Agent catalog — building templates and categories.

use serde::{Deserialize, Serialize};

/// Collection of building templates available for placement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCatalog {
    pub templates: Vec<BuildingTemplate>,
}

/// A building type that can be placed in the city.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildingTemplate {
    pub name: String,
    pub category: Category,
    pub radius: u32,
    pub priority: f32,
    pub connections: Vec<ConnectionDemand>,
}

/// Functional category of a building.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Category {
    Residential,
    Commercial,
    Sacred,
    Military,
    Infrastructure,
}

/// A demand for connection to another category.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionDemand {
    pub target: Category,
    pub weight: f32,
}
