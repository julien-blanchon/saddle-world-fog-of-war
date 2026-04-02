use saddle_world_fog_of_war_example_support as support;

use bevy::prelude::*;
use saddle_world_fog_of_war::{FogLayerId, FogOfWarPlugin, VisionSource};

#[derive(Component)]
struct Observer;

fn main() {
    let config = support::config_2d(UVec2::new(24, 18));

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "fog_of_war occlusion_2d".into(),
                resolution: (1220, 820).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(FogOfWarPlugin::default().with_config(config.clone()))
        .add_systems(Startup, move |mut commands: Commands| {
            setup(&mut commands, &config)
        })
        .add_systems(Update, sweep_observer)
        .run();
}

fn setup(commands: &mut Commands, config: &saddle_world_fog_of_war::FogOfWarConfig) {
    support::spawn_2d_camera(commands, config);
    support::spawn_2d_backdrop(commands, config, Color::srgb(0.09, 0.10, 0.11));

    support::spawn_wall_2d(
        commands,
        config,
        "Center Wall",
        IVec2::new(11, 2),
        UVec2::new(2, 14),
        Color::srgb(0.43, 0.40, 0.36),
    );
    support::spawn_wall_2d(
        commands,
        config,
        "Left Cover",
        IVec2::new(7, 7),
        UVec2::new(2, 4),
        Color::srgb(0.36, 0.34, 0.30),
    );
    support::spawn_wall_2d(
        commands,
        config,
        "Right Cover",
        IVec2::new(15, 5),
        UVec2::new(2, 6),
        Color::srgb(0.36, 0.34, 0.30),
    );

    let observer = support::spawn_source_2d(
        commands,
        config,
        "Observer",
        IVec2::new(6, 8),
        VisionSource::circle(FogLayerId(0), support::CELL_SIZE_2D * 4.2),
        Color::srgb(0.96, 0.83, 0.36),
    );
    commands.entity(observer).insert(Observer);

    support::spawn_overlay_2d(
        commands,
        FogLayerId(0),
        config.grid.origin,
        config.grid.world_size(),
        support::layer_palette(0.96, 0.78),
    );
}

fn sweep_observer(time: Res<Time>, mut observer: Query<&mut Transform, With<Observer>>) {
    let Ok(mut transform) = observer.single_mut() else {
        return;
    };
    let phase = time.elapsed_secs() * 0.7;
    transform.translation.x = 180.0 + phase.sin() * 40.0;
    transform.translation.y = 288.0 + (phase * 1.6).sin() * 160.0;
}
