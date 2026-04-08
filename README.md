# Saddle World Fog of War

Reusable fog-of-war and visibility runtime for Bevy. The crate keeps gameplay truth on a CPU grid, computes current visibility with configurable LOS, and then applies a pluggable persistence policy per layer. Built-in policies cover both `NoMemory` and `ExploredMemory`, while custom policies can override the commit semantics without forking the grid or LOS core.

Rendering is intentionally optional. `FogOfWarPlugin` owns the CPU truth and persistence step, while `FogOfWarRenderingPlugin` turns the committed state surface into shader-friendly textures for 2D overlays or 3D ground-plane projection.

## Quick Start

```rust,no_run
use bevy::prelude::*;
use saddle_world_fog_of_war::{
    FogGridSpec, FogLayerId, FogOfWarConfig, FogOfWarMap, FogOfWarPlugin,
    FogOfWarRenderingPlugin, FogOverlay2d, VisionSource,
};

#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum DemoState {
    #[default]
    Gameplay,
}

fn main() {
    let config = FogOfWarConfig {
        grid: FogGridSpec {
            origin: Vec2::ZERO,
            dimensions: UVec2::new(32, 20),
            cell_size: Vec2::splat(1.0),
            chunk_size: UVec2::splat(8),
        },
        ..default()
    };

    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<DemoState>()
        .add_plugins((
            FogOfWarPlugin::new(
                OnEnter(DemoState::Gameplay),
                OnExit(DemoState::Gameplay),
                Update,
            )
            .with_config(config.clone()),
            FogOfWarRenderingPlugin::default(),
        ))
        .add_systems(Startup, move |mut commands: Commands| {
            commands.spawn((
                Name::new("Observer Camera"),
                Camera2d,
            ));
            commands.spawn((
                Name::new("Observer"),
                VisionSource::circle(FogLayerId::ZERO, 5.0),
                Transform::from_xyz(6.5, 7.5, 0.0),
            ));
            commands.spawn((
                Name::new("Fog Overlay"),
                FogOverlay2d::new(FogLayerId::ZERO, config.grid.origin, config.grid.world_size()),
            ));
        })
        .add_systems(Update, inspect_visibility)
        .run();
}

fn inspect_visibility(map: Res<FogOfWarMap>) {
    if map.is_visible(FogLayerId::ZERO, IVec2::new(6, 7)) {
        // React to current vision here.
    }
}
```

For examples and crate-local labs, `FogOfWarPlugin::default()` is the always-on core runtime entrypoint. It activates on `PostStartup`, never deactivates, and updates in `Update`. Add `FogOfWarRenderingPlugin::default()` when you want the built-in overlay/projection materials and per-layer `Image` output.

## Public API

- `FogOfWarPlugin`: shared-crate core plugin with injectable activate, deactivate, and update schedules.
- `FogOfWarRenderingPlugin`: optional material/upload plugin for `FogOverlay2d`, `FogProjectionReceiver`, and `FogOfWarRenderAssets`.
- `FogOfWarSystems`: public ordering hooks for `CollectVisionSources`, `ComputeVisibility`, `ApplyPersistence`, and `UploadRenderData`.
- `FogOfWarConfig`, `FogGridSpec`, `FogOcclusionMode`, `FogWorldAxes`, `FogPersistenceMode`: top-level tuning surface for world-to-cell mapping, LOS mode, and persistence behavior.
- `FogLayerId`, `FogLayerMask`, `FogVisibilityState`, `FogChunkCoord`: reusable grid and layer vocabulary.
- `FogPersistencePolicy`, `FogPersistenceCell`, `FogCustomPersistence`: custom persistence hook for consumers that need non-default commit behavior.
- `FogOfWarMap`: query surface for gameplay and tools. Use `current_visibility_at_cell` / `is_visible` for current truth, and `visibility_at_world_pos` / `visibility_at_cell` / `iter_explored_cells` for the committed persistence surface.
- `VisionSource`, `VisionCellSource`, `VisionOccluder`, `FogRevealShape`, `FogOccluderShape`: ECS inputs for revealers and blockers.
- `FogOverlay2d`, `FogProjectionReceiver`, `FogPalette`: built-in presentation components for 2D and 3D consumers when the rendering plugin is enabled.
- `VisibilityMapUpdated`: batched message emitted when one or more chunks change in a layer's current-visibility or committed-state surface.
- `FogOfWarStats`, `FogOfWarRenderAssets`: runtime diagnostics and generated `Image` handles per layer. `FogOfWarRenderAssets` is only populated by the rendering plugin.

## Configuration Summary

- `FogGridSpec` controls origin, dimensions, cell size, and dirty-chunk addressing.
- `FogLayerId` uses a `u64` bitmask backend, so valid layer indices are `0..=63`.
- `FogOcclusionMode::Disabled` skips LOS and treats reveal shapes as pure area fill.
- `FogOcclusionMode::Bresenham` uses per-cell Bresenham LOS against blocker cells.
- `FogWorldAxes::XY` maps `GlobalTransform.translation.xy()` onto the grid.
- `FogWorldAxes::XZ` maps `translation.xz()` onto the grid for projected 3D ground planes.
- `FogPersistenceMode::NoMemory` clears committed state when vision leaves.
- `FogPersistenceMode::ExploredMemory` keeps the classic `Visible -> Explored -> Hidden never` behavior.
- `FogPersistenceMode::Custom` delegates the commit step to `FogCustomPersistence`.
- `VisionSource::with_shared_layers(...)` lets one revealer feed allied or mirrored fog layers.
- `VisionCellSource` lets other systems publish exact visible cells directly into the fog runtime without rasterizing a reveal shape.

