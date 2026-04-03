#[cfg(feature = "e2e")]
mod e2e;
#[cfg(feature = "e2e")]
mod scenarios;

use bevy::prelude::*;
#[cfg(feature = "dev")]
use bevy::remote::{RemotePlugin, http::RemoteHttpPlugin};
#[cfg(feature = "dev")]
use bevy_brp_extras::BrpExtrasPlugin;
use saddle_pane::prelude::*;
use saddle_world_fog_of_war::{
    FogLayerId, FogLayerSummary, FogOfWarConfig, FogOfWarMap, FogOfWarPlugin,
    FogProjectionReceiver, FogRevealShape, FogVisibilityState, FogWorldAxes, VisionOccluder,
    VisionSource,
};

pub const MEMORY_SAMPLE_CELL: IVec2 = IVec2::new(4, 4);
pub const OCCLUDED_SAMPLE_CELL: IVec2 = IVec2::new(14, 8);
pub const TEAM_ZERO_SAMPLE_CELL: IVec2 = IVec2::new(4, 4);
pub const TEAM_ONE_SAMPLE_CELL: IVec2 = IVec2::new(17, 6);

#[derive(Component)]
pub struct LabCamera;

#[derive(Component)]
struct ScoutAlpha;

#[derive(Component)]
struct ScoutBeta;

#[derive(Component)]
struct Sentry;

#[derive(Component)]
struct LabOverlay;

#[derive(Resource, Clone, Copy)]
#[allow(dead_code)]
struct LabEntities {
    scout_alpha: Entity,
    scout_beta: Entity,
    sentry: Entity,
    main_receiver: Entity,
    minimap_receiver: Entity,
}

#[derive(Resource, Clone, Copy, Pane)]
#[pane(title = "Fog Lab Controls", position = "top-right")]
pub struct LabControl {
    #[pane]
    pub pause_motion: bool,
    #[pane(skip)]
    pub selected_layer: FogLayerId,
    #[pane(slider, min = 2.0, max = 7.5, step = 0.1)]
    pub scout_alpha_radius: f32,
    #[pane(slider, min = 2.0, max = 7.5, step = 0.1)]
    pub scout_beta_radius: f32,
    #[pane(slider, min = 2.0, max = 8.0, step = 0.1)]
    pub sentry_radius: f32,
    #[pane(slider, min = 0.4, max = 1.8, step = 0.02)]
    pub sentry_spread: f32,
    #[pane(slider, min = 0.12, max = 0.8, step = 0.02)]
    pub scout_alpha_speed: f32,
    #[pane(slider, min = 0.12, max = 0.8, step = 0.02)]
    pub scout_beta_speed: f32,
    #[pane(slider, min = 0.1, max = 0.6, step = 0.02)]
    pub camera_orbit_speed: f32,
    #[pane(slider, min = 0.0, max = 0.6, step = 0.01)]
    pub main_edge_softness: f32,
}

impl Default for LabControl {
    fn default() -> Self {
        Self {
            pause_motion: false,
            selected_layer: FogLayerId(0),
            scout_alpha_radius: 4.8,
            scout_beta_radius: 4.0,
            sentry_radius: 5.5,
            sentry_spread: 1.2,
            scout_alpha_speed: 0.55,
            scout_beta_speed: 0.42,
            camera_orbit_speed: 0.18,
            main_edge_softness: 0.3,
        }
    }
}

#[derive(Resource, Debug, Clone)]
pub struct LabDiagnostics {
    pub selected_layer: FogLayerId,
    pub main_receiver_layer: FogLayerId,
    pub layer_zero: FogLayerSummary,
    pub layer_one: FogLayerSummary,
    pub memory_sample: FogVisibilityState,
    pub occluded_sample: FogVisibilityState,
    pub camera_pos: Vec3,
    pub scout_alpha_pos: Vec2,
    pub scout_beta_pos: Vec2,
    pub sentry_pos: Vec2,
}

