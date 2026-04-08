use bevy::prelude::*;
use saddle_world_fog_of_war::{FogLayerId, FogOfWarMap, FogVisibilityState};
use saddle_world_fog_of_war_example_support as support;
use saddle_world_fog_of_war_example_support::e2e_support::{Action, Scenario, assertions};

use crate::{Observer, OcclusionPane};

pub fn list() -> Vec<&'static str> {
    vec!["occlusion_2d_wall_shadow"]
}

pub fn by_name(name: &str) -> Option<Scenario> {
    match name {
        "occlusion_2d_wall_shadow" => Some(occlusion_2d_wall_shadow()),
        _ => None,
    }
}

fn set_pause(paused: bool) -> Action {
    Action::Custom(Box::new(move |world| {
        world.resource_mut::<OcclusionPane>().pause_motion = paused;
    }))
}

fn set_radius(radius_cells: f32) -> Action {
    Action::Custom(Box::new(move |world| {
        world.resource_mut::<OcclusionPane>().observer_radius_cells = radius_cells;
    }))
}

fn place_observer(cell: IVec2) -> Action {
    Action::Custom(Box::new(move |world| {
        let world_pos = (cell.as_vec2() + Vec2::splat(0.5)) * support::CELL_SIZE_2D;
        let mut query = world.query_filtered::<&mut Transform, With<Observer>>();
        if let Ok(mut transform) = query.single_mut(world) {
            transform.translation.x = world_pos.x;
            transform.translation.y = world_pos.y;
        }
    }))
}

fn occlusion_2d_wall_shadow() -> Scenario {
    Scenario::builder("occlusion_2d_wall_shadow")
        .description("Pin the observer near the central wall, verify the shadowed cells stay hidden, then resume the sweep and capture explored coverage.")
        .then(Action::WaitFrames(10))
        .then(set_pause(true))
        .then(set_radius(4.2))
        .then(place_observer(IVec2::new(8, 13)))
        .then(Action::WaitFrames(8))
        .then(assertions::custom("front-side cell is visible", |world| {
            world
                .resource::<FogOfWarMap>()
                .visibility_at_cell(FogLayerId(0), IVec2::new(10, 13))
                == Some(FogVisibilityState::Visible)
        }))
        .then(assertions::custom("cell behind wall stays hidden", |world| {
            world
                .resource::<FogOfWarMap>()
                .visibility_at_cell(FogLayerId(0), IVec2::new(14, 13))
                == Some(FogVisibilityState::Hidden)
        }))
        .then(Action::Screenshot("occlusion_shadow".into()))
        .then(set_pause(false))
        .then(Action::WaitFrames(90))
        .then(assertions::custom("sweep leaves explored coverage", |world| {
            world
                .resource::<FogOfWarMap>()
                .layer_summary(FogLayerId(0))
                .is_some_and(|summary| summary.explored_cells > summary.visible_cells)
        }))
        .then(Action::Screenshot("occlusion_sweep".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("occlusion_2d_wall_shadow"))
        .build()
}
