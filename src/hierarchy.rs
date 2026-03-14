//! Hierarchical generation — multi-scale city layout (district → block → building).

// TODO: Orchestrate multiple ogun::generate() calls at different scales.
// 1. Generate district layout (coarse grid, large nodes)
// 2. For each district: generate block layout (medium grid)
// 3. For each block: generate building layout (fine grid)
// 4. Merge into single CityLayout