impl Default for LabDiagnostics {
    fn default() -> Self {
        Self {
            selected_layer: FogLayerId(0),
            main_receiver_layer: FogLayerId(0),
            layer_zero: FogLayerSummary {
                visible_cells: 0,
                explored_cells: 0,
            },
            layer_one: FogLayerSummary {
                visible_cells: 0,
                explored_cells: 0,
            },
            memory_sample: FogVisibilityState::Hidden,
            occluded_sample: FogVisibilityState::Hidden,
            camera_pos: Vec3::ZERO,
            scout_alpha_pos: Vec2::ZERO,
            scout_beta_pos: Vec2::ZERO,
            sentry_pos: Vec2::ZERO,
        }
    }
}

fn main() {
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::srgb(0.06, 0.07, 0.08)));
    app.insert_resource(LabControl::default());
    app.insert_resource(LabDiagnostics::default());
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Fog Of War Lab".into(),
            resolution: (1440, 880).into(),
            ..default()
        }),
        ..default()
    }));
    #[cfg(feature = "dev")]
    app.add_plugins(RemotePlugin::default());
    #[cfg(feature = "dev")]
    app.add_plugins(BrpExtrasPlugin::with_http_plugin(
        RemoteHttpPlugin::default(),
    ));
    #[cfg(feature = "e2e")]
    app.add_plugins(e2e::FogOfWarLabE2EPlugin);

    app.add_plugins(FogOfWarPlugin::default().with_config(lab_config()));
    app.add_plugins((
        bevy_flair::FlairPlugin,
        bevy_input_focus::InputDispatchPlugin,
        bevy_ui_widgets::UiWidgetsPlugins,
        bevy_input_focus::tab_navigation::TabNavigationPlugin,
        PanePlugin,
    ))
        .register_pane::<LabControl>();
    app.add_systems(Startup, setup);
    app.add_systems(
        Update,
        (
            sync_demo_controls.before(saddle_world_fog_of_war::FogOfWarSystems::CollectVisionSources),
            animate_scout_alpha,
            animate_scout_beta,
            animate_sentry,
            orbit_camera,
            sync_receivers.before(saddle_world_fog_of_war::FogOfWarSystems::UploadRenderData),
            update_diagnostics.after(saddle_world_fog_of_war::FogOfWarSystems::UpdateExplorationMemory),
            update_overlay.after(saddle_world_fog_of_war::FogOfWarSystems::UploadRenderData),
        ),
    );
    app.run();
}

