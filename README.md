# Saddle World Fog of War

Reusable fog-of-war and visibility runtime for Bevy. The crate keeps gameplay truth on a CPU grid, tracks `Hidden` / `Explored` / `Visible` state per layer, and exposes shader-friendly presentation surfaces for both 2D overlays and 3D ground-plane projection.

It stays project-agnostic: no `game_core`, no screen vocabulary, no lore-specific team types, and no dependency on a specific map renderer. Consumers use the map resource for gameplay truth and opt into the built-in overlay components when they want a ready-made presentation path.

## Quick Start

```rust,no_run
use bevy::prelude::*;
use saddle_world_fog_of_war::{
    FogGridSpec, FogLayerId, FogOfWarConfig, FogOfWarMap, FogOfWarPlugin, FogOverlay2d,
    VisionSource,
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
        .add_plugins(FogOfWarPlugin::new(
            OnEnter(DemoState::Gameplay),
            OnExit(DemoState::Gameplay),
            Update,
        ).with_config(config.clone()))
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

For examples and crate-local labs, `FogOfWarPlugin::default()` is the always-on entrypoint. It activates on `PostStartup`, never deactivates, and updates in `Update`.

## Public API

- `FogOfWarPlugin`: shared-crate plugin with injectable activate, deactivate, and update schedules.
- `FogOfWarSystems`: public ordering hooks for `CollectVisionSources`, `ComputeVisibility`, `UpdateExplorationMemory`, and `UploadRenderData`.
- `FogOfWarConfig`, `FogGridSpec`, `FogOcclusionMode`, `FogWorldAxes`: top-level tuning surface for world-to-cell mapping and LOS mode.
- `FogLayerId`, `FogLayerMask`, `FogVisibilityState`, `FogChunkCoord`: reusable grid and layer vocabulary.
- `FogOfWarMap`: query surface for gameplay and tools. Use `visibility_at_world_pos`, `visibility_at_cell`, `is_visible`, `is_explored`, `iter_visible_cells`, and `iter_explored_cells`.
- `VisionSource`, `VisionOccluder`, `FogRevealShape`, `FogOccluderShape`: ECS inputs for revealers and blockers.
- `FogOverlay2d`, `FogProjectionReceiver`, `FogPalette`: built-in presentation components for 2D and 3D consumers.
- `VisibilityMapUpdated`: batched message emitted when one or more chunks change in a layer.
- `FogOfWarStats`, `FogOfWarRenderAssets`: runtime diagnostics and generated `Image` handles per layer.

## Configuration Summary

- `FogGridSpec` controls origin, dimensions, cell size, and dirty-chunk addressing.
- `FogLayerId` uses a `u64` bitmask backend, so valid layer indices are `0..=63`.
- `FogOcclusionMode::Disabled` skips LOS and treats reveal shapes as pure area fill.
- `FogOcclusionMode::Bresenham` uses per-cell Bresenham LOS against blocker cells.
- `FogWorldAxes::XY` maps `GlobalTransform.translation.xy()` onto the grid.
- `FogWorldAxes::XZ` maps `translation.xz()` onto the grid for projected 3D ground planes.

Full field-by-field guidance lives in [docs/configuration.md](docs/configuration.md).

## Examples

| Example | Purpose | Run |
| --- | --- | --- |
| `basic_2d` | Minimal room-and-corridor style overlay with one moving revealer and exploration memory | `cargo run -p saddle-world-fog-of-war-example-basic-2d` |
| `occlusion_2d` | Blocker-driven LOS example showing hidden cells behind walls | `cargo run -p saddle-world-fog-of-war-example-occlusion-2d` |
| `rts_large_map` | Larger shared-layer shroud with many revealers and minimap reuse | `cargo run -p saddle-world-fog-of-war-example-rts-large-map` |
| `projected_3d` | Same CPU truth rendered on a 3D ground-plane receiver | `cargo run -p saddle-world-fog-of-war-example-projected-3d` |
| `vision_cones` | Arc-based revealers proving the crate is not limited to circles | `cargo run -p saddle-world-fog-of-war-example-vision-cones` |
| `saddle-world-fog-of-war-lab` | Crate-local lab with BRP and E2E hooks | `cargo run -p saddle-world-fog-of-war-lab` |

## Crate-Local Lab

The workspace includes a richer lab app at `shared/world/saddle-world-fog-of-war/examples/lab`:

```bash
cargo run -p saddle-world-fog-of-war-lab
```

E2E verification commands:

```bash
cargo run -p saddle-world-fog-of-war-lab --features e2e -- fog_of_war_smoke
cargo run -p saddle-world-fog-of-war-lab --features e2e -- fog_of_war_exploration_memory
cargo run -p saddle-world-fog-of-war-lab --features e2e -- fog_of_war_occlusion
cargo run -p saddle-world-fog-of-war-lab --features e2e -- fog_of_war_team_layers
cargo run -p saddle-world-fog-of-war-lab --features e2e -- fog_of_war_3d_projection
```

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
- Dirty chunks are tracked for messages and diagnostics, but the built-in render upload still rewrites each active layer texture in full.
- There is no persistence snapshot API in v1. Consumers that need save/load should serialize their own explored-cell data from `FogOfWarMap`.
- The built-in rendering path focuses on reusable overlays, not concealment policy. Consumers still decide when hidden or explored units should stop rendering.

## More Docs

- [Architecture](docs/architecture.md)
- [Configuration](docs/configuration.md)
- [Performance](docs/performance.md)
