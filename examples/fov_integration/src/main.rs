use std::collections::HashSet;

use bevy::prelude::*;
use saddle_ai_fov::{FovPlugin, GridFov, GridFovState, GridMapSpec, GridOpacityMap};
use saddle_pane::prelude::*;
use saddle_world_fog_of_war::{
    FogLayerId, FogLayerMask, FogOfWarPlugin, FogOfWarRenderingPlugin, FogOverlay2d,
    VisionCellSource, VisionOccluder,
};
use saddle_world_fog_of_war_example_support as support;

const DEMO_GRID: &[&str] = &[
    "###############",
    "#.......#.....#",
    "#.#####.#.###.#",
    "#.#...#.#...#.#",
    "#.#.#.#.###.#.#",
    "#...#.#.....#.#",
    "###.#.#####.#.#",
    "#...#.....#...#",
    "#.#######.###.#",
    "#.............#",
    "###############",
];

const SCOUT_PATH: &[IVec2] = &[
    IVec2::new(2, 8),
    IVec2::new(2, 2),
    IVec2::new(6, 2),
    IVec2::new(6, 7),
    IVec2::new(10, 7),
    IVec2::new(12, 3),
    IVec2::new(12, 8),
    IVec2::new(2, 8),
];

#[derive(Component)]
struct ReconScout;

#[derive(Component)]
struct GridCellSprite(IVec2);

#[derive(Component)]
struct FogOverlayMarker;

#[derive(Resource, Debug, Clone, Copy, Pane)]
#[pane(title = "FOV -> Fog", position = "top-right")]
struct IntegrationPane {
    #[pane]
    pause_motion: bool,
    #[pane(slider, min = 2.0, max = 8.0, step = 1.0)]
    scout_radius: i32,
    #[pane(slider, min = 0.08, max = 1.0, step = 0.02)]
    scout_speed: f32,
    #[pane(slider, min = 0.0, max = 0.6, step = 0.01)]
    edge_softness: f32,
    #[pane(monitor)]
    visible_cells: usize,
    #[pane(monitor)]
    explored_cells: usize,
}

impl Default for IntegrationPane {
    fn default() -> Self {
        Self {
            pause_motion: false,
            scout_radius: 4,
            scout_speed: 0.36,
            edge_softness: 0.28,
            visible_cells: 0,
            explored_cells: 0,
        }
    }
}

fn main() {
    let fog_config = support::config_2d(UVec2::new(
        DEMO_GRID[0].len() as u32,
        DEMO_GRID.len() as u32,
    ));

    App::new()
        .insert_resource(ClearColor(Color::srgb(0.025, 0.028, 0.035)))
        .insert_resource(build_grid_map())
        .insert_resource(IntegrationPane::default())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "fog_of_war fov_integration".into(),
                resolution: (1280, 860).into(),
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
        .register_pane::<IntegrationPane>()
        .add_plugins(FovPlugin::default())
        .add_plugins((
            FogOfWarPlugin::default().with_config(fog_config.clone()),
            FogOfWarRenderingPlugin::default(),
        ))
        .configure_sets(
            Update,
            saddle_ai_fov::FovSystems::Recompute
                .before(saddle_world_fog_of_war::FogOfWarSystems::CollectVisionSources),
        )
        .add_systems(Startup, move |mut commands: Commands| {
            setup(&mut commands, &fog_config);
        })
        .add_systems(
            Update,
            animate_scout.before(saddle_ai_fov::FovSystems::MarkDirty),
        )
        .add_systems(
            Update,
            sync_controls.before(saddle_ai_fov::FovSystems::MarkDirty),
        )
        .add_systems(
            Update,
            sync_fov_to_fog
                .after(saddle_ai_fov::FovSystems::Recompute)
                .before(saddle_world_fog_of_war::FogOfWarSystems::CollectVisionSources),
        )
        .add_systems(
            Update,
            update_tiles.after(saddle_ai_fov::FovSystems::Recompute),
        )
        .add_systems(
            Update,
            update_monitors.after(saddle_world_fog_of_war::FogOfWarSystems::ApplyPersistence),
        )
        .run();
}

fn build_grid_map() -> GridOpacityMap {
    let spec = GridMapSpec {
        origin: Vec2::ZERO,
        dimensions: UVec2::new(DEMO_GRID[0].len() as u32, DEMO_GRID.len() as u32),
        cell_size: Vec2::splat(support::CELL_SIZE_2D),
    };

    GridOpacityMap::from_fn(spec, |cell| {
        DEMO_GRID[cell.y as usize].as_bytes()[cell.x as usize] == b'#'
    })
}

fn setup(commands: &mut Commands, fog_config: &saddle_world_fog_of_war::FogOfWarConfig) {
    let grid = build_grid_map();
    let world_size = fog_config.grid.world_size();

    support::spawn_2d_camera(commands, fog_config);
    support::spawn_2d_backdrop(commands, fog_config, Color::srgb(0.07, 0.08, 0.09));

    spawn_level_tiles(commands, &grid);
    spawn_poi(
        commands,
        &fog_config.grid,
        "Extraction Lift",
        IVec2::new(12, 2),
        Color::srgb(0.95, 0.74, 0.28),
    );
    spawn_poi(
        commands,
        &fog_config.grid,
        "Intel Cache",
        IVec2::new(6, 6),
        Color::srgb(0.34, 0.80, 0.98),
    );

    let scout_start = grid
        .spec
        .cell_to_world_center(SCOUT_PATH[0])
        .expect("scout path must start inside the grid")
        .extend(5.0);
    commands.spawn((
        Name::new("Recon Scout"),
        ReconScout,
        GridFov::new(4),
        VisionCellSource::new(FogLayerMask::bit(FogLayerId(0))),
        Sprite::from_color(Color::srgb(0.40, 0.96, 0.72), Vec2::splat(grid.spec.cell_size.x * 0.58)),
        Transform::from_translation(scout_start),
        GlobalTransform::from_translation(scout_start),
    ));

    commands.spawn((
        Name::new("Fog Overlay"),
        FogOverlayMarker,
        FogOverlay2d {
            layer: FogLayerId(0),
            world_origin: fog_config.grid.origin,
            world_size,
            palette: support::layer_palette(0.94, 0.72),
            opacity: 1.0,
            edge_softness: 0.28,
            z: 9.0,
        },
    ));

    support::spawn_instructions(
        commands,
        "FOV Integration",
        "Use the pane in the top-right to pause the recon route, adjust the grid-FOV radius and speed, and soften the fog edge.\nThis scene uses saddle-ai-fov for the exact visible cells, then feeds those cells into fog-of-war memory.",
    );
}

