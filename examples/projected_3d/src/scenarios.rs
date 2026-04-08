use bevy::prelude::*;
use saddle_world_fog_of_war::{FogLayerId, FogOfWarMap};
use saddle_world_fog_of_war_example_support::e2e_support::{Action, Scenario, assertions};

use crate::ProjectedPane;

#[derive(Resource, Default, Clone, Copy)]
struct CameraStartSnapshot(Vec3);

#[derive(Resource, Default, Clone, Copy)]
struct CameraCurrentSnapshot(Vec3);

#[derive(Resource, Default, Clone, Copy)]
struct VisibleSnapshot(usize);

pub fn list() -> Vec<&'static str> {
    vec!["projected_3d_projection_orbit"]
}

pub fn by_name(name: &str) -> Option<Scenario> {
    match name {
        "projected_3d_projection_orbit" => Some(projected_3d_projection_orbit()),
        _ => None,
    }
}

fn set_pause(paused: bool) -> Action {
    Action::Custom(Box::new(move |world| {
        world.resource_mut::<ProjectedPane>().pause_motion = paused;
    }))
}

fn set_radius(radius: f32) -> Action {
    Action::Custom(Box::new(move |world| {
        world.resource_mut::<ProjectedPane>().scout_radius = radius;
    }))
}

fn set_edge_softness(edge_softness: f32) -> Action {
    Action::Custom(Box::new(move |world| {
        world.resource_mut::<ProjectedPane>().edge_softness = edge_softness;
    }))
}

fn snapshot_camera() -> Action {
    Action::Custom(Box::new(|world| {
        let mut query = world.query_filtered::<&Transform, With<Camera3d>>();
        let position = query
            .single(world)
            .map_or(Vec3::ZERO, |transform| transform.translation);
        world.insert_resource(CameraStartSnapshot(position));
    }))
}

fn snapshot_current_camera() -> Action {
    Action::Custom(Box::new(|world| {
        let mut query = world.query_filtered::<&Transform, With<Camera3d>>();
        let position = query
            .single(world)
            .map_or(Vec3::ZERO, |transform| transform.translation);
        world.insert_resource(CameraCurrentSnapshot(position));
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

fn projected_3d_projection_orbit() -> Scenario {
    Scenario::builder("projected_3d_projection_orbit")
        .description("Let the camera orbit the projected receiver, verify exploration grows, then pause and widen the scout radius to confirm the projection responds.")
        .then(Action::WaitFrames(20))
        .then(snapshot_camera())
        .then(Action::Screenshot("projection_start".into()))
        .then(Action::WaitFrames(120))
        .then(snapshot_current_camera())
        .then(assertions::custom("camera orbit changes the viewpoint", |world| {
            let before = world.resource::<CameraStartSnapshot>().0;
            let after = world.resource::<CameraCurrentSnapshot>().0;
            after.distance(before) > 1.0
        }))
        .then(assertions::custom("scout movement leaves explored projection coverage", |world| {
            world
                .resource::<FogOfWarMap>()
                .layer_summary(FogLayerId(0))
                .is_some_and(|summary| summary.explored_cells > summary.visible_cells)
        }))
        .then(set_pause(true))
        .then(snapshot_visible_cells())
        .then(set_radius(7.0))
        .then(set_edge_softness(0.46))
        .then(Action::WaitFrames(8))
        .then(assertions::custom("larger projection radius expands visible cells", |world| {
            let before = world.resource::<VisibleSnapshot>().0;
            world
                .resource::<FogOfWarMap>()
                .layer_summary(FogLayerId(0))
                .is_some_and(|summary| summary.visible_cells > before)
        }))
        .then(Action::Screenshot("projection_tuned".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("projected_3d_projection_orbit"))
        .build()
}
