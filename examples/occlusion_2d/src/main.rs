use saddle_world_fog_of_war_example_support as support;

#[cfg(feature = "e2e")]
mod scenarios;

use bevy::prelude::*;
use saddle_pane::prelude::*;
use saddle_world_fog_of_war::{
    FogLayerId, FogOfWarMap, FogOfWarPlugin, FogOfWarRenderingPlugin, FogOverlay2d, FogRevealShape,
    VisionSource,
};

#[derive(Component)]
struct Observer;

#[derive(Resource, Debug, Clone, Copy, Pane)]
#[pane(title = "Occlusion 2D", position = "top-right")]
struct OcclusionPane {
    #[pane]
    pause_motion: bool,
    #[pane(slider, min = 2.0, max = 8.0, step = 0.1)]
    observer_radius_cells: f32,
    #[pane(slider, min = 0.1, max = 1.4, step = 0.02)]
    observer_speed: f32,
    #[pane(slider, min = 0.0, max = 0.6, step = 0.01)]
    edge_softness: f32,
    #[pane(monitor)]
    visible_cells: usize,
    #[pane(monitor)]
    explored_cells: usize,
}

impl Default for OcclusionPane {
    fn default() -> Self {
        Self {
            pause_motion: false,
            observer_radius_cells: 4.2,
            observer_speed: 0.7,
            edge_softness: 0.35,
            visible_cells: 0,
            explored_cells: 0,
        }
    }
}

fn main() {
    let config = support::config_2d(UVec2::new(24, 18));

    let mut app = App::new();
    app.insert_resource(OcclusionPane::default());
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "fog_of_war occlusion_2d".into(),
            resolution: (1220, 820).into(),
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
    .register_pane::<OcclusionPane>();
    app.add_plugins((
        FogOfWarPlugin::default().with_config(config.clone()),
        FogOfWarRenderingPlugin::default(),
    ));
    app.add_systems(Startup, move |mut commands: Commands| {
        setup(&mut commands, &config)
    });
    app.add_systems(
        Update,
        sync_controls.before(saddle_world_fog_of_war::FogOfWarSystems::CollectVisionSources),
    );
    app.add_systems(Update, sweep_observer);
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
    support::spawn_instructions(
        commands,
        "Occlusion 2D",
        "Use the pane in the top-right to pause the observer, change reveal radius and sweep speed, and soften the fog edge.\nSlide the observer around the walls to inspect how LOS blockers keep cells hidden behind cover.",
    );
}

fn sweep_observer(
    time: Res<Time>,
    pane: Res<OcclusionPane>,
    mut observer: Query<&mut Transform, With<Observer>>,
) {
    if pane.pause_motion {
        return;
    }
    let Ok(mut transform) = observer.single_mut() else {
        return;
    };
    let phase = time.elapsed_secs() * pane.observer_speed;
    transform.translation.x = 180.0 + phase.sin() * 40.0;
    transform.translation.y = 288.0 + (phase * 1.6).sin() * 160.0;
}

fn sync_controls(
    pane: Res<OcclusionPane>,
    mut observer: Single<&mut VisionSource, With<Observer>>,
    mut overlay: Single<&mut FogOverlay2d>,
) {
    if !pane.is_changed() {
        return;
    }

    observer.shape = FogRevealShape::circle(pane.observer_radius_cells * support::CELL_SIZE_2D);
    overlay.edge_softness = pane.edge_softness;
}

fn update_pane(map: Res<FogOfWarMap>, mut pane: ResMut<OcclusionPane>) {
    let summary =
        map.layer_summary(FogLayerId(0))
            .unwrap_or(saddle_world_fog_of_war::FogLayerSummary {
                visible_cells: 0,
                explored_cells: 0,
            });
    pane.visible_cells = summary.visible_cells;
    pane.explored_cells = summary.explored_cells;
}
