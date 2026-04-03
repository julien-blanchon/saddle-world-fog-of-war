use saddle_world_fog_of_war_example_support as support;

use bevy::prelude::*;
use saddle_pane::prelude::*;
use saddle_world_fog_of_war::{
    FogLayerId, FogOfWarMap, FogOfWarPlugin, FogOverlay2d, FogRevealShape, VisionSource,
};

#[derive(Component)]
struct ScoutRing {
    radius: f32,
    speed: f32,
    offset: f32,
}

#[derive(Resource, Debug, Clone, Copy, Pane)]
#[pane(title = "RTS Fog Map", position = "top-right")]
struct RtsPane {
    #[pane]
    pause_motion: bool,
    #[pane(slider, min = 2.0, max = 8.0, step = 0.1)]
    scout_radius_cells: f32,
    #[pane(slider, min = 0.2, max = 2.0, step = 0.05)]
    speed_scale: f32,
    #[pane(slider, min = 0.0, max = 0.6, step = 0.01)]
    edge_softness: f32,
    #[pane(monitor)]
    visible_cells: usize,
    #[pane(monitor)]
    explored_cells: usize,
}

impl Default for RtsPane {
    fn default() -> Self {
        Self {
            pause_motion: false,
            scout_radius_cells: 3.2,
            speed_scale: 1.0,
            edge_softness: 0.35,
            visible_cells: 0,
            explored_cells: 0,
        }
    }
}

fn main() {
    let config = support::config_2d(UVec2::new(96, 64));

    App::new()
        .insert_resource(RtsPane::default())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "fog_of_war rts_large_map".into(),
                resolution: (1680, 960).into(),
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
        .register_pane::<RtsPane>()
        .add_plugins(FogOfWarPlugin::default().with_config(config.clone()))
        .add_systems(Startup, move |mut commands: Commands| {
            setup(&mut commands, &config)
        })
        .add_systems(
            Update,
            sync_controls.before(saddle_world_fog_of_war::FogOfWarSystems::CollectVisionSources),
        )
        .add_systems(Update, orbit_scouts)
        .add_systems(
            Update,
            update_pane.after(saddle_world_fog_of_war::FogOfWarSystems::UpdateExplorationMemory),
        )
        .run();
}

fn setup(commands: &mut Commands, config: &saddle_world_fog_of_war::FogOfWarConfig) {
    support::spawn_2d_camera(commands, config);
    support::spawn_2d_backdrop(commands, config, Color::srgb(0.07, 0.10, 0.08));

    for x in [18, 28, 50, 67, 81] {
        support::spawn_wall_2d(
            commands,
            config,
            format!("Rock Ridge {x}"),
            IVec2::new(x, 18),
            UVec2::new(3, 24),
            Color::srgb(0.22, 0.23, 0.21),
        );
    }

    let center = config.grid.world_size() * 0.5;
    for index in 0..12 {
        let entity = support::spawn_source_2d(
            commands,
            config,
            format!("Scout {}", index + 1),
            IVec2::new(10 + index, 10 + (index % 6)),
            VisionSource::circle(FogLayerId(0), support::CELL_SIZE_2D * 3.2),
            Color::srgb(0.42, 0.90, 0.64),
        );
        commands.entity(entity).insert(ScoutRing {
            radius: 220.0 + (index % 4) as f32 * 90.0,
            speed: 0.10 + index as f32 * 0.015,
            offset: index as f32 * 0.52,
        });
        commands
            .entity(entity)
            .insert(Transform::from_translation(Vec3::new(
                center.x, center.y, 5.0,
            )));
    }

    for (index, cell) in [(22, 22), (54, 40), (76, 18)].into_iter().enumerate() {
        support::spawn_source_2d(
            commands,
            config,
            format!("Watchtower {}", index + 1),
            IVec2::new(cell.0, cell.1),
            VisionSource::circle(FogLayerId(0), support::CELL_SIZE_2D * 6.0),
            Color::srgb(0.94, 0.76, 0.34),
        );
    }

    support::spawn_overlay_2d(
        commands,
        FogLayerId(0),
        config.grid.origin,
        config.grid.world_size(),
        support::layer_palette(0.94, 0.72),
    );
    support::spawn_overlay_2d(
        commands,
        FogLayerId(0),
        Vec2::new(config.grid.world_size().x - 420.0, 20.0),
        Vec2::new(400.0, 266.0),
        support::layer_palette(0.82, 0.58),
    );
}

fn orbit_scouts(
    time: Res<Time>,
    pane: Res<RtsPane>,
    mut scouts: Query<(&ScoutRing, &mut Transform)>,
) {
    if pane.pause_motion {
        return;
    }
    let center = Vec2::new(
        96.0 * support::CELL_SIZE_2D * 0.5,
        64.0 * support::CELL_SIZE_2D * 0.5,
    );
    for (ring, mut transform) in &mut scouts {
        let angle = time.elapsed_secs() * ring.speed * pane.speed_scale + ring.offset;
        transform.translation.x = center.x + angle.cos() * ring.radius;
        transform.translation.y = center.y + angle.sin() * (ring.radius * 0.55);
    }
}

fn sync_controls(
    pane: Res<RtsPane>,
    mut scouts: Query<&mut VisionSource, With<ScoutRing>>,
    mut overlays: Query<&mut FogOverlay2d>,
) {
    if !pane.is_changed() {
        return;
    }

    for mut scout in &mut scouts {
        scout.shape = FogRevealShape::circle(pane.scout_radius_cells * support::CELL_SIZE_2D);
    }
    for mut overlay in &mut overlays {
        overlay.edge_softness = pane.edge_softness;
    }
}

fn update_pane(map: Res<FogOfWarMap>, mut pane: ResMut<RtsPane>) {
    let summary = map.layer_summary(FogLayerId(0)).unwrap_or(saddle_world_fog_of_war::FogLayerSummary {
        visible_cells: 0,
        explored_cells: 0,
    });
    pane.visible_cells = summary.visible_cells;
    pane.explored_cells = summary.explored_cells;
}
