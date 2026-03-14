# Oku — PCB-Inspired Procedural City Generator

> Status: Structural exploration
> Last updated: 2026-03-14

Named after the Yoruba concept encompassing death and the afterlife — fitting for a generator that builds cities meant to be found as ruins.

---

## Relationship to Ogun

```
┌─────────────────────────────────────────────┐
│  oku (application crate)                    │
│                                             │
│  CitySpec ──→ translate ──→ ogun::Graph     │
│                             ogun::Space     │
│                             ogun::Config    │
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

- **ogun** — domain-agnostic. Knows about nodes, edges, positions, potential functions, β. Does not know what a "building" or "road" is.
- **oku** — domain-specific. Translates urban concepts into ogun's abstract inputs, interprets ogun's abstract outputs back into urban terms. Owns erosion, hierarchy, and output formatting.

---

## What Oku Owns

| Responsibility | Why Oku, not Ogun |
|---------------|-------------------|
| Agent catalog (building types, footprints, connection demands) | Domain knowledge — what structures exist in a city |
| Urban potential function (attraction/repulsion rules between building types) | Domain knowledge — what makes a "good" city layout |
| Arrival ordering (founding → growth → infill) | Domain knowledge — how cities grow over time |
| Functional erosion (cascading degradation) | Post-processing with urban semantics — ogun doesn't know about material durability |
| Hierarchical generation (district → block → building) | Multi-scale orchestration — calls ogun multiple times at different scales |
| Output formatting (tile map, semantic grid) | Consumer-facing — ogun outputs abstract positions/paths |
| PCB-to-city mapping rules | The conceptual bridge that motivates the whole project |

---

## Core Abstractions

### CitySpec — what to generate

```rust
pub struct CitySpec {
    pub bounds: Rect,              // generation area
    pub city_type: CityType,       // planned_capital, frontier_outpost, trade_hub, ruin...
    pub era: Era,                  // founding, growth, decline, post_collapse
    pub beta: f32,                 // passed to ogun — controls optimization level
    pub erosion: Option<ErosionSpec>,
    pub seed: u64,
}
```

CityType and Era influence which agents are available, their arrival order, and the potential function weights. A `planned_capital` has more infrastructure agents arriving early; a `frontier_outpost` has sparse founding structures with organic infill.

### AgentCatalog — what can be placed

```rust
pub struct AgentCatalog {
    pub templates: Vec<BuildingTemplate>,
}

pub struct BuildingTemplate {
    pub id: TemplateId,
    pub name: String,
    pub footprint: Footprint,          // size and shape
    pub category: Category,            // residential, commercial, sacred, military, infrastructure
    pub connections: Vec<ConnectionDemand>,  // what it needs to connect to
    pub utility_profile: UtilityProfile,    // attraction/repulsion weights per category
    pub material: Material,            // for erosion: stone, wood, glass, metal
    pub priority: f32,                 // arrival priority (higher = placed earlier)
}
```

This is data-driven — loaded from JSON/RON files, matching Saltglass Steppe's content pipeline.

### UrbanPotential — the potential function

Oku constructs ogun's potential function from urban rules:

```rust
// Oku translates these rules into ogun's potential function terms:
//
// - Markets attract residential, repel other markets (competition)
// - Temples attract housing, repel industry (sacred space)
// - Walls attract military, define city boundary
// - All buildings incur congestion cost on shared road cells
// - Infrastructure (wells, granaries) attract everything within radius
//
// These become kernel functions in ogun's EVAL_UTILITY
```

The potential function is the creative lever — different rule sets produce different city characters without changing the algorithm.

### ArrivalOrder — temporal structure

```rust
pub enum ArrivalStrategy {
    /// Founding structures first, then growth phases
    Phased {
        phases: Vec<Phase>,
    },
    /// Priority-ordered (highest priority first)
    Priority,
    /// Random order (for ruins / chaotic settlements)
    Random,
}

