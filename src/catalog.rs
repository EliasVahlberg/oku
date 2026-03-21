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
    pub width: u32,
    pub height: u32,
    pub priority: f32,
    pub connections: Vec<ConnectionDemand>,
    #[serde(default)]
    pub material: Material,
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

/// Building material — affects erosion durability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum Material {
    #[default]
    Stone,
    Metal,
    Wood,
    Glass,
}

impl Material {
    pub fn durability(self) -> f32 {
        match self {
            Material::Stone => 0.9,
            Material::Metal => 0.7,
            Material::Wood => 0.4,
            Material::Glass => 0.2,
        }
    }
}
