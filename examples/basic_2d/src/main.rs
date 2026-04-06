use saddle_world_fog_of_war_example_support as support;

use bevy::prelude::*;
use saddle_pane::prelude::*;
use saddle_world_fog_of_war::{
    FogLayerId, FogOfWarMap, FogOfWarPlugin, FogOfWarRenderingPlugin, FogOverlay2d,
    FogRevealShape, VisionSource,
};

#[derive(Component)]
struct Scout;

#[derive(Resource, Debug, Clone, Copy, Pane)]
#[pane(title = "Basic Fog 2D", position = "top-right")]
struct BasicFogPane {
    #[pane]
    pause_motion: bool,
    #[pane(slider, min = 2.0, max = 8.0, step = 0.1)]
    scout_radius_cells: f32,
    #[pane(slider, min = 0.1, max = 1.2, step = 0.02)]
    scout_speed: f32,
    #[pane(slider, min = 0.0, max = 0.6, step = 0.01)]
    edge_softness: f32,
    #[pane(monitor)]
    visible_cells: usize,
    #[pane(monitor)]
    explored_cells: usize,
}

impl Default for BasicFogPane {
    fn default() -> Self {
        Self {
            pause_motion: false,
            scout_radius_cells: 4.5,
            scout_speed: 0.55,
            edge_softness: 0.35,
            visible_cells: 0,
            explored_cells: 0,
        }
    }
}

fn main() {
    let config = support::config_2d(UVec2::new(28, 18));

    App::new()
        .insert_resource(BasicFogPane::default())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "fog_of_war basic_2d".into(),
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
        .register_pane::<BasicFogPane>()
        .add_plugins((
            FogOfWarPlugin::default().with_config(config.clone()),
            FogOfWarRenderingPlugin::default(),
        ))
        .add_systems(Startup, move |mut commands: Commands| {
            setup(&mut commands, &config)
        })
        .add_systems(
            Update,
            sync_controls.before(saddle_world_fog_of_war::FogOfWarSystems::CollectVisionSources),
        )
        .add_systems(Update, animate_scout)
        .add_systems(
            Update,
            update_pane.after(saddle_world_fog_of_war::FogOfWarSystems::ApplyPersistence),
        )
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
    support::spawn_instructions(
        commands,
        "Basic Fog 2D",
        "Use the pane in the top-right to pause the scout, tune reveal radius and speed, and soften the fog edge.\nWatch the trail behind the scout to compare current visibility against explored memory.",
    );
}

fn animate_scout(time: Res<Time>, pane: Res<BasicFogPane>, mut scout: Query<&mut Transform, With<Scout>>) {
    if pane.pause_motion {
        return;
    }
    let Ok(mut transform) = scout.single_mut() else {
        return;
    };
    let phase = time.elapsed_secs() * pane.scout_speed;
    transform.translation.x = 240.0 + phase.sin() * 220.0;
    transform.translation.y = 256.0 + (phase * 1.4).cos() * 84.0;
}

fn sync_controls(
    pane: Res<BasicFogPane>,
    mut scout: Single<&mut VisionSource, With<Scout>>,
    mut overlay: Single<&mut FogOverlay2d>,
) {
    if !pane.is_changed() {
        return;
    }

    scout.shape = FogRevealShape::circle(pane.scout_radius_cells * support::CELL_SIZE_2D);
    overlay.edge_softness = pane.edge_softness;
}

fn update_pane(map: Res<FogOfWarMap>, mut pane: ResMut<BasicFogPane>) {
    let summary = map.layer_summary(FogLayerId(0)).unwrap_or(saddle_world_fog_of_war::FogLayerSummary {
        visible_cells: 0,
        explored_cells: 0,
    });
    pane.visible_cells = summary.visible_cells;
    pane.explored_cells = summary.explored_cells;
}
