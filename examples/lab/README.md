# Fog Of War Lab

Crate-local standalone lab app for validating the shared `saddle-world-fog-of-war` crate in a real Bevy scene.

## Purpose

- verify that the shared crate updates `Hidden`, `Explored`, and `Visible` states in a live app
- keep one scene that exercises shared-team vision, arc revealers, blockers, and 3D projection together
- expose stable named entities, overlay diagnostics, BRP resources, and screenshot hooks for E2E and manual debugging

## Status

Working

## Run

```bash
cargo run -p saddle-world-fog-of-war-lab
```

## E2E

```bash
cargo run -p saddle-world-fog-of-war-lab --features e2e -- fog_of_war_smoke
cargo run -p saddle-world-fog-of-war-lab --features e2e -- fog_of_war_exploration_memory
cargo run -p saddle-world-fog-of-war-lab --features e2e -- fog_of_war_occlusion
cargo run -p saddle-world-fog-of-war-lab --features e2e -- fog_of_war_team_layers
cargo run -p saddle-world-fog-of-war-lab --features e2e -- fog_of_war_3d_projection
```

## BRP

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

## Notes

- The lab uses explicit top-level names for revealers, blockers, receivers, and the camera so BRP queries stay stable.
- The scene keeps both a main projected receiver and a minimap receiver alive so one map resource proves multiple presentation surfaces.
- Motion can be paused and layer selection can be changed through E2E custom actions, which keeps screenshot checkpoints deterministic.
