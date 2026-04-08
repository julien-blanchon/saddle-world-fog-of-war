use bevy::prelude::*;
use saddle_world_fog_of_war::{FogLayerId, FogOfWarMap, FogOverlay2d};
use saddle_world_fog_of_war_example_support::e2e_support::{Action, Scenario, assertions};

use crate::RtsPane;

#[derive(Resource, Default, Clone, Copy)]
struct VisibleSnapshot(usize);

#[derive(Resource, Default, Clone, Copy)]
struct OverlayCountSnapshot(usize);

pub fn list() -> Vec<&'static str> {
    vec!["rts_large_map_multi_overlay"]
}

pub fn by_name(name: &str) -> Option<Scenario> {
    match name {
        "rts_large_map_multi_overlay" => Some(rts_large_map_multi_overlay()),
        _ => None,
    }
}

fn set_pause(paused: bool) -> Action {
    Action::Custom(Box::new(move |world| {
        world.resource_mut::<RtsPane>().pause_motion = paused;
    }))
}

fn set_radius(radius_cells: f32) -> Action {
    Action::Custom(Box::new(move |world| {
        world.resource_mut::<RtsPane>().scout_radius_cells = radius_cells;
    }))
}

fn set_edge_softness(edge_softness: f32) -> Action {
    Action::Custom(Box::new(move |world| {
        world.resource_mut::<RtsPane>().edge_softness = edge_softness;
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

fn snapshot_overlay_count() -> Action {
    Action::Custom(Box::new(|world| {
        let mut query = world.query::<&FogOverlay2d>();
        world.insert_resource(OverlayCountSnapshot(query.iter(world).count()));
    }))
}

fn rts_large_map_multi_overlay() -> Scenario {
    Scenario::builder("rts_large_map_multi_overlay")
        .description("Validate the dual-overlay RTS showcase, let the scout rings explore the map, then widen the reveal radius and verify the shared layer grows.")
        .then(Action::WaitFrames(20))
        .then(snapshot_overlay_count())
        .then(assertions::custom("both world and minimap overlays exist", |world| {
            world.resource::<OverlayCountSnapshot>().0 == 2
        }))
        .then(Action::WaitFrames(150))
        .then(assertions::custom("scout rings explore beyond current sight", |world| {
            world
                .resource::<FogOfWarMap>()
                .layer_summary(FogLayerId(0))
                .is_some_and(|summary| summary.explored_cells > summary.visible_cells)
        }))
        .then(Action::Screenshot("rts_overview".into()))
        .then(set_pause(true))
        .then(snapshot_visible_cells())
        .then(set_radius(5.2))
        .then(set_edge_softness(0.48))
        .then(Action::WaitFrames(8))
        .then(assertions::custom("larger scout radius expands shared fog coverage", |world| {
            let before = world.resource::<VisibleSnapshot>().0;
            world
                .resource::<FogOfWarMap>()
                .layer_summary(FogLayerId(0))
                .is_some_and(|summary| summary.visible_cells > before)
        }))
        .then(Action::Screenshot("rts_tuned".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("rts_large_map_multi_overlay"))
        .build()
}
