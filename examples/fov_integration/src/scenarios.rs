use std::collections::HashSet;

use bevy::prelude::*;
use saddle_ai_fov::{GridFov, GridFovState};
use saddle_world_fog_of_war::{FogLayerId, FogOfWarMap, VisionCellSource};
use saddle_world_fog_of_war_example_support::e2e_support::{Action, Scenario, assertions};

use crate::{IntegrationPane, ReconScout};

#[derive(Resource, Default, Clone, Copy)]
struct VisibleSnapshot(usize);

#[derive(Resource, Default, Clone, Copy)]
struct BridgeSnapshot {
    matches: bool,
    visible_count: usize,
    radius: i32,
}

pub fn list() -> Vec<&'static str> {
    vec!["fov_integration_bridge"]
}

pub fn by_name(name: &str) -> Option<Scenario> {
    match name {
        "fov_integration_bridge" => Some(fov_integration_bridge()),
        _ => None,
    }
}

fn set_pause(paused: bool) -> Action {
    Action::Custom(Box::new(move |world| {
        world.resource_mut::<IntegrationPane>().pause_motion = paused;
    }))
}

fn set_radius(radius: i32) -> Action {
    Action::Custom(Box::new(move |world| {
        world.resource_mut::<IntegrationPane>().scout_radius = radius;
    }))
}

fn set_edge_softness(edge_softness: f32) -> Action {
    Action::Custom(Box::new(move |world| {
        world.resource_mut::<IntegrationPane>().edge_softness = edge_softness;
    }))
}

fn place_scout(cell: IVec2) -> Action {
    Action::Custom(Box::new(move |world| {
        let Some(position) = world
            .resource::<saddle_ai_fov::GridOpacityMap>()
            .spec
            .cell_to_world_center(cell)
        else {
            return;
        };
        let mut query =
            world.query_filtered::<(&mut Transform, &mut GlobalTransform), With<ReconScout>>();
        if let Ok((mut transform, mut global)) = query.single_mut(world) {
            let translation = position.extend(5.0);
            transform.translation = translation;
            *global = GlobalTransform::from_translation(translation);
        }
    }))
}

fn snapshot_visible_cells() -> Action {
    Action::Custom(Box::new(|world| {
        let mut query = world.query_filtered::<&GridFovState, With<ReconScout>>();
        let visible = query
            .single(world)
            .map_or(0, |state| state.visible_now.len());
        world.insert_resource(VisibleSnapshot(visible));
    }))
}

fn snapshot_bridge_state() -> Action {
    Action::Custom(Box::new(|world| {
        let mut query = world
            .query_filtered::<(&GridFovState, &VisionCellSource, &GridFov), With<ReconScout>>();
        let snapshot = if let Ok((fov_state, fog_source, fov)) = query.single(world) {
            let visible: HashSet<_> = fov_state.visible_now.iter().copied().collect();
            let bridged: HashSet<_> = fog_source.cells.iter().copied().collect();
            BridgeSnapshot {
                matches: !visible.is_empty() && visible == bridged,
                visible_count: visible.len(),
                radius: fov.config.radius,
            }
        } else {
            BridgeSnapshot::default()
        };
        world.insert_resource(snapshot);
    }))
}

fn fov_integration_bridge() -> Scenario {
    Scenario::builder("fov_integration_bridge")
        .description("Verify the FOV output is copied into fog cell sources, let the scout explore the dungeon, then widen the FOV radius and confirm the bridge updates the fog footprint.")
        .then(Action::WaitFrames(30))
        .then(snapshot_bridge_state())
        .then(assertions::custom("FOV cells are bridged into fog cell input", |world| {
            world.resource::<BridgeSnapshot>().matches
        }))
        .then(Action::Screenshot("fov_bridge_start".into()))
        .then(Action::WaitFrames(120))
        .then(assertions::custom("fog memory accumulates explored cells", |world| {
            world
                .resource::<FogOfWarMap>()
                .layer_summary(FogLayerId(0))
                .is_some_and(|summary| summary.explored_cells > summary.visible_cells)
        }))
        .then(set_pause(true))
        .then(place_scout(IVec2::new(12, 9)))
        .then(Action::WaitFrames(8))
        .then(snapshot_visible_cells())
        .then(set_radius(8))
        .then(set_edge_softness(0.42))
        .then(Action::WaitFrames(8))
        .then(snapshot_bridge_state())
        .then(assertions::custom("larger FOV radius expands bridged cells", |world| {
            let before = world.resource::<VisibleSnapshot>().0;
            let snapshot = world.resource::<BridgeSnapshot>();
            snapshot.radius == 8 && snapshot.visible_count > before && snapshot.matches
        }))
        .then(Action::Screenshot("fov_bridge_tuned".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("fov_integration_bridge"))
        .build()
}
