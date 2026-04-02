use saddle_world_fog_of_war_example_support as support;

use bevy::prelude::*;
use saddle_world_fog_of_war::{FogLayerId, FogOfWarPlugin, VisionSource};

#[derive(Component)]
struct Sensor;

fn main() {
    let config = support::config_2d(UVec2::new(26, 18));

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "fog_of_war vision_cones".into(),
                resolution: (1280, 820).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(FogOfWarPlugin::default().with_config(config.clone()))
        .add_systems(Startup, move |mut commands: Commands| {
            setup(&mut commands, &config)
        })
        .add_systems(Update, spin_sensor)
        .run();
}

fn setup(commands: &mut Commands, config: &saddle_world_fog_of_war::FogOfWarConfig) {
    support::spawn_2d_camera(commands, config);
    support::spawn_2d_backdrop(commands, config, Color::srgb(0.08, 0.09, 0.11));

    support::spawn_wall_2d(
        commands,
        config,
        "Cone Wall 1",
        IVec2::new(10, 4),
        UVec2::new(2, 10),
        Color::srgb(0.36, 0.34, 0.30),
    );
    support::spawn_wall_2d(
        commands,
        config,
        "Cone Wall 2",
        IVec2::new(16, 8),
        UVec2::new(2, 6),
        Color::srgb(0.36, 0.34, 0.30),
    );

    let sensor = support::spawn_source_2d(
        commands,
        config,
        "Sensor",
        IVec2::new(7, 9),
        VisionSource::new(FogLayerId(0), support::moving_arc_shape(0.0, 5.5, 1.0)),
        Color::srgb(0.98, 0.62, 0.32),
    );
    commands.entity(sensor).insert(Sensor);

    support::spawn_overlay_2d(
        commands,
        FogLayerId(0),
        config.grid.origin,
        config.grid.world_size(),
        support::layer_palette(0.96, 0.76),
    );
}

fn spin_sensor(time: Res<Time>, mut sensor: Query<&mut VisionSource, With<Sensor>>) {
    let Ok(mut source) = sensor.single_mut() else {
        return;
    };
    let angle = time.elapsed_secs() * 0.65;
    source.shape = support::moving_arc_shape(angle, 5.5, 1.0);
}
