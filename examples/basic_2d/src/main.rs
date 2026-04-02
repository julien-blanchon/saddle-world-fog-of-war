use saddle_world_fog_of_war_example_support as support;

use bevy::prelude::*;
use saddle_world_fog_of_war::{FogLayerId, FogOfWarPlugin, VisionSource};

#[derive(Component)]
struct Scout;

fn main() {
    let config = support::config_2d(UVec2::new(28, 18));

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "fog_of_war basic_2d".into(),
                resolution: (1280, 820).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(FogOfWarPlugin::default().with_config(config.clone()))
        .add_systems(Startup, move |mut commands: Commands| {
            setup(&mut commands, &config)
        })
        .add_systems(Update, animate_scout)
        .run();
}

fn setup(commands: &mut Commands, config: &saddle_world_fog_of_war::FogOfWarConfig) {
    support::spawn_2d_camera(commands, config);
    support::spawn_2d_backdrop(commands, config, Color::srgb(0.10, 0.12, 0.14));

    support::spawn_wall_2d(
        commands,
        config,
        "North Wall",
        IVec2::new(6, 10),
        UVec2::new(12, 2),
        Color::srgb(0.32, 0.30, 0.28),
    );
    support::spawn_wall_2d(
        commands,
        config,
        "South Wall",
        IVec2::new(6, 6),
        UVec2::new(12, 2),
        Color::srgb(0.32, 0.30, 0.28),
    );
    support::spawn_wall_2d(
        commands,
        config,
        "East Pillar",
        IVec2::new(18, 8),
        UVec2::new(2, 6),
        Color::srgb(0.38, 0.34, 0.30),
    );

    let scout = support::spawn_source_2d(
        commands,
        config,
        "Scout",
        IVec2::new(8, 8),
        VisionSource::circle(FogLayerId(0), support::CELL_SIZE_2D * 4.5),
        Color::srgb(0.35, 0.92, 0.70),
    );
    commands.entity(scout).insert(Scout);

    support::spawn_overlay_2d(
        commands,
        FogLayerId(0),
        config.grid.origin,
        config.grid.world_size(),
        support::layer_palette(0.94, 0.72),
    );
}

fn animate_scout(time: Res<Time>, mut scout: Query<&mut Transform, With<Scout>>) {
    let Ok(mut transform) = scout.single_mut() else {
        return;
    };
    let phase = time.elapsed_secs() * 0.55;
    transform.translation.x = 240.0 + phase.sin() * 220.0;
    transform.translation.y = 256.0 + (phase * 1.4).cos() * 84.0;
}
