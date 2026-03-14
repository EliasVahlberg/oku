//! Translation layer: CitySpec + AgentCatalog → ogun inputs.

use ogun::{Edge, EdgeId, Graph, Node, NodeId, OgunConfig, Space};

use crate::catalog::AgentCatalog;
use crate::spec::CitySpec;

/// Translate urban domain inputs into ogun's abstract graph, space, and config.
pub fn translate(spec: &CitySpec, catalog: &AgentCatalog) -> (Graph, Space, OgunConfig) {
    let nodes: Vec<Node> = catalog
        .templates
        .iter()
        .enumerate()
        .map(|(i, t)| Node {
            id: NodeId(i as u32),
            radius: t.radius,
        })
        .collect();

    // Build edges from connection demands between templates.
    let mut edges = Vec::new();
    let mut edge_id = 0u32;
    for (i, t) in catalog.templates.iter().enumerate() {
        for demand in &t.connections {
            for (j, other) in catalog.templates.iter().enumerate() {
                if j > i && other.category == demand.target {
                    edges.push(Edge {
                        id: EdgeId(edge_id),
                        src: NodeId(i as u32),
                        dst: NodeId(j as u32),
                        weight: demand.weight,
                    });
                    edge_id += 1;
                }
            }
        }
    }

    let graph = Graph { nodes, edges };
    let space = Space {
        width: spec.width,
        height: spec.height,
        obstacles: Vec::new(),
    };
    let config = OgunConfig {
        beta: spec.beta,
        seed: spec.seed,
        ..Default::default()
    };

    (graph, space, config)
}
