//! Translation layer: CitySpec + AgentCatalog → ogun inputs.
//!
//! Reorders nodes by arrival strategy, generates edges from the
//! interaction matrix, and maps domain types to ogun abstractions.

use ogun::{Edge, EdgeId, Graph, Node, NodeId, OgunConfig, Space};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use crate::arrival::{self, ArrivalStrategy, Phase};
use crate::catalog::{AgentCatalog, Category};
use crate::potential::InteractionMatrix;
use crate::spec::{CitySpec, CityType};

/// Translate urban domain inputs into ogun's abstract graph, space, and config.
///
/// Returns the graph (with nodes in arrival order), the ordered template
/// indices (so interpret can map NodeId back to the original template),
/// and the ogun space + config.
pub fn translate(
    spec: &CitySpec,
    catalog: &AgentCatalog,
) -> (Graph, Space, OgunConfig, Vec<usize>) {
    let mut rng = ChaCha8Rng::seed_from_u64(spec.seed);
    let strategy = default_strategy(spec.city_type);
    let order = arrival::order_agents(&catalog.templates, &strategy, &mut rng);

    let matrix = InteractionMatrix::default();

    // Nodes in arrival order.
    let nodes: Vec<Node> = order
        .iter()
        .enumerate()
        .map(|(new_idx, &orig_idx)| Node {
            id: NodeId(new_idx as u32),
            radius: catalog.templates[orig_idx].radius,
        })
        .collect();

    // Edges from interaction matrix — connect every pair with non-zero weight.
    let mut edges = Vec::new();
    let mut eid = 0u32;
    for (ni, &oi) in order.iter().enumerate() {
        for (nj, &oj) in order.iter().enumerate().skip(ni + 1) {
            let w = matrix.weight(
                catalog.templates[oi].category,
                catalog.templates[oj].category,
            );
            if w > 0.0 {
                edges.push(Edge {
                    id: EdgeId(eid),
                    src: NodeId(ni as u32),
                    dst: NodeId(nj as u32),
                    weight: w,
                });
                eid += 1;
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

    (graph, space, config, order)
}

/// Default arrival strategy based on city type.
fn default_strategy(city_type: CityType) -> ArrivalStrategy {
    use Category::*;
    match city_type {
        CityType::PlannedCapital => ArrivalStrategy::Priority,
        CityType::Ruin => ArrivalStrategy::Random,
        CityType::FrontierOutpost | CityType::TradeHub => ArrivalStrategy::Phased {
            phases: vec![
                Phase { name: "founding".into(), categories: vec![Military, Infrastructure] },
                Phase { name: "core".into(), categories: vec![Sacred, Commercial] },
                Phase { name: "growth".into(), categories: vec![Residential] },
            ],
        },
    }
}
