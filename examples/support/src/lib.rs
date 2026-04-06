use bevy::prelude::*;
use saddle_world_fog_of_war::{
    FogLayerId, FogLayerMask, FogOfWarConfig, FogOverlay2d, FogPalette, FogProjectionReceiver,
    FogRevealShape, FogWorldAxes, VisionOccluder, VisionSource,
};

pub const CELL_SIZE_2D: f32 = 32.0;

pub fn config_2d(dimensions: UVec2) -> FogOfWarConfig {
    let mut config = FogOfWarConfig::default();
    config.grid.origin = Vec2::ZERO;
    config.grid.dimensions = dimensions;
    config.grid.cell_size = Vec2::splat(CELL_SIZE_2D);
    config.grid.chunk_size = UVec2::splat(16);
    config.world_axes = FogWorldAxes::XY;
    config
}

pub fn config_3d(dimensions: UVec2) -> FogOfWarConfig {
    let mut config = FogOfWarConfig::default();
    config.grid.origin = Vec2::ZERO;
    config.grid.dimensions = dimensions;
    config.grid.cell_size = Vec2::splat(1.0);
    config.grid.chunk_size = UVec2::splat(8);
    config.world_axes = FogWorldAxes::XZ;
    config
}

pub fn spawn_2d_camera(commands: &mut Commands, config: &FogOfWarConfig) {
    let center = config.grid.origin + config.grid.world_size() * 0.5;
    commands.spawn((
        Name::new("Example Camera"),
        Camera2d,
        Transform::from_xyz(center.x, center.y, 1000.0),
    ));
}

pub fn spawn_2d_backdrop(commands: &mut Commands, config: &FogOfWarConfig, color: Color) {
    let center = config.grid.origin + config.grid.world_size() * 0.5;
    commands.spawn((
        Name::new("Map Backdrop"),
        Sprite::from_color(color, config.grid.world_size()),
        Transform::from_translation(center.extend(-2.0)),
    ));
}

pub fn spawn_wall_2d(
    commands: &mut Commands,
    config: &FogOfWarConfig,
    name: impl Into<String>,
    min_cell: IVec2,
    size_cells: UVec2,
    color: Color,
) {
    let size_world = config.grid.cell_size * size_cells.as_vec2();
    let origin = config.grid.origin + min_cell.as_vec2() * config.grid.cell_size;
    let center = origin + size_world * 0.5;
    commands.spawn((
        Name::new(name.into()),
        VisionOccluder::rect(FogLayerMask::ALL, size_world * 0.5),
        Sprite::from_color(color, size_world),
        Transform::from_translation(center.extend(1.0)),
    ));
}

pub fn spawn_source_2d(
    commands: &mut Commands,
    config: &FogOfWarConfig,
    name: impl Into<String>,
    cell: IVec2,
    source: VisionSource,
    color: Color,
) -> Entity {
    commands
        .spawn((
            Name::new(name.into()),
            source,
            Sprite::from_color(color, config.grid.cell_size * 0.55),
            Transform::from_translation(
                config
                    .grid
                    .cell_to_world_center(cell)
                    .expect("cell should be in bounds")
                    .extend(5.0),
            ),
        ))
        .id()
}

pub fn spawn_overlay_2d(
    commands: &mut Commands,
    layer: FogLayerId,
    origin: Vec2,
    size: Vec2,
    palette: FogPalette,
) {
    commands.spawn((
        Name::new("Fog Overlay 2D"),
        FogOverlay2d {
            layer,
            world_origin: origin,
            world_size: size,
            palette,
            opacity: 1.0,
            edge_softness: 0.35,
            z: 10.0,
        },
    ));
}

pub fn spawn_3d_camera(commands: &mut Commands, config: &FogOfWarConfig) {
    let center = config.grid.origin + config.grid.world_size() * 0.5;
    commands.spawn((
        Name::new("Projection Camera"),
        Camera3d::default(),
        Transform::from_xyz(center.x - 8.0, 18.0, center.y + 10.0)
            .looking_at(Vec3::new(center.x, 0.0, center.y), Vec3::Y),
    ));
    commands.spawn((
        Name::new("Projection Sun"),
        DirectionalLight {
            illuminance: 18_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(center.x - 6.0, 18.0, center.y - 4.0)
            .looking_at(Vec3::new(center.x, 0.0, center.y), Vec3::Y),
    ));
}

pub fn spawn_ground_3d(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    config: &FogOfWarConfig,
) {
    let center = config.grid.origin + config.grid.world_size() * 0.5;
    commands.spawn((
        Name::new("Projection Ground"),
        Mesh3d(
            meshes.add(
                Plane3d::default()
                    .mesh()
                    .size(config.grid.world_size().x, config.grid.world_size().y),
            ),
        ),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.23, 0.27, 0.23),
            perceptual_roughness: 1.0,
            ..default()
        })),
        Transform::from_xyz(center.x, 0.0, center.y),
    ));
}

pub fn spawn_wall_3d(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    name: impl Into<String>,
    position: Vec2,
    size: Vec2,
    height: f32,
    color: Color,
) {
    commands.spawn((
        Name::new(name.into()),
        VisionOccluder::rect(FogLayerMask::ALL, size * 0.5),
        Mesh3d(meshes.add(Cuboid::new(size.x, height, size.y))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: color,
            perceptual_roughness: 0.92,
            ..default()
        })),
        Transform::from_xyz(position.x, height * 0.5, position.y),
    ));
}

pub fn spawn_source_3d(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    name: impl Into<String>,
    position: Vec2,
    source: VisionSource,
    color: Color,
) -> Entity {
    commands
        .spawn((
            Name::new(name.into()),
            source,
            Mesh3d(meshes.add(Sphere::new(0.22).mesh().uv(16, 9))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                emissive: LinearRgba::from(color) * 0.2,
                ..default()
            })),
            Transform::from_xyz(position.x, 0.25, position.y),
        ))
        .id()
}

pub fn spawn_projection_receiver(
    commands: &mut Commands,
    layer: FogLayerId,
    origin: Vec2,
    size: Vec2,
    palette: FogPalette,
) {
    commands.spawn((
        Name::new("Fog Projection Receiver"),
        FogProjectionReceiver {
            layer,
            world_origin: origin,
            world_size: size,
            palette,
            opacity: 1.0,
            edge_softness: 0.3,
            elevation: 0.05,
        },
    ));
}

pub fn layer_palette(hidden_alpha: f32, explored_alpha: f32) -> FogPalette {
    FogPalette {
        hidden: LinearRgba::new(0.03, 0.05, 0.08, hidden_alpha),
        explored: LinearRgba::new(0.22, 0.25, 0.30, explored_alpha),
        visible: LinearRgba::new(0.0, 0.0, 0.0, 0.0),
    }
}

pub fn moving_arc_shape(angle: f32, radius_cells: f32, spread_radians: f32) -> FogRevealShape {
    FogRevealShape::arc(
        radius_cells * CELL_SIZE_2D,
        spread_radians,
        Vec2::from_angle(angle),
    )
}

pub fn spawn_instructions(commands: &mut Commands, name: &str, body: &str) {
    commands.spawn((
        Name::new(format!("{name} Instructions")),
        Node {
            position_type: PositionType::Absolute,
            left: px(18.0),
            bottom: px(18.0),
            width: px(440.0),
            padding: UiRect::all(px(12.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.03, 0.04, 0.06, 0.84)),
        Text::new(body.to_string()),
        TextFont {
            font_size: 15.0,
            ..default()
        },
        TextColor(Color::WHITE),
    ));
}
