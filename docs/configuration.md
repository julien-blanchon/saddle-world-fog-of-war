# Configuration

This document covers the main public tuning surfaces. Defaults describe the built-in constructors and `Default` impls shipped in v1.

## `FogOfWarConfig`

| Field | Type | Default | Valid range | Perf impact | Gameplay effect | Rendering effect |
| --- | --- | --- | --- | --- | --- | --- |
| `grid` | `FogGridSpec` | `origin = Vec2::ZERO`, `dimensions = 64x64`, `cell_size = 1x1`, `chunk_size = 16x16` | positive dimensions and positive cell size | dominant driver of memory and upload cost | defines the discretized world, dirty-chunk granularity, and query bounds | determines the texture size and overlay scaling |
| `occlusion_mode` | `FogOcclusionMode` | `Bresenham` | `Disabled` or `Bresenham` | `Bresenham` adds LOS work per candidate cell | decides whether blockers matter for visibility truth | changes the visible silhouette and wall shadowing in overlays |
| `world_axes` | `FogWorldAxes` | `XY` | `XY` or `XZ` | negligible | decides which transform plane maps into fog cells | keeps 2D overlays and 3D ground-plane receivers aligned to the same truth |
| `persistence_mode` | `FogPersistenceMode` | `ExploredMemory` | `NoMemory`, `ExploredMemory`, or `Custom` | `NoMemory` is cheapest; custom depends on your policy | decides how current visibility becomes the committed state surface | controls whether remembered exploration survives after vision leaves |

## `FogPersistenceMode`

| Variant | Default | Perf impact | Gameplay effect | Rendering effect |
| --- | --- | --- | --- | --- |
| `NoMemory` | no | cheapest built-in mode | cells return to `Hidden` as soon as they leave current vision | overlays and receivers show only current sight |
| `ExploredMemory` | yes | same cost profile as v1 | cells degrade from `Visible` to `Explored` and never return to hidden automatically | preserves a classic RTS/roguelike explored shroud |
| `Custom` | no | depends on your policy | lets a `FogPersistencePolicy` decide the committed state per cell | built-in renderer consumes whatever `Hidden` / `Explored` / `Visible` states your policy emits |

## `FogPersistencePolicy`

Custom persistence policies receive a `FogPersistenceCell` containing:

- `layer`
- `cell`
- `visible_now`
- `previous_state`

Implement the trait and pass it through `FogOfWarPlugin::with_custom_persistence(...)` when you need non-default commit rules.

## `FogGridSpec`

| Field | Type | Default | Valid range | Perf impact | Gameplay effect | Rendering effect |
| --- | --- | --- | --- | --- | --- | --- |
| `origin` | `Vec2` | `Vec2::ZERO` | any finite value | negligible | shifts the fog grid relative to world coordinates | shifts overlay/projection alignment |
| `dimensions` | `UVec2` | `64 x 64` | each axis `>= 1` | scales map memory, commit cost, and full-texture upload size | sets the playable fog bounds | sets texture resolution per active layer |
| `cell_size` | `Vec2` | `1 x 1` | clamped to at least `0.001` by constructors | larger cells reduce total cell count; smaller cells increase it | controls spatial precision and LOS granularity | affects how sharp or coarse the overlay looks |
| `chunk_size` | `UVec2` | `16 x 16` | each axis `>= 1` | affects dirty-chunk counts and downstream batching, not full recompute cost in v1 | no truth change by itself | useful for custom consumers that want chunk-level minimap or network updates |

## Layers

| Type | Default | Valid range | Perf impact | Gameplay effect | Rendering effect |
| --- | --- | --- | --- | --- | --- |
| `FogLayerId(pub u8)` | `FogLayerId::ZERO` | `0..=63` | more active layers mean more per-layer storage and uploads | selects which team, faction, sensor net, or floor receives vision | receivers choose one target layer at a time |
| `FogLayerMask(pub u64)` | `FogLayerMask::EMPTY` | bitset across the same `0..=63` layer range | negligible | lets blockers affect one or many layers | indirectly changes visible silhouettes by layer |

## `VisionSource`

