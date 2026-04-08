use bevy::prelude::*;
use saddle_world_fog_of_war::{FogLayerId, FogOfWarMap};
use saddle_world_fog_of_war_example_support::e2e_support::{Action, Scenario, assertions};

use crate::ConePane;

#[derive(Resource, Default, Clone, Copy)]
struct VisibleSnapshot(usize);

pub fn list() -> Vec<&'static str> {
    vec!["vision_cones_directional_arc"]
}

pub fn by_name(name: &str) -> Option<Scenario> {
    match name {
        "vision_cones_directional_arc" => Some(vision_cones_directional_arc()),
        _ => None,
    }
}

fn set_pause(paused: bool) -> Action {
    Action::Custom(Box::new(move |world| {
        world.resource_mut::<ConePane>().pause_motion = paused;
    }))
}

fn set_radius(radius_cells: f32) -> Action {
    Action::Custom(Box::new(move |world| {
        world.resource_mut::<ConePane>().radius_cells = radius_cells;
    }))
}

fn set_spread(spread_radians: f32) -> Action {
    Action::Custom(Box::new(move |world| {
        world.resource_mut::<ConePane>().spread_radians = spread_radians;
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

fn vision_cones_directional_arc() -> Scenario {
    Scenario::builder("vision_cones_directional_arc")
        .description("Freeze the sentry facing east, verify the forward arc reveals cells while the rear stays hidden, then widen the cone and confirm the footprint grows.")
        .then(Action::WaitFrames(10))
        .then(set_pause(true))
        .then(set_radius(5.5))
        .then(set_spread(0.9))
        .then(Action::WaitFrames(8))
        .then(assertions::custom("forward cell is revealed", |world| {
            world
                .resource::<FogOfWarMap>()
                .is_visible(FogLayerId(0), IVec2::new(9, 9))
        }))
        .then(assertions::custom("rear cell stays outside the cone", |world| {
            !world
                .resource::<FogOfWarMap>()
                .is_visible(FogLayerId(0), IVec2::new(5, 9))
        }))
        .then(Action::Screenshot("vision_cone_narrow".into()))
        .then(snapshot_visible_cells())
        .then(set_radius(7.0))
        .then(set_spread(1.5))
        .then(Action::WaitFrames(8))
        .then(assertions::custom("wider cone reveals more cells", |world| {
            let before = world.resource::<VisibleSnapshot>().0;
            world
                .resource::<FogOfWarMap>()
                .layer_summary(FogLayerId(0))
                .is_some_and(|summary| summary.visible_cells > before)
        }))
        .then(Action::Screenshot("vision_cone_wide".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("vision_cones_directional_arc"))
        .build()
}