fn lab_config() -> FogOfWarConfig {
    let mut config = FogOfWarConfig::default();
    config.grid.origin = Vec2::ZERO;
    config.grid.dimensions = UVec2::new(24, 18);
    config.grid.cell_size = Vec2::splat(1.0);
    config.grid.chunk_size = UVec2::splat(8);
    config.world_axes = FogWorldAxes::XZ;
    config
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let config = lab_config();
    let world_size = config.grid.world_size();
    let center = world_size * 0.5;

    commands.spawn((
        Name::new("Lab Camera"),
        LabCamera,
        Camera3d::default(),
        Transform::from_xyz(center.x - 8.0, 16.0, center.y + 10.0)
            .looking_at(Vec3::new(center.x, 0.0, center.y), Vec3::Y),
    ));
    commands.spawn((
        Name::new("Lab Sun"),
        DirectionalLight {
            illuminance: 18_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(center.x - 4.0, 16.0, center.y - 2.0)
            .looking_at(Vec3::new(center.x, 0.0, center.y), Vec3::Y),
    ));
    commands.spawn((
        Name::new("Lab Ground"),
        Mesh3d(meshes.add(Plane3d::default().mesh().size(world_size.x, world_size.y))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.22, 0.25, 0.22),
            perceptual_roughness: 1.0,
            ..default()
        })),
        Transform::from_xyz(center.x, 0.0, center.y),
    ));

    spawn_wall(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Central Wall",
        Vec2::new(11.5, 8.5),
        Vec2::new(1.2, 10.0),
        2.4,
        Color::srgb(0.38, 0.35, 0.32),
    );
    spawn_wall(
        &mut commands,
        &mut meshes,
        &mut materials,
        "South Wall",
        Vec2::new(16.5, 5.0),
        Vec2::new(7.0, 1.2),
        2.0,
        Color::srgb(0.34, 0.34, 0.38),
    );

    let scout_alpha = spawn_source(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Scout Alpha",
        Vec2::new(4.5, 4.5),
        Color::srgb(0.36, 0.92, 0.70),
        VisionSource::circle(FogLayerId(0), 4.8),
    );
    commands.entity(scout_alpha).insert(ScoutAlpha);

    let scout_beta = spawn_source(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Scout Beta",
        Vec2::new(20.5, 13.0),
        Color::srgb(0.34, 0.78, 0.98),
        VisionSource::circle(FogLayerId(0), 4.0),
    );
    commands.entity(scout_beta).insert(ScoutBeta);

    let sentry = spawn_source(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Sentry",
        Vec2::new(18.0, 6.5),
        Color::srgb(0.98, 0.66, 0.24),
        VisionSource::new(
            FogLayerId(1),
            FogRevealShape::arc(5.5, 1.2, Vec2::new(-1.0, 0.0)),
        ),
    );
    commands.entity(sentry).insert(Sentry);

    let main_receiver = commands
        .spawn((
            Name::new("Main Projection"),
            FogProjectionReceiver::new(FogLayerId(0), Vec2::ZERO, world_size),
        ))
        .id();
    let minimap_receiver = commands
        .spawn((
            Name::new("Minimap Projection"),
            FogProjectionReceiver {
                layer: FogLayerId(0),
                world_origin: Vec2::new(world_size.x - 7.0, 1.0),
                world_size: Vec2::new(6.0, 4.5),
                palette: saddle_world_fog_of_war::FogPalette {
                    hidden: LinearRgba::new(0.05, 0.08, 0.12, 0.82),
                    explored: LinearRgba::new(0.18, 0.24, 0.28, 0.48),
                    visible: LinearRgba::new(0.0, 0.0, 0.0, 0.0),
                },
                opacity: 1.0,
                edge_softness: 0.2,
                elevation: 0.08,
            },
        ))
        .id();

    commands.insert_resource(LabEntities {
        scout_alpha,
        scout_beta,
        sentry,
        main_receiver,
        minimap_receiver,
    });

    commands.spawn((
        Name::new("Lab Overlay"),
        LabOverlay,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(18.0),
            top: Val::Px(18.0),
            width: Val::Px(460.0),
            padding: UiRect::all(Val::Px(12.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.03, 0.04, 0.06, 0.82)),
        Text::default(),
        TextFont {
            font_size: 15.0,
            ..default()
        },
        TextColor(Color::WHITE),
    ));
}

fn spawn_wall(
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
        VisionOccluder::rect(saddle_world_fog_of_war::FogLayerMask::ALL, size * 0.5),
        Mesh3d(meshes.add(Cuboid::new(size.x, height, size.y))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: color,
            perceptual_roughness: 0.92,
            ..default()
        })),
        Transform::from_xyz(position.x, height * 0.5, position.y),
    ));
}

fn spawn_source(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    name: impl Into<String>,
    position: Vec2,
    color: Color,
    source: VisionSource,
) -> Entity {
    commands
        .spawn((
            Name::new(name.into()),
            source,
            Mesh3d(meshes.add(Sphere::new(0.22).mesh().uv(16, 9))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                ..default()
            })),
            Transform::from_xyz(position.x, 0.28, position.y),
        ))
        .id()
}

fn animate_scout_alpha(
    time: Res<Time>,
    control: Res<LabControl>,
    mut scout: Query<&mut Transform, With<ScoutAlpha>>,
) {
    if control.pause_motion {
        return;
    }
    let Ok(mut transform) = scout.single_mut() else {
        return;
    };
    let phase = time.elapsed_secs() * control.scout_alpha_speed;
    transform.translation.x = 4.5 + phase.sin() * 2.6;
    transform.translation.z = 4.5 + (phase * 1.2).cos() * 1.8;
}

