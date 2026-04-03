# Architecture

## Layering

`saddle-world-fog-of-war` is split into three layers:

1. Pure visibility core:
   - `grid.rs`
   - `math.rs`
   - `visibility.rs`
2. ECS integration:
   - `components.rs`
   - `resources.rs`
   - `messages.rs`
   - `systems.rs`
3. Rendering:
   - `rendering/material_2d.rs`
   - `rendering/material_3d.rs`
   - `rendering/upload.rs`

The core layer owns correctness. The ECS layer only gathers world inputs, updates resources, and emits messages. The rendering layer derives textures and materials from the CPU truth but never becomes the source of truth itself.

## Runtime Flow

```text
VisionSource / VisionCellSource / VisionOccluder components
  -> collect_inputs
  -> rebuild blocker grid
  -> rasterize reveal shapes or apply exact visible cells
  -> apply LOS per candidate cell
  -> commit Visible / Explored / Hidden states
  -> emit VisibilityMapUpdated messages
  -> upload per-layer R8 textures
  -> sync 2D overlay or 3D receiver materials
```

## Schedule Ordering

`FogOfWarSystems` is intentionally public and chained in this order:

1. `CollectVisionSources`
2. `ComputeVisibility`
3. `UpdateExplorationMemory`
4. `UploadRenderData`

That guarantees:

- world transforms and source components are sampled before visibility is computed
- `FogOfWarMap` is stable before messages are emitted
- render assets only observe committed state, never half-updated intermediate buffers
- cross-crate bridge systems can safely run after another visibility system and before `CollectVisionSources`

The plugin accepts injectable activate, deactivate, and update schedules so consumers can map the runtime into their own state machine.

## Algorithm Choice

`saddle-world-fog-of-war` uses a CPU grid as gameplay truth and a shader/material path for presentation.

What v1 does:

- rasterizes each reveal shape into a candidate cell rectangle
- can also accept exact visible cells through `VisionCellSource`
- filters candidates by shape containment
- runs Bresenham LOS from the source cell to each candidate cell when occlusion is enabled
- merges overlapping revealers by incrementing per-cell visible counts
- converts no-longer-visible cells from `Visible` to `Explored`

Why this baseline was chosen:

- easier to test than a GPU-first design
- easy to inspect over BRP and unit tests
- works for both small roguelike maps and projected 3D minimap-style overlays
- keeps the public data model stable if a future compute path is added

References adopted in this design:

- Red Blob Games and Adam Milazzo informed the visibility semantics and LOS tradeoffs
- Brendan Keesing informed the CPU-truth plus presentation-texture split
- official Bevy shader material examples informed the 2D and 3D receiver material structure

What v1 does not do:

- recursive or symmetric shadowcasting
- partial GPU texture uploads
- volumetric or multi-level visibility
- compute-shader reveal accumulation

## Storage Model

`FogOfWarMap` stores one monolithic cell array per active layer:

- `states: Vec<FogVisibilityState>`
- `visible_counts: Vec<u16>`
- `dirty_chunks: HashSet<UVec2>`

Chunking is used for change reporting and integration, not for storage eviction. `FogGridSpec::chunk_size` controls how `VisibilityMapUpdated` batches dirty work for consumers such as minimaps, networking layers, or custom renderers.

## Teams And Layers

- `FogLayerId(pub u8)` selects the logical visibility layer.
- `FogLayerMask(pub u64)` lets blockers affect one layer or many layers at once.
- `VisionSource::shared_layers` duplicates one revealer into additional layers without spawning duplicate entities.
- valid layer indices are `0..=63` because the mask backend is a `u64`.

This keeps the API generic. One game can treat layers as factions, another as sensor networks, and another as floor-specific viewers.

## World Axes

`FogWorldAxes` decouples the CPU truth from one render path:

- `XY`: 2D worlds, tilemaps, and UI-like orthographic spaces
- `XZ`: top-down 3D or isometric scenes where gameplay happens on the ground plane

This is why the same `FogOfWarMap` can drive both the 2D examples and the 3D projected lab.

## Rendering Model

The rendering layer consumes `FogOfWarMap` and produces one `R8Unorm` image per active layer.

- `FogOverlay2d` spawns or updates a `Material2d` quad in world space.
- `FogProjectionReceiver` spawns or updates a `Material` plane in 3D.
- `FogPalette`, opacity, and edge softness affect presentation only.

Important boundary:

- the CPU map stores discrete truth
- smoothing and palette choices happen only in the material path

This prevents visual polish from leaking into gameplay semantics.

## Headless And MinimalPlugins Behavior

The crate checks for `RenderApp` before registering shader assets or render-side resources.

That means:

- `DefaultPlugins` gets the full overlay/projection path
- `MinimalPlugins` keeps the CPU truth, messages, and stats without panicking

This split is what allows unit tests and perf probes to run headlessly.

## Verification Strategy

The crate verifies each layer separately:

- unit tests for grid conversion, state transitions, overlap behavior, LOS, arcs, and chunk addressing
- Bevy app tests for activation, collection, message emission, deactivation, and `XZ` projection
- standalone examples for focused 2D, occlusion, RTS-style, 3D, and cone-based usage
- crate-local E2E scenarios for smoke, exploration memory, occlusion, layer switching, and projected 3D alignment
- BRP checks for named entities, reflected config/stats resources, and screenshot capture
