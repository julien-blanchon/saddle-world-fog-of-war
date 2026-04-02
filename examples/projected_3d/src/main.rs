use saddle_world_fog_of_war_example_support as support;

use bevy::prelude::*;
use saddle_world_fog_of_war::{FogLayerId, FogOfWarPlugin, VisionSource};

#[derive(Component)]
struct ProjectionScout;

#[derive(Component)]
struct ProjectionCamera;

fn main() {
    let config = support::config_3d(UVec2::new(26, 20));

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "fog_of_war projected_3d".into(),
                resolution: (1400, 860).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(FogOfWarPlugin::default().with_config(config.clone()))
        .add_systems(
            Startup,
            move |mut commands: Commands,
                  mut meshes: ResMut<Assets<Mesh>>,
                  mut materials: ResMut<Assets<StandardMaterial>>| {
                setup(&mut commands, &mut meshes, &mut materials, &config);
            },
        )
        .add_systems(Update, (move_projection_scout, orbit_camera))
        .run();
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
}

fn move_projection_scout(time: Res<Time>, mut scout: Query<&mut Transform, With<ProjectionScout>>) {
    let Ok(mut transform) = scout.single_mut() else {
        return;
    };
    let phase = time.elapsed_secs() * 0.6;
    transform.translation.x = 8.0 + phase.sin() * 5.0;
    transform.translation.z = 9.0 + (phase * 1.2).cos() * 4.0;
}

fn orbit_camera(
    time: Res<Time>,
    mut camera: Query<&mut Transform, (With<Camera3d>, Without<ProjectionScout>)>,
) {
    let Ok(mut transform) = camera.single_mut() else {
        return;
    };
    let center = Vec3::new(13.0, 0.0, 10.0);
    let angle = time.elapsed_secs() * 0.18;
    transform.translation = Vec3::new(
        center.x + angle.cos() * 14.0,
        18.0 + (angle * 2.0).sin() * 1.5,
        center.z + angle.sin() * 11.0,
    );
    transform.look_at(center, Vec3::Y);
}