fn animate_scout_beta(
    time: Res<Time>,
    control: Res<LabControl>,
    mut scout: Query<&mut Transform, (With<ScoutBeta>, Without<ScoutAlpha>)>,
) {
    if control.pause_motion {
        return;
    }
    let Ok(mut transform) = scout.single_mut() else {
        return;
    };
    let phase = time.elapsed_secs() * control.scout_beta_speed;
    transform.translation.x = 20.0 + phase.cos() * 2.4;
    transform.translation.z = 13.0 + phase.sin() * 2.0;
}

fn animate_sentry(
    time: Res<Time>,
    control: Res<LabControl>,
    mut sentry: Query<(&Transform, &mut VisionSource), With<Sentry>>,
) {
    if control.pause_motion {
        return;
    }
    let Ok((transform, mut source)) = sentry.single_mut() else {
        return;
    };
    let phase = time.elapsed_secs() * control.camera_orbit_speed * 3.3333;
    source.shape = FogRevealShape::arc(
        control.sentry_radius,
        control.sentry_spread,
        Vec2::new(phase.cos(), phase.sin()).normalize_or_zero(),
    );
    if transform.translation.x < 0.0 {
        source.shape = FogRevealShape::arc(
            control.sentry_radius,
            control.sentry_spread,
            Vec2::new(-1.0, 0.0),
        );
    }
}

fn orbit_camera(
    time: Res<Time>,
    control: Res<LabControl>,
    mut camera: Query<&mut Transform, With<LabCamera>>,
) {
    if control.pause_motion {
        return;
    }
    let Ok(mut transform) = camera.single_mut() else {
        return;
    };
    let center = Vec3::new(12.0, 0.0, 9.0);
    let angle = time.elapsed_secs() * control.camera_orbit_speed;
    transform.translation = Vec3::new(
        center.x + angle.cos() * 13.0,
        16.0 + (angle * 1.8).sin() * 1.2,
        center.z + angle.sin() * 9.5,
    );
    transform.look_at(center, Vec3::Y);
}

fn sync_demo_controls(
    control: Res<LabControl>,
    entities: Res<LabEntities>,
    mut sources: Query<&mut VisionSource>,
    mut receivers: Query<&mut FogProjectionReceiver>,
) {
    if !control.is_changed() {
        return;
    }

    if let Ok(mut source) = sources.get_mut(entities.scout_alpha) {
        source.shape = FogRevealShape::circle(control.scout_alpha_radius);
    }
    if let Ok(mut source) = sources.get_mut(entities.scout_beta) {
        source.shape = FogRevealShape::circle(control.scout_beta_radius);
    }
    if let Ok(mut source) = sources.get_mut(entities.sentry) {
        let facing = match source.shape {
            FogRevealShape::Arc { facing, .. } => facing,
            _ => Vec2::new(-1.0, 0.0),
        };
        source.shape = FogRevealShape::arc(control.sentry_radius, control.sentry_spread, facing);
    }

    if let Ok(mut receiver) = receivers.get_mut(entities.main_receiver) {
        receiver.edge_softness = control.main_edge_softness;
    }
}

fn sync_receivers(
    control: Res<LabControl>,
    entities: Res<LabEntities>,
    mut receivers: Query<&mut FogProjectionReceiver>,
) {
    if !control.is_changed() {
        return;
    }

    for entity in [entities.main_receiver, entities.minimap_receiver] {
        if let Ok(mut receiver) = receivers.get_mut(entity) {
            receiver.layer = control.selected_layer;
        }
    }
}

