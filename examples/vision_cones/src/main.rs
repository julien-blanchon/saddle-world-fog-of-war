use saddle_world_fog_of_war_example_support as support;

use bevy::prelude::*;
use saddle_pane::prelude::*;
use saddle_world_fog_of_war::{FogLayerId, FogOfWarMap, FogOfWarPlugin, FogOverlay2d, VisionSource};

#[derive(Component)]
struct Sensor;

#[derive(Resource, Debug, Clone, Copy, Pane)]
#[pane(title = "Vision Cones", position = "top-right")]
struct ConePane {
    #[pane]
    pause_motion: bool,
    #[pane(slider, min = 2.0, max = 8.0, step = 0.1)]
    radius_cells: f32,
    #[pane(slider, min = 0.2, max = 1.6, step = 0.02)]
    spread_radians: f32,
    #[pane(slider, min = 0.1, max = 1.2, step = 0.02)]
    spin_speed: f32,
    #[pane(slider, min = 0.0, max = 0.6, step = 0.01)]
    edge_softness: f32,
    #[pane(monitor)]
    visible_cells: usize,
    #[pane(monitor)]
    explored_cells: usize,
}

impl Default for ConePane {
    fn default() -> Self {
        Self {
            pause_motion: false,
            radius_cells: 5.5,
            spread_radians: 1.0,
            spin_speed: 0.65,
            edge_softness: 0.35,
            visible_cells: 0,
            explored_cells: 0,
        }
    }
}

fn main() {
    let config = support::config_2d(UVec2::new(26, 18));

    App::new()
        .insert_resource(ConePane::default())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "fog_of_war vision_cones".into(),
                resolution: (1280, 820).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins((
            bevy_flair::FlairPlugin,
            bevy_input_focus::InputDispatchPlugin,
            bevy_ui_widgets::UiWidgetsPlugins,
            bevy_input_focus::tab_navigation::TabNavigationPlugin,
            PanePlugin,
        ))
        .register_pane::<ConePane>()
        .add_plugins(FogOfWarPlugin::default().with_config(config.clone()))
        .add_systems(Startup, move |mut commands: Commands| {
            setup(&mut commands, &config)
        })
        .add_systems(
            Update,
            sync_controls.before(saddle_world_fog_of_war::FogOfWarSystems::CollectVisionSources),
        )
        .add_systems(Update, spin_sensor)
        .add_systems(
            Update,
            update_pane.after(saddle_world_fog_of_war::FogOfWarSystems::UpdateExplorationMemory),
        )
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

fn spin_sensor(
    time: Res<Time>,
    pane: Res<ConePane>,
    mut sensor: Query<&mut VisionSource, With<Sensor>>,
) {
    let Ok(mut source) = sensor.single_mut() else {
        return;
    };
    let angle = if pane.pause_motion {
        0.0
    } else {
        time.elapsed_secs() * pane.spin_speed
    };
    source.shape = support::moving_arc_shape(angle, pane.radius_cells, pane.spread_radians);
}

fn sync_controls(
    pane: Res<ConePane>,
    mut overlay: Single<&mut FogOverlay2d>,
) {
    if !pane.is_changed() {
        return;
    }

    overlay.edge_softness = pane.edge_softness;
}

fn update_pane(map: Res<FogOfWarMap>, mut pane: ResMut<ConePane>) {
    let summary = map.layer_summary(FogLayerId(0)).unwrap_or(saddle_world_fog_of_war::FogLayerSummary {
        visible_cells: 0,
        explored_cells: 0,
    });
    pane.visible_cells = summary.visible_cells;
    pane.explored_cells = summary.explored_cells;
}
