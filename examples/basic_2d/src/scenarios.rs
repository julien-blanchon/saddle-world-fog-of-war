use bevy::prelude::*;
use saddle_world_fog_of_war::{FogLayerId, FogOfWarMap};
use saddle_world_fog_of_war_example_support::e2e_support::{Action, Scenario, assertions};

use crate::BasicFogPane;

#[derive(Resource, Default, Clone, Copy)]
struct VisibleSnapshot(usize);

pub fn list() -> Vec<&'static str> {
    vec!["basic_2d_memory_trail"]
}

pub fn by_name(name: &str) -> Option<Scenario> {
    match name {
        "basic_2d_memory_trail" => Some(basic_2d_memory_trail()),
        _ => None,
    }
}

fn set_pause(paused: bool) -> Action {
    Action::Custom(Box::new(move |world| {
        world.resource_mut::<BasicFogPane>().pause_motion = paused;
    }))
}

fn set_radius(radius_cells: f32) -> Action {
    Action::Custom(Box::new(move |world| {
        world.resource_mut::<BasicFogPane>().scout_radius_cells = radius_cells;
    }))
}

fn set_edge_softness(edge_softness: f32) -> Action {
    Action::Custom(Box::new(move |world| {
        world.resource_mut::<BasicFogPane>().edge_softness = edge_softness;
    }))
}

fn snapshot_visible_cells() -> Action {
    Action::Custom(Box::new(|world| {
        let visible = world
            .resource::<FogOfWarMap>()
            .layer_summary(FogLayerId(0))
            .map_or(0, |summary| summary.visible_cells);
        world.insert_resource(VisibleSnapshot(visible));
    }))
}

fn basic_2d_memory_trail() -> Scenario {
    Scenario::builder("basic_2d_memory_trail")
        .description("Let the scout carve an explored trail, then widen the reveal radius and verify the visible footprint grows.")
        .then(Action::WaitFrames(20))
        .then(assertions::custom("initial visible footprint exists", |world| {
            world
                .resource::<FogOfWarMap>()
                .layer_summary(FogLayerId(0))
                .is_some_and(|summary| summary.visible_cells > 0)
        }))
        .then(snapshot_visible_cells())
        .then(Action::Screenshot("basic_start".into()))
        .then(Action::WaitFrames(120))
        .then(assertions::custom("movement leaves explored cells behind", |world| {
            world
                .resource::<FogOfWarMap>()
                .layer_summary(FogLayerId(0))
                .is_some_and(|summary| summary.explored_cells > summary.visible_cells)
        }))
        .then(set_pause(true))
        .then(set_radius(6.8))
        .then(set_edge_softness(0.45))
        .then(Action::WaitFrames(8))
        .then(assertions::custom("larger radius expands visible footprint", |world| {
            let before = world.resource::<VisibleSnapshot>().0;
            world
                .resource::<FogOfWarMap>()
                .layer_summary(FogLayerId(0))
                .is_some_and(|summary| summary.visible_cells > before)
        }))
        .then(Action::Screenshot("basic_tuned".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("basic_2d_memory_trail"))
        .build()
}
