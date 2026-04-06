# Performance

## Cost Model

`saddle-world-fog-of-war` is CPU-truth first. The dominant costs in v1 are:

1. blocker rasterization area
2. candidate cells covered by each reveal shape
3. LOS checks per candidate cell when `Bresenham` occlusion is enabled
4. per-layer persistence commit across the whole grid
5. optional full-layer texture upload for every active rendered layer

The runtime is fast on small and medium grids, but its cost still scales with both map size and revealer count because v1 recomputes visibility from current sources each frame.

## What Scales With World Size

- `FogOfWarMap` storage:
  - one `FogVisibilityState` per cell per active layer
  - one `bool` current-visibility slot per cell per active layer
  - one `u16` visible-count slot per cell per active layer
  - one blocker mask per cell shared across layers
- commit cost:
  - walks every cell in every active layer each update
- upload cost:
  - rewrites the full `R8Unorm` texture for every active rendered layer

Increasing `dimensions` is the most important cost lever.

## What Scales With Revealer Count

- `collect_inputs`: linear in active sources and blockers
- `accumulate_visibility`: roughly linear in total candidate cells covered by all revealers
- `Bresenham` LOS: candidate count multiplied by average ray length
- `ApplyPersistence`: linear in active layer cell count; `NoMemory` is the cheapest built-in mode because it does not preserve explored state

Many overlapping revealers are still merged correctly, but they are not free. If dozens of sources cover large radii, the visibility pass will dominate before the upload pass does.

## What Chunking Helps Today

`FogGridSpec::chunk_size` affects:

- `VisibilityMapUpdated.dirty_chunks`
- diagnostics such as `dirty_chunk_count`
- downstream batching for minimaps, network sync, or custom renderers

`chunk_size` does **not** currently reduce:

- full-layer visibility recompute cost
- full-layer persistence commit cost
- full-layer texture upload cost when the optional rendering plugin is enabled
- in-memory layer storage size

Use chunking in v1 for integration and dirty-region reporting, not as a substitute for true streaming or sparse storage.

## Observed Debug-Run Numbers

These measurements were collected on March 30, 2026 on an Apple M4 Max in a debug build.

Headless CPU profiles from `cargo test -p saddle-world-fog-of-war log_visibility_profiles -- --nocapture`:

| Profile | Map | Sources | Occluders | Compute | Upload | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| `small_lab_like` | `24 x 18` | `3` | `2` | `17 us` | `0 us` | headless `MinimalPlugins`, mirrors the crate-local lab scale |
| `large_rts_like` | `96 x 64` | `15` | `5` | `62 us` | `0 us` | larger shared-layer shroud with many revealers |
| `blocker_heavy` | `64 x 64` | `2` | `16` | `60 us` | `0 us` | blocker-dense LOS case |

Rendered BRP probe from the live lab after the scene settled:

| Scene | Map | Sources | Occluders | Compute | Upload | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| `saddle-world-fog-of-war-lab` live sample | `24 x 18` | `3` | `2` | `15 us` | `1 us` | queried through `FogOfWarStats` over BRP after the scene settled |

These numbers are useful for relative intuition, not as contractual thresholds.

## Memory Assumptions

Per active layer, v1 stores:

- one visibility-state vector
- one visible-count vector
- one dirty-chunk set

Shared across all layers, it stores:

- one blocker-mask vector

Approximate intuition:

- large layers are primarily cell-count driven
- many active layers cost roughly linearly more memory
- render textures add one `R8Unorm` image per active presented layer

If your game keeps many layers live at once, layer count will matter almost as much as map size.

## When To Use This Crate As-Is

Good fit:

- roguelike maps
- tactics boards
- isometric or top-down 3D scenes with projected ground fog
- RTS minimaps or terrain shrouds with modest map sizes

Proceed carefully:

- very large worlds with many simultaneously active layers
- hundreds of large-radius revealers
- cases that require sparse storage or true chunk eviction

## Likely Next Optimizations

If a project outgrows the v1 path, the most natural next steps are:

1. source-movement thresholds or cadence throttling
2. partial texture uploads using dirty chunks
3. sparse or streamed layer storage
4. a shadowcasting core for strictly tile-based games
5. an optional GPU accumulation path for very large RTS-style workloads

The current public API was chosen so these upgrades can happen behind the same visibility core while keeping persistence policy and rendering concerns split.
