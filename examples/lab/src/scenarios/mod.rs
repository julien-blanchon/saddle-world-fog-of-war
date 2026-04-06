use bevy::prelude::*;
use saddle_bevy_e2e::{action::Action, actions::assertions, scenario::Scenario};
use saddle_world_fog_of_war::{
    FogLayerId, FogOfWarMap, FogOfWarRenderAssets, FogPersistenceMode, FogVisibilityState,
};

use crate::{
    LabControl, LabDiagnostics, MEMORY_SAMPLE_CELL, OCCLUDED_SAMPLE_CELL, TEAM_ONE_SAMPLE_CELL,
    TEAM_ZERO_SAMPLE_CELL, place_scout_alpha, place_scout_beta, place_sentry,
    set_exploration_memory, set_pause_motion, set_selected_layer,
};

#[derive(Resource, Default, Clone, Copy)]
struct ProjectionCameraSnapshot(Vec3);

pub fn list_scenarios() -> Vec<&'static str> {
    vec![
        "fog_of_war_smoke",
        "fog_of_war_exploration_memory",
        "fog_of_war_no_memory",
        "fog_of_war_occlusion",
        "fog_of_war_team_layers",
        "fog_of_war_3d_projection",
    ]
}

pub fn scenario_by_name(name: &str) -> Option<Scenario> {
    match name {
        "fog_of_war_smoke" => Some(fog_of_war_smoke()),
        "fog_of_war_exploration_memory" => Some(fog_of_war_exploration_memory()),
        "fog_of_war_no_memory" => Some(fog_of_war_no_memory()),
        "fog_of_war_occlusion" => Some(fog_of_war_occlusion()),
        "fog_of_war_team_layers" => Some(fog_of_war_team_layers()),
        "fog_of_war_3d_projection" => Some(fog_of_war_3d_projection()),
        _ => None,
    }
}

fn set_layer(layer: FogLayerId) -> Action {
    Action::Custom(Box::new(move |world| set_selected_layer(world, layer)))
}

fn pause_motion(paused: bool) -> Action {
    Action::Custom(Box::new(move |world| set_pause_motion(world, paused)))
}

fn exploration_memory(enabled: bool) -> Action {
    Action::Custom(Box::new(move |world| set_exploration_memory(world, enabled)))
}

fn move_alpha(position: Vec2) -> Action {
    Action::Custom(Box::new(move |world| place_scout_alpha(world, position)))
}

fn move_beta(position: Vec2) -> Action {
    Action::Custom(Box::new(move |world| place_scout_beta(world, position)))
}

fn move_sentry(position: Vec2, facing: Vec2) -> Action {
    Action::Custom(Box::new(move |world| place_sentry(world, position, facing)))
}

