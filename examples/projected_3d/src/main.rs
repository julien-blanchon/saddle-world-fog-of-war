use saddle_world_fog_of_war_example_support as support;

#[cfg(feature = "e2e")]
mod scenarios;

use bevy::prelude::*;
use saddle_pane::prelude::*;
use saddle_world_fog_of_war::{
    FogLayerId, FogOfWarMap, FogOfWarPlugin, FogOfWarRenderingPlugin, FogProjectionReceiver,
    FogRevealShape, VisionSource,
};

#[derive(Component)]
struct ProjectionScout;

#[derive(Component)]
struct ProjectionCamera;

#[derive(Resource, Debug, Clone, Copy, Pane)]
#[pane(title = "Projected Fog 3D", position = "top-right")]
struct ProjectedPane {
    #[pane]
    pause_motion: bool,
    #[pane(slider, min = 2.0, max = 8.0, step = 0.1)]
    scout_radius: f32,
    #[pane(slider, min = 0.1, max = 1.2, step = 0.02)]
    scout_speed: f32,
    #[pane(slider, min = 0.05, max = 0.6, step = 0.02)]
    camera_speed: f32,
    #[pane(slider, min = 0.0, max = 0.6, step = 0.01)]
    edge_softness: f32,
    #[pane(monitor)]
    visible_cells: usize,
    #[pane(monitor)]
    explored_cells: usize,
}

impl Default for ProjectedPane {
    fn default() -> Self {
        Self {
            pause_motion: false,
            scout_radius: 4.4,
            scout_speed: 0.6,
            camera_speed: 0.18,
            edge_softness: 0.3,
            visible_cells: 0,
            explored_cells: 0,
        }
    }
}

fn main() {
    let config = support::config_3d(UVec2::new(26, 20));

    let mut app = App::new();
    app.insert_resource(ProjectedPane::default());
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "fog_of_war projected_3d".into(),
            resolution: (1400, 860).into(),
            ..default()
        }),
        ..default()
    }));
    app.add_plugins((
        bevy_flair::FlairPlugin,
        bevy_input_focus::InputDispatchPlugin,
        bevy_ui_widgets::UiWidgetsPlugins,
        bevy_input_focus::tab_navigation::TabNavigationPlugin,
        PanePlugin,
    ))
    .register_pane::<ProjectedPane>();
    app.add_plugins((
        FogOfWarPlugin::default().with_config(config.clone()),
        FogOfWarRenderingPlugin::default(),
    ));
    app.add_systems(
        Startup,
        move |mut commands: Commands,
              mut meshes: ResMut<Assets<Mesh>>,
              mut materials: ResMut<Assets<StandardMaterial>>| {
            setup(&mut commands, &mut meshes, &mut materials, &config);
        },
    );
    app.add_systems(
        Update,
        sync_controls.before(saddle_world_fog_of_war::FogOfWarSystems::CollectVisionSources),
    );
    app.add_systems(Update, (move_projection_scout, orbit_camera));
    app.add_systems(
        Update,
        update_pane.after(saddle_world_fog_of_war::FogOfWarSystems::ApplyPersistence),
    );
    #[cfg(feature = "e2e")]
    app.add_plugins(support::e2e_support::ExampleE2EPlugin::new(
        scenarios::list,
        scenarios::by_name,
    ));
    app.run();
}

fn setup(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    config: &saddle_world_fog_of_war::FogOfWarConfig,
) {
    support::spawn_3d_camera(commands, config);
    commands.spawn((Name::new("Orbit Camera Marker"), ProjectionCamera));
    support::spawn_ground_3d(commands, meshes, materials, config);

    support::spawn_wall_3d(
        commands,
        meshes,
        materials,
        "Projection Wall A",
        Vec2::new(9.0, 7.0),
        Vec2::new(1.5, 8.0),
        2.4,
        Color::srgb(0.38, 0.36, 0.33),
    );
    support::spawn_wall_3d(
        commands,
        meshes,
        materials,
        "Projection Wall B",
        Vec2::new(15.5, 13.0),
        Vec2::new(7.0, 1.5),
        2.1,
        Color::srgb(0.33, 0.33, 0.38),
    );

    let scout = support::spawn_source_3d(
        commands,
        meshes,
        materials,
        "Projection Scout",
        Vec2::new(6.0, 6.0),
        VisionSource::circle(FogLayerId(0), 4.4),
        Color::srgb(0.36, 0.92, 0.70),
    );
    commands.entity(scout).insert(ProjectionScout);

    support::spawn_projection_receiver(
        commands,
        FogLayerId(0),
        config.grid.origin,
        config.grid.world_size(),
        support::layer_palette(0.94, 0.70),
    );
    support::spawn_instructions(
        commands,
        "Projected Fog 3D",
        "Use the pane in the top-right to pause the scene, tune the scout radius, adjust scout and camera speed, and soften the projection edge.\nThis example keeps the same CPU fog truth but renders it as a projected 3D receiver.",
    );
}

fn move_projection_scout(
    time: Res<Time>,
    pane: Res<ProjectedPane>,
    mut scout: Query<&mut Transform, With<ProjectionScout>>,
) {
    if pane.pause_motion {
        return;
    }
    let Ok(mut transform) = scout.single_mut() else {
        return;
    };
    let phase = time.elapsed_secs() * pane.scout_speed;
    transform.translation.x = 8.0 + phase.sin() * 5.0;
    transform.translation.z = 9.0 + (phase * 1.2).cos() * 4.0;
}

fn orbit_camera(
    time: Res<Time>,
    pane: Res<ProjectedPane>,
    mut camera: Query<&mut Transform, (With<Camera3d>, Without<ProjectionScout>)>,
) {
    if pane.pause_motion {
        return;
    }
    let Ok(mut transform) = camera.single_mut() else {
        return;
    };
    let center = Vec3::new(13.0, 0.0, 10.0);
    let angle = time.elapsed_secs() * pane.camera_speed;
    transform.translation = Vec3::new(
        center.x + angle.cos() * 14.0,
        18.0 + (angle * 2.0).sin() * 1.5,
        center.z + angle.sin() * 11.0,
    );
    transform.look_at(center, Vec3::Y);
}

fn sync_controls(
    pane: Res<ProjectedPane>,
    mut scout: Single<&mut VisionSource, With<ProjectionScout>>,
    mut receiver: Single<&mut FogProjectionReceiver>,
) {
    if !pane.is_changed() {
        return;
    }

    scout.shape = FogRevealShape::circle(pane.scout_radius);
    receiver.edge_softness = pane.edge_softness;
}

fn update_pane(map: Res<FogOfWarMap>, mut pane: ResMut<ProjectedPane>) {
    let summary =
        map.layer_summary(FogLayerId(0))
            .unwrap_or(saddle_world_fog_of_war::FogLayerSummary {
                visible_cells: 0,
                explored_cells: 0,
            });
    pane.visible_cells = summary.visible_cells;
    pane.explored_cells = summary.explored_cells;
}