Full field-by-field guidance lives in [docs/configuration.md](docs/configuration.md).

## Examples

| Example | Purpose | Run | E2E |
| --- | --- | --- | --- |
| `basic_2d` | Minimal room-and-corridor style overlay with one moving revealer and exploration memory | `cargo run -p saddle-world-fog-of-war-example-basic-2d` | `cargo run -p saddle-world-fog-of-war-example-basic-2d --features e2e -- basic_2d_memory_trail` |
| `occlusion_2d` | Blocker-driven LOS example showing hidden cells behind walls | `cargo run -p saddle-world-fog-of-war-example-occlusion-2d` | `cargo run -p saddle-world-fog-of-war-example-occlusion-2d --features e2e -- occlusion_2d_wall_shadow` |
| `rts_large_map` | Larger shared-layer shroud with many revealers and minimap reuse | `cargo run -p saddle-world-fog-of-war-example-rts-large-map` | `cargo run -p saddle-world-fog-of-war-example-rts-large-map --features e2e -- rts_large_map_multi_overlay` |
| `projected_3d` | Same CPU truth rendered on a 3D ground-plane receiver | `cargo run -p saddle-world-fog-of-war-example-projected-3d` | `cargo run -p saddle-world-fog-of-war-example-projected-3d --features e2e -- projected_3d_projection_orbit` |
| `vision_cones` | Arc-based revealers proving the crate is not limited to circles | `cargo run -p saddle-world-fog-of-war-example-vision-cones` | `cargo run -p saddle-world-fog-of-war-example-vision-cones --features e2e -- vision_cones_directional_arc` |
| `fov_integration` | Cross-crate demo where `saddle-ai-fov` feeds exact visible cells into fog-of-war memory | `cargo run -p saddle-world-fog-of-war-example-fov-integration` | `cargo run -p saddle-world-fog-of-war-example-fov-integration --features e2e -- fov_integration_bridge` |
| `saddle-world-fog-of-war-lab` | Crate-local integration lab with BRP and multi-surface E2E hooks | `cargo run -p saddle-world-fog-of-war-lab` | see commands below |

## Crate-Local Lab

The workspace includes a richer lab app at `crates/world/saddle-world-fog-of-war/examples/lab`:

```bash
cargo run -p saddle-world-fog-of-war-lab
```

E2E verification commands:

```bash
cargo run -p saddle-world-fog-of-war-lab --features e2e -- fog_of_war_smoke
cargo run -p saddle-world-fog-of-war-lab --features e2e -- fog_of_war_exploration_memory
cargo run -p saddle-world-fog-of-war-lab --features e2e -- fog_of_war_no_memory
cargo run -p saddle-world-fog-of-war-lab --features e2e -- fog_of_war_occlusion
cargo run -p saddle-world-fog-of-war-lab --features e2e -- fog_of_war_team_layers
cargo run -p saddle-world-fog-of-war-lab --features e2e -- fog_of_war_3d_projection
```

The focused example scenarios above verify each showcase in isolation. The lab stays valuable as the broader integration check that combines 3D projection, multiple layers, shared controls, and BRP inspection in one scene.

## BRP

Useful BRP commands against the lab:

```bash
uv run --project .codex/skills/bevy-brp/script brp app launch saddle-world-fog-of-war-lab
uv run --project .codex/skills/bevy-brp/script brp world query bevy_ecs::name::Name
uv run --project .codex/skills/bevy-brp/script brp world query saddle_world_fog_of_war::components::VisionSource
uv run --project .codex/skills/bevy-brp/script brp world query saddle_world_fog_of_war::components::VisionOccluder
uv run --project .codex/skills/bevy-brp/script brp resource get saddle_world_fog_of_war::resources::FogOfWarConfig
uv run --project .codex/skills/bevy-brp/script brp resource get saddle_world_fog_of_war::resources::FogOfWarStats
uv run --project .codex/skills/bevy-brp/script brp extras screenshot /tmp/fog_of_war_lab.png
uv run --project .codex/skills/bevy-brp/script brp extras shutdown
```

## Limitations And Non-Goals

- Gameplay truth is a 2D grid projected onto either the `XY` or `XZ` plane. There is no volumetric or multi-floor visibility model in v1.
- The occlusion model is intentionally simple: disabled or Bresenham LOS through blocker cells. Recursive shadowcasting and GPU compute are not included yet.
- Dirty chunks are tracked for messages and diagnostics, but the optional render upload still rewrites each active layer texture in full.
- There is no persistence snapshot API in v1. Consumers that need save/load should serialize their own explored-cell data from `FogOfWarMap`.
- The built-in rendering path is an optional consumer of the committed state surface, not part of the core visibility runtime. Consumers still decide when hidden or explored units should stop rendering.

## More Docs

- [Architecture](docs/architecture.md)
- [Configuration](docs/configuration.md)
- [Performance](docs/performance.md)