fn spawn_level_tiles(commands: &mut Commands, grid: &GridOpacityMap) {
    for y in 0..grid.spec.dimensions.y as i32 {
        for x in 0..grid.spec.dimensions.x as i32 {
            let cell = IVec2::new(x, y);
            let center = grid
                .spec
                .cell_to_world_center(cell)
                .expect("demo grid cells must stay in bounds");
            let opaque = grid.is_opaque(cell);
            let size = grid.spec.cell_size - Vec2::splat(1.5);

            let mut entity = commands.spawn((
                Name::new(format!("Grid Cell {x},{y}")),
                GridCellSprite(cell),
                Sprite::from_color(
                    if opaque {
                        Color::srgb(0.18, 0.19, 0.22)
                    } else {
                        Color::srgb(0.09, 0.10, 0.12)
                    },
                    size,
                ),
                Transform::from_translation(center.extend(if opaque { 1.0 } else { 0.0 })),
            ));
            if opaque {
                entity.insert(VisionOccluder::cell(FogLayerMask::ALL));
            }
        }
    }
}

fn spawn_poi(
    commands: &mut Commands,
    spec: &saddle_world_fog_of_war::FogGridSpec,
    name: &str,
    cell: IVec2,
    color: Color,
) {
    let position = spec
        .cell_to_world_center(cell)
        .expect("poi cells should be in bounds")
        .extend(3.0);
    commands.spawn((
        Name::new(name.to_string()),
        Sprite::from_color(color, spec.cell_size * 0.36),
        Transform::from_translation(position),
    ));
}

fn animate_scout(
    time: Res<Time>,
    pane: Res<IntegrationPane>,
    grid: Res<GridOpacityMap>,
    mut scout: Single<(&mut Transform, &mut GlobalTransform), With<ReconScout>>,
) {
    if pane.pause_motion {
        return;
    }

    let position = sample_path(
        &grid.spec,
        SCOUT_PATH,
        time.elapsed_secs(),
        pane.scout_speed,
        5.0,
    );
    scout.0.translation = position;
    *scout.1 = GlobalTransform::from_translation(position);
}

fn sync_controls(
    pane: Res<IntegrationPane>,
    mut scout: Single<&mut GridFov, With<ReconScout>>,
    mut overlay: Single<&mut FogOverlay2d, With<FogOverlayMarker>>,
) {
    if !pane.is_changed() {
        return;
    }

    scout.config.radius = pane.scout_radius.max(0);
    overlay.edge_softness = pane.edge_softness;
}

fn sync_fov_to_fog(
    scout: Single<(&GridFovState, &mut VisionCellSource), With<ReconScout>>,
) {
    let (fov_state, mut fog_source) = scout.into_inner();
    fog_source.cells.clone_from(&fov_state.visible_now);
}

fn update_tiles(
    grid: Res<GridOpacityMap>,
    scout: Single<&GridFovState, With<ReconScout>>,
    mut tiles: Query<(&GridCellSprite, &mut Sprite)>,
) {
    let visible: HashSet<_> = scout.visible_now.iter().copied().collect();
    let explored: HashSet<_> = scout.explored.iter().copied().collect();

    for (cell, mut sprite) in &mut tiles {
        sprite.color = if visible.contains(&cell.0) {
            if grid.is_opaque(cell.0) {
                Color::srgb(0.78, 0.72, 0.50)
            } else {
                Color::srgb(0.22, 0.66, 0.90)
            }
        } else if explored.contains(&cell.0) {
            if grid.is_opaque(cell.0) {
                Color::srgb(0.28, 0.27, 0.24)
            } else {
                Color::srgb(0.14, 0.18, 0.20)
            }
        } else if grid.is_opaque(cell.0) {
            Color::srgb(0.18, 0.19, 0.22)
        } else {
            Color::srgb(0.09, 0.10, 0.12)
        };
    }
}

fn update_monitors(
    scout: Single<&GridFovState, With<ReconScout>>,
    mut pane: ResMut<IntegrationPane>,
) {
    pane.visible_cells = scout.visible_now.len();
    pane.explored_cells = scout.explored.len();
}

fn sample_path(spec: &GridMapSpec, path: &[IVec2], elapsed_secs: f32, speed: f32, z: f32) -> Vec3 {
    let progress = elapsed_secs * speed;
    let from = progress.floor() as usize % path.len();
    let to = (from + 1) % path.len();
    let t = progress.fract();
    let start = spec
        .cell_to_world_center(path[from])
        .expect("path cells must be in bounds")
        .extend(z);
    let end = spec
        .cell_to_world_center(path[to])
        .expect("path cells must be in bounds")
        .extend(z);
    start.lerp(end, t)
}
