## Oku Overview

> v0.2.0 — ~1200 LOC Rust crate wrapping ogun (v0.3.0) with urban domain knowledge.

## Architecture — a linear pipeline:

```
CitySpec + AgentCatalog
    → translate()      (spec.rs, catalog.rs → translate.rs)
    → ogun::generate() (external crate, negotiated routing)
    → interpret()      (interpret.rs)
    → merge_roads()    (interpret.rs, populates Road.serves)
    → erode()          (erosion.rs, optional, material-aware)
    → CityLayout       (output.rs for tilemap/semantic grid)
```

## Key components by role:

- **lib.rs** — Single entry point: `generate(spec, catalog) -> CityLayout`. Orchestrates the full pipeline. Re-exports all public types including `Material` and `ScoreBreakdown`.
- **catalog.rs** — Domain types: `AgentCatalog`, `BuildingTemplate`, `Category` (5 variants), `ConnectionDemand`, `Material` (Stone/Metal/Wood/Glass with durability coefficients).
- **spec.rs** — Generation parameters: `CitySpec` (dimensions, beta, seed, erosion), `CityType`, `Era`.
- **potential.rs** — `InteractionMatrix` built from `InteractionFn { attraction, gap }` per category pair. Gap inflates radii for road-width spacing. Loaded from JSON defaults.
- **arrival.rs** — `order_agents()` with 3 strategies (Priority, Phased, Random) to control placement order.
- **translate.rs** — The bridge: converts domain types into ogun's `Graph + Space + OgunConfig`. Inflates radii by gap, caps same-category edges to K=3, wires `ConnectionDemand` as supplementary edges to nearest match, builds `repulsion_pairs` from gap values.
- **interpret.rs** — Converts ogun's abstract `Layout` back into `CityLayout` with `PlacedBuilding` and `Road`. `merge_roads()` deduplicates cells, thins blobs, rebuilds as connected components, and populates `Road.serves` with adjacent building indices.
- **erosion.rs** — Material-aware post-processing: `material.durability() * 0.4 + accessibility * 0.4 + noise * 0.2`. Stone temples outlast wooden houses.
- **output.rs** — `to_tilemap()` and `to_semantic_grid()` for consuming the layout.
- **hierarchy.rs** — Stub for future multi-scale generation.
- **config.rs** — Reserved.

## Entry points:

- **Library**: `oku::generate()`
- **Examples**: `visualize.rs` (terminal Unicode), `svg.rs` (SVG file output)
- **Benchmarks**: `benches/city_generation.rs` — criterion suite covering 3 scales, 3 erosion severities, output conversion

## Output stats (default 96-building catalog):

- 96 buildings placed, 130 roads, score ~0.25 on 200×200 grid
- 5 categories: Military, Infrastructure, Sacred, Commercial, Residential
- Negotiated routing via ogun 0.3.0 produces emergent road hierarchy

## Dependencies:

- ogun 0.3.0 (spatial layout engine)
- rand + rand_chacha 0.9 (deterministic RNG)
- serde + serde_json 1 (serialization)
- criterion 0.5 (dev, benchmarks)
