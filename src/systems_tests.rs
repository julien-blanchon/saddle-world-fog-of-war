use bevy::{ecs::schedule::ScheduleLabel, prelude::*};

use super::*;
use crate::{
    FogOfWarPlugin, FogOfWarSystems,
    grid::{FogGridSpec, FogLayerId},
    messages::VisibilityMapUpdated,
    resources::{FogOfWarConfig, FogOfWarMap, FogOfWarStats},
};

#[derive(Resource, Default, Debug)]
struct UpdateCapture(Vec<VisibilityMapUpdated>);

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct DeactivateSchedule;

fn collect_updates(
    mut reader: MessageReader<VisibilityMapUpdated>,
    mut capture: ResMut<UpdateCapture>,
) {
    capture.0.extend(reader.read().cloned());
}

fn test_config() -> FogOfWarConfig {
    FogOfWarConfig {
        grid: FogGridSpec {
            origin: Vec2::ZERO,
            dimensions: UVec2::new(10, 10),
            cell_size: Vec2::ONE,
            chunk_size: UVec2::splat(4),
        },
        ..default()
    }
}

fn layer_totals(map: &FogOfWarMap) -> (usize, usize, usize) {
    [FogLayerId(0), FogLayerId(1)]
        .into_iter()
        .filter_map(|layer| map.layer_summary(layer))
        .fold((0, 0, 0), |(layers, visible, explored), summary| {
            (
                layers + 1,
                visible + summary.visible_cells,
                explored + summary.explored_cells,
            )
        })
}

#[test]
fn plugin_collects_entities_emits_messages_and_clears_on_deactivate() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_schedule(DeactivateSchedule);
    app.insert_resource(UpdateCapture::default());
    app.add_plugins(
        FogOfWarPlugin::new(Startup, DeactivateSchedule, Update).with_config(test_config()),
    );
    app.add_systems(
        Update,
        collect_updates.after(FogOfWarSystems::UpdateExplorationMemory),
    );

    app.world_mut().spawn((
        Transform::from_xyz(2.5, 2.5, 0.0),
        GlobalTransform::from_xyz(2.5, 2.5, 0.0),
        crate::components::VisionSource::circle(FogLayerId(0), 2.5),
    ));

    app.update();

    assert!(app.world().resource::<FogRuntimeState>().active);
    assert!(
        app.world()
            .resource::<FogOfWarMap>()
            .layer_summary(FogLayerId(0))
            .is_some_and(|summary| summary.visible_cells > 0)
    );
    assert_eq!(app.world().resource::<FogOfWarStats>().source_count, 1);
    assert!(app.world().resource::<FogOfWarStats>().visible_cells_total > 0);
    assert_eq!(app.world().resource::<UpdateCapture>().0.len(), 1);

    app.world_mut().run_schedule(DeactivateSchedule);

    assert!(!app.world().resource::<FogRuntimeState>().active);
    assert_eq!(
        app.world()
            .resource::<FogOfWarMap>()
            .visibility_at_cell(FogLayerId(0), IVec2::new(2, 2)),
        Some(crate::grid::FogVisibilityState::Explored)
    );
    assert_eq!(app.world().resource::<FogOfWarStats>().source_count, 0);
    assert_eq!(app.world().resource::<FogOfWarStats>().occluder_count, 0);
    assert_eq!(
        app.world().resource::<FogOfWarStats>().visible_cells_total,
        0
    );
    assert!(app.world().resource::<FogOfWarStats>().explored_cells_total > 0);
}

#[test]
fn xz_projection_uses_ground_plane_coordinates() {
    let mut config = FogOfWarConfig::default();
    config.grid.dimensions = UVec2::new(12, 12);
    config.world_axes = crate::resources::FogWorldAxes::XZ;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(FogOfWarPlugin::default().with_config(config));

    app.world_mut().spawn((
        Transform::from_xyz(4.5, 9.0, 7.5),
        GlobalTransform::from_xyz(4.5, 9.0, 7.5),
        crate::components::VisionSource::circle(FogLayerId(0), 1.6),
    ));

    app.update();

    assert_eq!(
        app.world()
            .resource::<FogOfWarMap>()
            .visibility_at_cell(FogLayerId(0), IVec2::new(4, 7)),
        Some(crate::grid::FogVisibilityState::Visible)
    );
}

#[test]
fn stats_totals_include_steady_and_unchanged_layers() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(FogOfWarPlugin::default().with_config(test_config()));

    let moving_source = app
        .world_mut()
        .spawn((
            Transform::from_xyz(2.5, 2.5, 0.0),
            GlobalTransform::from_xyz(2.5, 2.5, 0.0),
            crate::components::VisionSource::circle(FogLayerId(0), 2.5),
        ))
        .id();

    app.world_mut().spawn((
        Transform::from_xyz(7.5, 7.5, 0.0),
        GlobalTransform::from_xyz(7.5, 7.5, 0.0),
        crate::components::VisionSource::circle(FogLayerId(1), 2.5),
    ));

    app.update();

    let (expected_layers, expected_visible, expected_explored) = {
        let map = app.world().resource::<FogOfWarMap>();
        layer_totals(map)
    };
    let stats = app.world().resource::<FogOfWarStats>();
    assert_eq!(stats.layer_count, expected_layers);
    assert_eq!(stats.visible_cells_total, expected_visible);
    assert_eq!(stats.explored_cells_total, expected_explored);

    app.world_mut().entity_mut(moving_source).insert((
        Transform::from_xyz(4.5, 4.5, 0.0),
        GlobalTransform::from_xyz(4.5, 4.5, 0.0),
    ));

    app.update();

    let (expected_layers, expected_visible, expected_explored) = {
        let map = app.world().resource::<FogOfWarMap>();
        layer_totals(map)
    };
    let stats = app.world().resource::<FogOfWarStats>();
    assert_eq!(stats.layer_count, expected_layers);
    assert_eq!(stats.visible_cells_total, expected_visible);
    assert_eq!(stats.explored_cells_total, expected_explored);
    assert_eq!(stats.source_count, 2);
}

#[test]
fn shared_layers_reveal_allied_fog_layers() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(FogOfWarPlugin::default().with_config(test_config()));

    app.world_mut().spawn((
        Transform::from_xyz(2.5, 2.5, 0.0),
        GlobalTransform::from_xyz(2.5, 2.5, 0.0),
        crate::components::VisionSource::circle(FogLayerId(0), 2.5)
            .with_shared_layers(crate::grid::FogLayerMask::bit(FogLayerId(1))),
    ));

    app.update();

    let map = app.world().resource::<FogOfWarMap>();
    assert_eq!(
        map.visibility_at_cell(FogLayerId(0), IVec2::new(2, 2)),
        Some(crate::grid::FogVisibilityState::Visible)
    );
    assert_eq!(
        map.visibility_at_cell(FogLayerId(1), IVec2::new(2, 2)),
        Some(crate::grid::FogVisibilityState::Visible)
    );
}