fn update_diagnostics(
    map: Res<FogOfWarMap>,
    control: Res<LabControl>,
    entities: Res<LabEntities>,
    scout_alpha: Query<&Transform, With<ScoutAlpha>>,
    scout_beta: Query<&Transform, With<ScoutBeta>>,
    sentry: Query<&Transform, With<Sentry>>,
    camera: Query<&Transform, With<LabCamera>>,
    receivers: Query<&FogProjectionReceiver>,
    mut diagnostics: ResMut<LabDiagnostics>,
) {
    diagnostics.selected_layer = control.selected_layer;
    diagnostics.layer_zero = map.layer_summary(FogLayerId(0)).unwrap_or(FogLayerSummary {
        visible_cells: 0,
        explored_cells: 0,
    });
    diagnostics.layer_one = map.layer_summary(FogLayerId(1)).unwrap_or(FogLayerSummary {
        visible_cells: 0,
        explored_cells: 0,
    });
    diagnostics.memory_sample = map
        .visibility_at_cell(FogLayerId(0), MEMORY_SAMPLE_CELL)
        .unwrap_or(FogVisibilityState::Hidden);
    diagnostics.occluded_sample = map
        .visibility_at_cell(FogLayerId(0), OCCLUDED_SAMPLE_CELL)
        .unwrap_or(FogVisibilityState::Hidden);
    diagnostics.main_receiver_layer = receivers
        .get(entities.main_receiver)
        .map(|receiver| receiver.layer)
        .unwrap_or(FogLayerId(0));
    diagnostics.camera_pos = camera
        .single()
        .map(|transform| transform.translation)
        .unwrap_or(Vec3::ZERO);
    diagnostics.scout_alpha_pos = scout_alpha
        .single()
        .map(|transform| transform.translation.xz())
        .unwrap_or(Vec2::ZERO);
    diagnostics.scout_beta_pos = scout_beta
        .single()
        .map(|transform| transform.translation.xz())
        .unwrap_or(Vec2::ZERO);
    diagnostics.sentry_pos = sentry
        .single()
        .map(|transform| transform.translation.xz())
        .unwrap_or(Vec2::ZERO);
}

fn update_overlay(
    diagnostics: Res<LabDiagnostics>,
    mut overlay: Query<&mut Text, With<LabOverlay>>,
) {
    let Ok(mut text) = overlay.single_mut() else {
        return;
    };

    text.0 = format!(
        "Fog Of War Lab\nSelected layer {}\nLayer 0 visible/explored: {}/{}\nLayer 1 visible/explored: {}/{}\nMemory sample: {:?}\nOccluded sample: {:?}\nScout alpha: {:.1?}\nScout beta: {:.1?}\nSentry: {:.1?}",
        diagnostics.selected_layer.0,
        diagnostics.layer_zero.visible_cells,
        diagnostics.layer_zero.explored_cells,
        diagnostics.layer_one.visible_cells,
        diagnostics.layer_one.explored_cells,
        diagnostics.memory_sample,
        diagnostics.occluded_sample,
        diagnostics.scout_alpha_pos,
        diagnostics.scout_beta_pos,
        diagnostics.sentry_pos,
    );
}

#[cfg(feature = "e2e")]
pub(crate) fn set_selected_layer(world: &mut World, layer: FogLayerId) {
    world.resource_mut::<LabControl>().selected_layer = layer;
}

#[cfg(feature = "e2e")]
pub(crate) fn set_pause_motion(world: &mut World, paused: bool) {
    world.resource_mut::<LabControl>().pause_motion = paused;
}

#[cfg(feature = "e2e")]
pub(crate) fn place_scout_alpha(world: &mut World, position: Vec2) {
    let entities = *world.resource::<LabEntities>();
    if let Ok(mut entity) = world.get_entity_mut(entities.scout_alpha) {
        if let Some(mut transform) = entity.get_mut::<Transform>() {
            transform.translation.x = position.x;
            transform.translation.z = position.y;
        }
    }
}

#[cfg(feature = "e2e")]
pub(crate) fn place_scout_beta(world: &mut World, position: Vec2) {
    let entities = *world.resource::<LabEntities>();
    if let Ok(mut entity) = world.get_entity_mut(entities.scout_beta) {
        if let Some(mut transform) = entity.get_mut::<Transform>() {
            transform.translation.x = position.x;
            transform.translation.z = position.y;
        }
    }
}

#[cfg(feature = "e2e")]
pub(crate) fn place_sentry(world: &mut World, position: Vec2, facing: Vec2) {
    let entities = *world.resource::<LabEntities>();
    if let Ok(mut entity) = world.get_entity_mut(entities.sentry) {
        if let Some(mut transform) = entity.get_mut::<Transform>() {
            transform.translation.x = position.x;
            transform.translation.z = position.y;
        }
        if let Some(mut source) = entity.get_mut::<VisionSource>() {
            source.shape = FogRevealShape::arc(5.5, 1.2, facing);
        }
    }
}