| Field | Type | Default | Valid range | Perf impact | Gameplay effect | Rendering effect |
| --- | --- | --- | --- | --- | --- | --- |
| `layer` | `FogLayerId` | constructor-provided | `0..=63` | active layers increase storage | decides which layer becomes visible | determines which receiver texture updates |
| `shared_layers` | `FogLayerMask` | `EMPTY` | any layer bitset | negligible | mirrors the same reveal footprint into additional layers | lets one revealer drive multiple layer textures |
| `shape` | `FogRevealShape` | constructor-provided | see shape table below | larger shapes touch more candidate cells | controls reveal footprint | controls fog holes and silhouette size |
| `offset` | `Vec2` | `Vec2::ZERO` | any finite value | negligible | lets the reveal origin sit away from the entity pivot | keeps projected visuals centered on the intended reveal point |
| `enabled` | `bool` | `true` | `true` or `false` | disabled sources are skipped entirely | turns a revealer on or off without despawning | removes or restores the hole in the overlay |

## `VisionCellSource`

| Field | Type | Default | Valid range | Perf impact | Gameplay effect | Rendering effect |
| --- | --- | --- | --- | --- | --- | --- |
| `layers` | `FogLayerMask` | constructor-provided | any layer bitset | scales with number of cells published and target layers touched | injects exact visible cells without using a geometric reveal shape | updates the same layer textures as shape-based revealers |
| `cells` | `Vec<IVec2>` | empty | in-bounds grid cells | proportional to the number of cells copied each update | ideal for bridge systems such as FOV, tactical sensors, or server-authoritative vision | reveals exactly the supplied cells, preserving upstream visibility semantics |
| `enabled` | `bool` | `true` | `true` or `false` | disabled sources are skipped entirely | turns exact-cell reveal on or off without despawning | removes or restores those revealed cells from presentation |

## `FogRevealShape`

| Variant | Fields | Default / constructor | Perf impact | Gameplay effect | Rendering effect |
| --- | --- | --- | --- | --- | --- |
| `Circle` | `radius: f32` | `FogRevealShape::circle(radius)` | candidate area grows with `radius^2` | radial sight or sensor coverage | round reveal hole |
| `Arc` | `radius: f32`, `angle_radians: f32`, `facing: Vec2` | `FogRevealShape::arc(...)` | similar to circle plus angle check per candidate | directional cones, sentries, stealth sensors | wedge-shaped reveal |
| `Rect` | `half_extents: Vec2` | `FogRevealShape::rect(...)` | scales with covered rectangle area | corridors, scanners, rectangular sensors | box-shaped reveal |

Guidance:

- keep radii and half extents positive
- normalize `facing` when you care about exact cone direction
- use arcs when you need directional stealth or watchtower logic without adding a custom shape type
- use `shared_layers` when the same revealer should update allied team vision or a mirrored minimap layer

## `VisionOccluder`

| Field | Type | Default | Valid range | Perf impact | Gameplay effect | Rendering effect |
| --- | --- | --- | --- | --- | --- | --- |
| `layers` | `FogLayerMask` | constructor-provided | any layer bitset | negligible | decides which layers the blocker affects | only layers in the mask gain shadowed regions |
| `shape` | `FogOccluderShape` | constructor-provided | see shape table below | larger blockers touch more blocker cells | changes LOS and hidden zones | changes shadow silhouettes |
| `offset` | `Vec2` | `Vec2::ZERO` | any finite value | negligible | lets the blocker be centered away from the entity pivot | keeps the rendered wall or prop aligned to the fog blocker |
| `enabled` | `bool` | `true` | `true` or `false` | disabled blockers are skipped entirely | turns LOS blocking on or off without despawning | removes or restores the corresponding fog shadow |

## `FogOccluderShape`

| Variant | Fields | Default / constructor | Perf impact | Gameplay effect | Rendering effect |
| --- | --- | --- | --- | --- | --- |
| `Cell` | none | `VisionOccluder::cell(mask)` | cheapest | single blocker cell | single-cell occlusion |
| `Circle` | `radius: f32` | `VisionOccluder::circle(mask, radius)` | scales with covered area | round pillars or dense foliage clumps | rounded LOS shadow |
| `Rect` | `half_extents: Vec2` | `VisionOccluder::rect(mask, half_extents)` | scales with covered area | walls, cover strips, buildings | rectangular LOS shadow |

## `FogOverlay2d`

`FogOverlay2d` is only rendered when `FogOfWarRenderingPlugin` is enabled.

