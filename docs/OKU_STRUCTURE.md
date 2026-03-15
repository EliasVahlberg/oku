# Oku — PCB-Inspired Procedural City Generator

> Status: v0.2.0 — published on [crates.io](https://crates.io/crates/oku)
> Last updated: 2026-03-15

Named after the Yoruba concept encompassing death and the afterlife — fitting for a generator that builds cities meant to be found as ruins.

---

## Relationship to Ogun

```
┌─────────────────────────────────────────────┐
│  oku (application crate)                    │
│                                             │
│  CitySpec ──→ translate ──→ ogun::Graph     │
│                             ogun::Space     │
│                             ogun::OgunConfig│
│                                │            │
│                          ogun::generate()   │
│                                │            │
│  CityLayout ←── interpret ←── ogun::Layout  │
│       │                                     │
│    erosion (optional)                       │
│       │                                     │
│  CityLayout (eroded)                        │
└─────────────────────────────────────────────┘
```

- **ogun** (v0.3.0) — domain-agnostic. Nodes, edges, positions, potential functions, β, negotiated routing, per-pair repulsion, potential kernel. Does not know what a "building" or "road" is.
- **oku** — domain-specific. Translates urban concepts into ogun's abstract inputs, interprets ogun's abstract outputs back into urban terms. Owns erosion, hierarchy, and output formatting.

---

## What Oku Owns

| Responsibility | Why Oku, not Ogun |
|---------------|-------------------|
| Agent catalog (building types, footprints, materials, connection demands) | Domain knowledge — what structures exist in a city |
| Interaction matrix (attraction/repulsion rules between building categories) | Domain knowledge — what makes a "good" city layout |
| Arrival ordering (founding → growth → infill) | Domain knowledge — how cities grow over time |
| Functional erosion (material-aware cascading degradation) | Post-processing with urban semantics — uses material durability + ogun accessibility |
| Hierarchical generation (district → block → building) | Multi-scale orchestration — calls ogun multiple times at different scales |
| Output formatting (tile map, semantic grid) | Consumer-facing — ogun outputs abstract positions/paths |
| PCB-to-city mapping rules | The conceptual bridge that motivates the whole project |

---

## Core Abstractions

### CitySpec — what to generate

```rust
pub struct CitySpec {
    pub width: u32,
    pub height: u32,
    pub city_type: CityType,       // PlannedCapital, FrontierOutpost, TradeHub, Ruin
    pub era: Era,                  // Founding, Growth, Decline, PostCollapse
    pub beta: f32,                 // passed to ogun — controls optimization level
    pub seed: u64,
    pub erosion: Option<ErosionSpec>,
}
```

CityType and Era influence arrival order and the interaction matrix. A `PlannedCapital` uses priority-based arrival; a `TradeHub` uses phased arrival (founding → core → growth).

### AgentCatalog — what can be placed

```rust
pub struct AgentCatalog {
    pub templates: Vec<BuildingTemplate>,
}

pub struct BuildingTemplate {
    pub name: String,
    pub category: Category,            // Residential, Commercial, Sacred, Military, Infrastructure
    pub radius: u32,                   // footprint radius (ogun node radius)
    pub priority: f32,                 // arrival priority (higher = placed earlier)
    pub connections: Vec<ConnectionDemand>,  // per-template road demands
    pub material: Material,            // Stone, Metal, Wood, Glass — affects erosion durability
}
```

`ConnectionDemand` generates targeted edges to the nearest building of a given category, supplementing the category-level `InteractionMatrix`.

### InteractionMatrix — the potential function

Oku constructs ogun's edge weights and per-pair repulsion from an `InteractionMatrix` of `InteractionFn { attraction, gap }` values per category pair:

- `attraction > 0` → ogun edge with that weight (buildings pull toward each other)
- `gap > 0` → radius inflation for minimum spacing + extra repulsion force via `repulsion_pairs`
- Same-category edges capped at K=3 nearest neighbors to prevent O(n²) explosion

### ArrivalOrder — temporal structure

```rust
pub enum ArrivalStrategy {
    Phased { phases: Vec<Phase> },  // founding → core → growth
    Priority,                        // highest priority first
    Random,                          // for ruins / chaotic settlements
}

pub struct Phase {
    pub name: String,
    pub categories: Vec<Category>,
}
```

Default for TradeHub/FrontierOutpost: Military+Infrastructure → Sacred+Commercial → Residential.

### Erosion — material-aware degradation

```rust
pub struct ErosionSpec {
    pub severity: f32,          // 0.0 = pristine, 1.0 = total ruin
    pub seed: u64,
}
```

Erosion scoring uses material durability, ogun-provided accessibility, and noise:

```
durability = material.durability() * 0.4 + accessibility * 0.4 + noise * 0.2
```

| Material | Base Durability |
|----------|----------------|
| Stone    | 0.9            |
| Metal    | 0.7            |
| Wood     | 0.4            |
| Glass    | 0.2            |

Stone temples outlast wooden houses. Buildings that lose road access decay faster.

### CityLayout — the output

```rust
pub struct CityLayout {
    pub buildings: Vec<PlacedBuilding>,
    pub roads: Vec<Road>,
    pub width: u32,
    pub height: u32,
    pub score: ScoreBreakdown,
    pub unplaced: Vec<usize>,
    pub accessibility: Vec<f32>,
    pub route_costs: Vec<f32>,
    pub congestion_grid: Vec<u32>,
}

pub struct Road {
    pub from: usize,
    pub to: usize,
    pub path: Vec<(u32, u32)>,
    pub serves: Vec<usize>,    // building indices adjacent to this road (after merge)
}

impl CityLayout {
    pub fn to_tilemap(&self) -> TileMap { ... }
    pub fn to_semantic_grid(&self) -> SemanticGrid { ... }
}
```

---

## Module Structure

```
oku/
├── Cargo.toml
├── src/
│   ├── lib.rs          # pub fn generate(spec, catalog) -> CityLayout
│   ├── spec.rs         # CitySpec, CityType, Era
│   ├── catalog.rs      # AgentCatalog, BuildingTemplate, Category, Material
│   ├── potential.rs    # InteractionMatrix, InteractionFn
│   ├── arrival.rs      # ArrivalStrategy, Phase, ordering logic
│   ├── translate.rs    # CitySpec + Catalog → ogun::Graph + Space + OgunConfig
│   ├── interpret.rs    # ogun::Layout → CityLayout, merge_roads()
│   ├── erosion.rs      # Material-aware functional erosion
│   ├── hierarchy.rs    # Multi-scale generation (stub)
│   ├── output.rs       # TileMap, SemanticGrid output formats
│   └── config.rs       # Reserved
├── examples/
│   ├── visualize.rs    # Terminal renderer with colored Unicode
│   └── svg.rs          # SVG visualization generator
├── benches/
│   └── city_generation.rs  # Criterion benchmarks
├── data/
│   └── default_weights.json
└── docs/
```

---

## Generation Flow

```
1. Load CitySpec + AgentCatalog

2. translate(spec, catalog):
   a. Order templates by arrival strategy
   b. Build ogun nodes (radii inflated by gap padding)
   c. Build ogun edges from InteractionMatrix + ConnectionDemand
   d. Build per-pair repulsion from gap values
   e. Return Graph + Space + OgunConfig + order mapping

3. ogun::generate(graph, space, config):
   a. Sequential Boltzmann placement with potential function
   b. Negotiated Dijkstra routing with rip-up-and-reroute
   c. Return Layout with positions, paths, scores, metadata

4. interpret(layout, graph, catalog, order):
   a. Map NodeIds back to PlacedBuildings
   b. Map EdgeIds back to Roads with from/to building indices
   c. Carry accessibility, route costs, congestion grid

5. merge_roads():
   a. Deduplicate road cells, remove building overlaps
   b. Thin interior cells, rebuild as connected components
   c. Populate Road.serves with adjacent building indices

6. If erosion requested:
   a. Score buildings by material durability + accessibility + noise
   b. Remove weakest, reindex roads, repeat until severity target

7. Return CityLayout
```

---

## Dependencies

```toml
[dependencies]
ogun = "0.3"
rand = "0.9"
rand_chacha = "0.9"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

---

## Open Questions

1. **Hierarchical generation** — `hierarchy.rs` is a stub. Needs ogun's `routing_costs` (available in v0.3.0) to mark outer-scale roads as preferred corridors for inner-scale routing.

2. **terrain-forge integration** — Should Oku implement terrain-forge's `Algorithm` trait? Tradeoff: constrains output to terrain-forge's grid model.

3. **Saltglass coupling** — Glass storm erosion is game-specific. Core Oku stays generic; Saltglass extensions via feature flag.