pub struct Phase {
    pub name: String,           // "founding", "expansion", "infill"
    pub filter: CategoryFilter, // which agent categories appear in this phase
    pub count: Range<usize>,    // how many agents in this phase
}
```

Phased arrival is the default: walls and gates first, then core infrastructure (market, temple, well), then residential infill, then secondary commercial. This produces the temporal layering that makes layouts readable.

### Erosion — functional degradation

```rust
pub struct ErosionSpec {
    pub severity: f32,          // 0.0 = pristine, 1.0 = total ruin
    pub seed: u64,
    pub storm_exposure: Option<StormMap>,  // saltglass-specific: glass storm damage zones
}
```

Erosion is a post-processing pass on the CityLayout. It doesn't call ogun — it operates on the placed/routed result:

1. Select weakest link (lowest durability road segment or building)
2. Degrade it (reduce function, eventually collapse)
3. Propagate: structures that depended on it lose accessibility → accelerated decay
4. Repeat until severity target reached

The cascade produces ruins that tell a story — you can read which road failed first by tracing the pattern of collapse outward.

### CityLayout — the output

```rust
pub struct CityLayout {
    pub buildings: Vec<PlacedBuilding>,
    pub roads: Vec<Road>,
    pub districts: Vec<District>,       // from hierarchical generation
    pub metadata: LayoutMetadata,       // scores, connectivity, congestion map
}

impl CityLayout {
    pub fn to_tilemap(&self, palette: &TilePalette) -> TileMap { ... }
    pub fn to_semantic_grid(&self) -> SemanticGrid { ... }
}
```

---

## Module Structure

```
oku/
├── Cargo.toml
├── src/
│   ├── lib.rs                  # pub fn generate(spec, catalog) -> CityLayout
│   ├── spec.rs                 # CitySpec, CityType, Era
│   ├── catalog.rs              # AgentCatalog, BuildingTemplate, Category
│   ├── potential.rs            # Urban potential function → ogun potential terms
│   ├── arrival.rs              # ArrivalStrategy, Phase, ordering logic
│   ├── translate.rs            # CitySpec + Catalog → ogun::Graph + Space + Config
│   ├── interpret.rs            # ogun::Layout → CityLayout
│   ├── erosion.rs              # Functional erosion (cascade, material, severity)
│   ├── hierarchy.rs            # Multi-scale generation (district → block → building)
│   ├── output.rs               # TileMap, SemanticGrid output formats
│   └── config.rs               # Serializable generation config
├── data/
│   └── templates/              # Default building templates (JSON/RON)
└── docs/
```

---

## Generation Flow

```
1. Load CitySpec + AgentCatalog

2. If hierarchical:
   a. Generate district layout (ogun, coarse scale)
   b. For each district: generate block layout (ogun, medium scale)
   c. For each block: generate building layout (ogun, fine scale)
   d. Merge into single CityLayout

3. If flat (single scale):
   a. translate(spec, catalog) → ogun inputs
   b. ogun::generate(graph, space, config) → abstract layout
   c. interpret(layout, catalog) → CityLayout

4. If erosion requested:
   a. Apply functional erosion cascade to CityLayout
   b. Update metadata (connectivity, accessibility scores)

5. Return CityLayout
```

---

## Dependencies

```toml
[dependencies]
ogun = { path = "../ogun" }     # or version from crates.io
rand = "0.9"
rand_chacha = "0.9"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

---

## Open Questions

1. **terrain-forge integration** — Should Oku implement terrain-forge's `Algorithm` trait? This would let it plug into terrain-forge's pipeline system. Tradeoff: constrains output to terrain-forge's grid model.

2. **Data format** — JSON (matching Saltglass Steppe) or RON (more Rust-idiomatic) for building templates?

3. **Hierarchical vs flat** — Is hierarchical generation essential for v0.1, or can we start flat and add hierarchy later?

4. **Erosion scope** — Is erosion part of Oku's core, or should it be a separate crate/module that operates on any CityLayout?

5. **Saltglass coupling** — How tightly should Oku couple to Saltglass Steppe's data formats? The glass storm erosion is very game-specific. Maybe: core Oku is generic, with a `saltglass` feature flag for game-specific extensions.
