//! Translation layer: CitySpec + AgentCatalog → ogun inputs.
//!
//! Reorders nodes by arrival strategy, generates edges from the
//! interaction matrix, and inflates node radii by per-category gap
//! so ogun's overlap rejection enforces road-width spacing.

use std::collections::HashMap;

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

    // Nodes in arrival order, radii inflated by per-category gap padding
    // so ogun's overlap check enforces minimum spacing between buildings.
    let nodes: Vec<Node> = order
        .iter()
        .enumerate()
        .map(|(new_idx, &orig_idx)| {
            let cat = catalog.templates[orig_idx].category;
            let padding = matrix.padding(cat).ceil() as u32;
            Node {
                id: NodeId(new_idx as u32),
                radius: catalog.templates[orig_idx].radius + padding,
            }
        })
        .collect();

    // Edges: only positive attraction creates ogun edges.
    // Same-category pairs are capped to K nearest neighbors to avoid O(n²)
    // road explosion (60 residential buildings → 1770 edges without cap).
    const K_SAME: usize = 3;

    // Count how many same-category edges each node already has.
    let mut same_count: Vec<usize> = vec![0; order.len()];

    let mut edges = Vec::new();
    let mut eid = 0u32;
    for (ni, &oi) in order.iter().enumerate() {
        for (nj, &oj) in order.iter().enumerate().skip(ni + 1) {
            let cat_i = catalog.templates[oi].category;
            let cat_j = catalog.templates[oj].category;
            let f = matrix.get(cat_i, cat_j);
            if f.attraction <= f32::EPSILON {
                continue;
            }
            // Cap same-category edges per node.
            if cat_i == cat_j {
                if same_count[ni] >= K_SAME || same_count[nj] >= K_SAME {
                    continue;
                }
                same_count[ni] += 1;
                same_count[nj] += 1;
            }
            edges.push(Edge {
                id: EdgeId(eid),
                src: NodeId(ni as u32),
                dst: NodeId(nj as u32),
                weight: f.attraction,
            });
            eid += 1;
        }
    }

    let graph = Graph { nodes, edges };
    let space = Space {
        width: spec.width,
        height: spec.height,
        obstacles: Vec::new(),
    };
    // Moderate repulsion_k to spread buildings across the grid.
    let grid_diag = ((spec.width * spec.width + spec.height * spec.height) as f32).sqrt();

    // Per-pair repulsion: category pairs with gap but no attraction get extra
    // repulsion force so ogun pushes them apart beyond the minimum spacing.
    let mut repulsion_pairs: HashMap<(NodeId, NodeId), f32> = HashMap::new();
    for (ni, &oi) in order.iter().enumerate() {
        for (nj, &oj) in order.iter().enumerate().skip(ni + 1) {
            let f = matrix.get(
                catalog.templates[oi].category,
                catalog.templates[oj].category,
            );
            if f.gap > 0.0 && f.attraction <= f32::EPSILON {
                repulsion_pairs.insert((NodeId(ni as u32), NodeId(nj as u32)), f.gap / 2.0);
            }
        }
    }

    let config = OgunConfig {
        beta: spec.beta,
        seed: spec.seed,
        repulsion_k: grid_diag * 2.0,
        repulsion_pairs,
        ..OgunConfig::default()
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
                Phase {
                    name: "founding".into(),
                    categories: vec![Military, Infrastructure],
                },
                Phase {
                    name: "core".into(),
                    categories: vec![Sacred, Commercial],
                },
                Phase {
                    name: "growth".into(),
                    categories: vec![Residential],
                },
            ],
        },
    }
}
