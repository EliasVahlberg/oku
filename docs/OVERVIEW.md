## Oku Overview

Oku is a ~1100 LOC Rust crate that wraps ogun (a domain-agnostic spatial layout engine) with urban domain knowledge to
procedurally generate city layouts.

## Architecture — a linear pipeline:

CitySpec + AgentCatalog
    → translate()      (spec.rs, catalog.rs → translate.rs)
    → ogun::generate() (external crate)
    → interpret()      (interpret.rs)
    → merge_roads()    (interpret.rs)
    → erode()          (erosion.rs, optional)
    → CityLayout       (output.rs for tilemap/semantic grid conversion)


## Key components by role:

- lib.rs — Single entry point: generate(spec, catalog) -> CityLayout. Orchestrates the full pipeline.
- catalog.rs — Domain types: AgentCatalog, BuildingTemplate, Category (Residential/Commercial/Sacred/Military/Infrastructure),
ConnectionDemand.
- spec.rs — Generation parameters: CitySpec (dimensions, beta, seed, erosion), CityType, Era.
- potential.rs — InteractionMatrix built from InteractionFn (attraction + gap per category pair). The gap-based spacing is what
gives buildings road-width separation. Loaded from JSON defaults.
- arrival.rs — order_agents() with 3 strategies (Priority, Phased, Random) to control which buildings get placed first.
- translate.rs — The bridge: converts domain types into ogun's Graph + Space + OgunConfig. Inflates radii by gap, caps same-
category edges to K=3, only creates edges for positive attraction.
- interpret.rs — Highest-scoring file (34.2). Converts ogun's abstract Layout back into CityLayout with PlacedBuildings and Road
s. merge_roads() deduplicates and thins the road network.
- erosion.rs — Post-processing pass that cascades building removal by severity.
- output.rs — to_tilemap() and to_semantic_grid() for consuming the layout.
- config.rs — OkuConfig (beta, seed, repulsion_k) with defaults.
- hierarchy.rs — Stub (7 LOC), placeholder for future multi-scale generation.

## Entry points:

- Library: oku::generate()
- Example: examples/visualize.rs — terminal renderer with colored Unicode, supports --erode flag
- Benchmarks: benches/city_generation.rs — criterion suite covering 3 scales, 3 erosion severities, and output conversion

The crate is thin by design — all algorithmic heavy lifting happens in ogun. Oku owns the domain translation, road post-
processing, erosion, and output formatting.