| Field | Type | Default from `FogOverlay2d::new` | Valid range | Perf impact | Gameplay effect | Rendering effect |
| --- | --- | --- | --- | --- | --- | --- |
| `layer` | `FogLayerId` | constructor-provided | `0..=63` | one active layer texture per presented layer | none directly | chooses which layer texture is shown |
| `world_origin` | `Vec2` | constructor-provided | any finite value | negligible | none | positions the quad in world space |
| `world_size` | `Vec2` | constructor-provided | positive extents | negligible | none | scales the quad and UV mapping |
| `palette` | `FogPalette` | `FogPalette::default()` | any `LinearRgba` colors | negligible | none | sets hidden/explored/visible colors |
| `opacity` | `f32` | `1.0` | typically `0.0..=1.0` | negligible | none | global alpha multiplier |
| `edge_softness` | `f32` | `0.2` | typically `0.0..=1.0` | negligible | none | softens transitions between discrete state values in the material |
| `z` | `f32` | `20.0` | any finite value | negligible | none | draw order / depth within 2D scenes |

## `FogProjectionReceiver`

`FogProjectionReceiver` is only rendered when `FogOfWarRenderingPlugin` is enabled.

| Field | Type | Default from `FogProjectionReceiver::new` | Valid range | Perf impact | Gameplay effect | Rendering effect |
| --- | --- | --- | --- | --- | --- | --- |
| `layer` | `FogLayerId` | constructor-provided | `0..=63` | one active layer texture per presented layer | none directly | chooses which layer texture is shown |
| `world_origin` | `Vec2` | constructor-provided | any finite value | negligible | none | positions the projected plane on the ground |
| `world_size` | `Vec2` | constructor-provided | positive extents | negligible | none | scales the projected plane and UV mapping |
| `palette` | `FogPalette` | `FogPalette::default()` | any `LinearRgba` colors | negligible | none | sets hidden/explored/visible colors |
| `opacity` | `f32` | `1.0` | typically `0.0..=1.0` | negligible | none | global alpha multiplier |
| `edge_softness` | `f32` | `0.25` | typically `0.0..=1.0` | negligible | none | softens state transitions on the projected receiver |
| `elevation` | `f32` | `0.03` | any finite value | negligible | none | lifts the plane off the ground to avoid z-fighting |

## `FogPalette`

| Field | Type | Default | Valid range | Perf impact | Gameplay effect | Rendering effect |
| --- | --- | --- | --- | --- | --- | --- |
| `hidden` | `LinearRgba` | `FogPalette::grayscale().hidden` | any color | negligible | none | color used for never-seen cells |
| `explored` | `LinearRgba` | `FogPalette::grayscale().explored` | any color | negligible | none | color used for explored but not currently visible cells |
| `visible` | `LinearRgba` | transparent black | any color | negligible | none | color blended over currently visible cells, often fully transparent |

Preset helpers:

- `FogPalette::grayscale()`
- `FogPalette::cinematic()`

## Runtime Resources

| Resource | Purpose | Notes |
| --- | --- | --- |
| `FogOfWarMap` | gameplay-truth and committed-state query surface | use `is_visible` / `current_visibility_at_cell` for current truth, and `visibility_at_cell` / `iter_explored_cells` for remembered state |
| `FogOfWarStats` | per-frame counts and microsecond timings | reflected for BRP inspection |
| `FogOfWarRenderAssets` | per-layer `Handle<Image>` output | only initialized by `FogOfWarRenderingPlugin`; lets custom materials or UI reuse the generated textures |

## Practical Tuning Advice

- Start with `cell_size = 1.0` for tactics or projected 3D scenes and `cell_size = 16..64` for coarse 2D shrouds.
- Prefer `chunk_size = 8..32` when downstream systems consume dirty updates. It has little effect on the built-in renderer in v1.
- Use `world_axes = XZ` whenever revealers move on the 3D ground plane. This avoids mixing camera height into the fog truth.
- Leave `occlusion_mode = Disabled` for terrain shrouds where walls do not matter, then switch to `Bresenham` when blockers become part of gameplay.
- Use `NoMemory` when the fog should represent only present line-of-sight, and `ExploredMemory` when discovered terrain should remain revealed.
- Keep `Custom` for games that need non-default degradation rules; the core grid and LOS systems stay unchanged while only the persistence step changes.