fn fog_of_war_smoke() -> Scenario {
    Scenario::builder("fog_of_war_smoke")
        .description("Boot the lab, verify fog resources initialize, and capture the default projection view.")
        .then(Action::WaitFrames(30))
        .then(assertions::custom("map resource exists", |world| {
            world.contains_resource::<FogOfWarMap>()
        }))
        .then(assertions::custom("render image for layer 0", |world| {
            world.resource::<FogOfWarRenderAssets>()
                .image(FogLayerId(0))
                .is_some()
        }))
        .then(assertions::custom("layer 0 has visible cells", |world| {
            world
                .resource::<FogOfWarMap>()
                .layer_summary(FogLayerId(0))
                .is_some_and(|summary| summary.visible_cells > 0)
        }))
        .then(assertions::custom("selected layer defaults to zero", |world| {
            world.resource::<LabDiagnostics>().selected_layer == FogLayerId(0)
        }))
        .then(Action::Screenshot("smoke".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("fog_of_war_smoke"))
        .build()
}

fn fog_of_war_exploration_memory() -> Scenario {
    Scenario::builder("fog_of_war_exploration_memory")
        .description("Move the primary scout across the arena and verify old cells degrade to explored instead of returning to hidden.")
        .then(Action::WaitFrames(2))
        .then(pause_motion(true))
        .then(exploration_memory(true))
        .then(set_layer(FogLayerId(0)))
        .then(move_beta(Vec2::new(21.0, 14.0)))
        .then(move_alpha(Vec2::new(4.5, 4.5)))
        .then(Action::WaitFrames(6))
        .then(assertions::custom("memory sample starts visible", |world| {
            world
                .resource::<FogOfWarMap>()
                .visibility_at_cell(FogLayerId(0), MEMORY_SAMPLE_CELL)
                == Some(FogVisibilityState::Visible)
        }))
        .then(Action::Screenshot("memory_visible".into()))
        .then(Action::WaitFrames(1))
        .then(move_alpha(Vec2::new(16.5, 4.5)))
        .then(Action::WaitFrames(8))
        .then(assertions::custom("memory sample becomes explored", |world| {
            world
                .resource::<FogOfWarMap>()
                .visibility_at_cell(FogLayerId(0), MEMORY_SAMPLE_CELL)
                == Some(FogVisibilityState::Explored)
        }))
        .then(assertions::custom("destination cell becomes visible", |world| {
            world
                .resource::<FogOfWarMap>()
                .visibility_at_cell(FogLayerId(0), IVec2::new(16, 4))
                == Some(FogVisibilityState::Visible)
        }))
        .then(Action::Screenshot("memory_explored".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("fog_of_war_exploration_memory"))
        .build()
}

fn fog_of_war_no_memory() -> Scenario {
    Scenario::builder("fog_of_war_no_memory")
        .description("Disable exploration memory and verify cells return to hidden as soon as vision leaves them.")
        .then(Action::WaitFrames(2))
        .then(pause_motion(true))
        .then(exploration_memory(false))
        .then(set_layer(FogLayerId(0)))
        .then(move_beta(Vec2::new(21.0, 14.0)))
        .then(move_alpha(Vec2::new(4.5, 4.5)))
        .then(Action::WaitFrames(8))
        .then(assertions::custom("lab switched to no-memory mode", |world| {
            world.resource::<LabDiagnostics>().persistence_mode == FogPersistenceMode::NoMemory
        }))
        .then(assertions::custom("sample starts visible in no-memory mode", |world| {
            world
                .resource::<FogOfWarMap>()
                .visibility_at_cell(FogLayerId(0), MEMORY_SAMPLE_CELL)
                == Some(FogVisibilityState::Visible)
        }))
        .then(Action::Screenshot("no_memory_visible".into()))
        .then(Action::WaitFrames(1))
        .then(move_alpha(Vec2::new(16.5, 4.5)))
        .then(Action::WaitFrames(8))
        .then(assertions::custom("sample returns to hidden", |world| {
            world
                .resource::<FogOfWarMap>()
                .visibility_at_cell(FogLayerId(0), MEMORY_SAMPLE_CELL)
                == Some(FogVisibilityState::Hidden)
        }))
        .then(Action::Screenshot("no_memory_hidden".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("fog_of_war_no_memory"))
        .build()
}

fn fog_of_war_occlusion() -> Scenario {
    Scenario::builder("fog_of_war_occlusion")
        .description("Pin the scout against the central wall and assert that blocked cells remain hidden behind the occluder.")
        .then(Action::WaitFrames(2))
        .then(pause_motion(true))
        .then(exploration_memory(true))
        .then(set_layer(FogLayerId(0)))
        .then(move_beta(Vec2::new(22.0, 15.0)))
        .then(move_alpha(Vec2::new(7.5, 8.5)))
        .then(Action::WaitFrames(6))
        .then(assertions::custom("front-side cell visible", |world| {
            world
                .resource::<FogOfWarMap>()
                .visibility_at_cell(FogLayerId(0), IVec2::new(9, 8))
                == Some(FogVisibilityState::Visible)
        }))
        .then(assertions::custom("occluded cell hidden", |world| {
            world
                .resource::<FogOfWarMap>()
                .visibility_at_cell(FogLayerId(0), OCCLUDED_SAMPLE_CELL)
                == Some(FogVisibilityState::Hidden)
        }))
        .then(Action::Screenshot("occlusion".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("fog_of_war_occlusion"))
        .build()
}

fn fog_of_war_team_layers() -> Scenario {
    Scenario::builder("fog_of_war_team_layers")
        .description("Switch the presentation between team layers and verify the selected receiver layer and visible samples change with it.")
        .then(Action::WaitFrames(2))
        .then(pause_motion(true))
        .then(exploration_memory(true))
        .then(move_alpha(Vec2::new(4.5, 4.5)))
        .then(move_beta(Vec2::new(20.5, 13.0)))
        .then(move_sentry(Vec2::new(18.0, 6.5), Vec2::new(-1.0, 0.0)))
        .then(set_layer(FogLayerId(0)))
        .then(Action::WaitFrames(5))
        .then(assertions::custom("team zero sample visible", |world| {
            world
                .resource::<FogOfWarMap>()
                .visibility_at_cell(FogLayerId(0), TEAM_ZERO_SAMPLE_CELL)
                == Some(FogVisibilityState::Visible)
        }))
        .then(assertions::custom("receiver synced to layer zero", |world| {
            let diagnostics = world.resource::<LabDiagnostics>();
            diagnostics.selected_layer == FogLayerId(0)
                && diagnostics.main_receiver_layer == FogLayerId(0)
        }))
        .then(Action::Screenshot("team_zero".into()))
        .then(Action::WaitFrames(1))
        .then(set_layer(FogLayerId(1)))
        .then(Action::WaitFrames(5))
        .then(assertions::custom("team one sample visible", |world| {
            world
                .resource::<FogOfWarMap>()
                .visibility_at_cell(FogLayerId(1), TEAM_ONE_SAMPLE_CELL)
                == Some(FogVisibilityState::Visible)
        }))
        .then(assertions::custom("receiver synced to layer one", |world| {
            let diagnostics = world.resource::<LabDiagnostics>();
            diagnostics.selected_layer == FogLayerId(1)
                && diagnostics.main_receiver_layer == FogLayerId(1)
        }))
        .then(Action::Screenshot("team_one".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("fog_of_war_team_layers"))
        .build()
}

fn fog_of_war_3d_projection() -> Scenario {
    Scenario::builder("fog_of_war_3d_projection")
        .description("Let the camera orbit the projected fog receiver and verify the plane stays aligned while the camera moves.")
        .then(exploration_memory(true))
        .then(pause_motion(false))
        .then(set_layer(FogLayerId(0)))
        .then(Action::WaitFrames(20))
        .then(Action::Custom(Box::new(|world| {
            let mut snapshot = world
                .remove_resource::<ProjectionCameraSnapshot>()
                .unwrap_or_default();
            snapshot.0 = world.resource::<LabDiagnostics>().camera_pos;
            world.insert_resource(snapshot);
        })))
        .then(Action::Screenshot("projection_start".into()))
        .then(Action::WaitFrames(1))
        .then(Action::WaitFrames(90))
        .then(assertions::custom("camera moved during orbit", |world| {
            let before = world.resource::<ProjectionCameraSnapshot>().0;
            let after = world.resource::<LabDiagnostics>().camera_pos;
            before.distance(after) > 1.0
        }))
        .then(assertions::custom("projection receiver still showing selected layer", |world| {
            let diagnostics = world.resource::<LabDiagnostics>();
            diagnostics.main_receiver_layer == world.resource::<LabControl>().selected_layer
        }))
        .then(Action::Screenshot("projection_orbit".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("fog_of_war_3d_projection"))
        .build()
}
