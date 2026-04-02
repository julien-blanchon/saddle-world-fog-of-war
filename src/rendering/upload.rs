use std::time::Instant;

use bevy::{
    asset::RenderAssetUsages,
    image::ImageSampler,
    pbr::MeshMaterial3d,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    sprite_render::MeshMaterial2d,
};

use crate::{
    components::{FogOverlay2d, FogProjectionReceiver},
    resources::{FogOfWarMap, FogOfWarRenderAssets, FogOfWarStats},
};

use super::{FogOverlayMaterial2d, FogProjectionMaterial3d, material_2d::FogRenderUniform};

#[derive(Resource, Default)]
pub(crate) struct FogRenderGeometry {
    pub quad_2d: Handle<Mesh>,
    pub plane_3d: Handle<Mesh>,
}

pub(crate) fn render_upload_noop() {}

pub(crate) fn ensure_render_geometry(
    mut geometry: ResMut<FogRenderGeometry>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if geometry.quad_2d == Handle::default() {
        geometry.quad_2d = meshes.add(Rectangle::new(1.0, 1.0));
    }
    if geometry.plane_3d == Handle::default() {
        geometry.plane_3d = meshes.add(Plane3d::default().mesh().size(1.0, 1.0));
    }
}

pub(crate) fn upload_images(
    map: Res<FogOfWarMap>,
    mut render_assets: ResMut<FogOfWarRenderAssets>,
    mut images: ResMut<Assets<Image>>,
    mut stats: ResMut<FogOfWarStats>,
) {
    let start = Instant::now();
    let grid = map.grid();

    for (layer, layer_state) in map.layers() {
        let handle = render_assets
            .images
            .entry(layer)
            .or_insert_with(|| images.add(make_state_image(grid.dimensions)))
            .clone();
        let image = images
            .get_mut(&handle)
            .expect("fog render image should exist");
        if !matches_size(image, grid.dimensions) {
            *image = make_state_image(grid.dimensions);
        }
        image.data = Some(
            layer_state
                .states
                .iter()
                .map(|state| FogOfWarMap::state_byte(*state))
                .collect(),
        );
    }

    stats.last_upload_micros = start.elapsed().as_micros() as u64;
}

pub(crate) fn sync_overlay_materials_2d(
    mut commands: Commands,
    geometry: Res<FogRenderGeometry>,
    render_assets: Res<FogOfWarRenderAssets>,
    mut materials: ResMut<Assets<FogOverlayMaterial2d>>,
    overlays: Query<(
        Entity,
        &FogOverlay2d,
        Option<&MeshMaterial2d<FogOverlayMaterial2d>>,
    )>,
) {
    for (entity, overlay, material_handle) in &overlays {
        let Some(state_texture) = render_assets.image(overlay.layer) else {
            continue;
        };
        let settings =
            FogRenderUniform::new(overlay.palette, overlay.opacity, overlay.edge_softness);

        let handle = if let Some(existing) = material_handle {
            if let Some(material) = materials.get_mut(&existing.0) {
                material.settings = settings;
                material.state_texture = state_texture.clone();
            }
            existing.0.clone()
        } else {
            materials.add(FogOverlayMaterial2d {
                settings,
                state_texture: state_texture.clone(),
            })
        };

        let center = overlay.world_origin + overlay.world_size * 0.5;
        commands.entity(entity).insert((
            Mesh2d(geometry.quad_2d.clone()),
            MeshMaterial2d(handle),
            Transform::from_translation(center.extend(overlay.z))
                .with_scale(overlay.world_size.extend(1.0)),
        ));
    }
}

pub(crate) fn sync_projection_materials_3d(
    mut commands: Commands,
    geometry: Res<FogRenderGeometry>,
    render_assets: Res<FogOfWarRenderAssets>,
    mut materials: ResMut<Assets<FogProjectionMaterial3d>>,
    receivers: Query<(
        Entity,
        &FogProjectionReceiver,
        Option<&MeshMaterial3d<FogProjectionMaterial3d>>,
    )>,
) {
    for (entity, receiver, material_handle) in &receivers {
        let Some(state_texture) = render_assets.image(receiver.layer) else {
            continue;
        };
        let settings =
            FogRenderUniform::new(receiver.palette, receiver.opacity, receiver.edge_softness);

        let handle = if let Some(existing) = material_handle {
            if let Some(material) = materials.get_mut(&existing.0) {
                material.settings = settings;
                material.state_texture = state_texture.clone();
            }
            existing.0.clone()
        } else {
            materials.add(FogProjectionMaterial3d {
                settings,
                state_texture: state_texture.clone(),
            })
        };

        let center = receiver.world_origin + receiver.world_size * 0.5;
        commands.entity(entity).insert((
            Mesh3d(geometry.plane_3d.clone()),
            MeshMaterial3d(handle),
            Transform::from_xyz(center.x, receiver.elevation, center.y).with_scale(Vec3::new(
                receiver.world_size.x,
                1.0,
                receiver.world_size.y,
            )),
        ));
    }
}

fn make_state_image(dimensions: UVec2) -> Image {
    let mut image = Image::new_fill(
        Extent3d {
            width: dimensions.x,
            height: dimensions.y,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0],
        TextureFormat::R8Unorm,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = ImageSampler::linear();
    image
}

fn matches_size(image: &Image, dimensions: UVec2) -> bool {
    image.texture_descriptor.size.width == dimensions.x
        && image.texture_descriptor.size.height == dimensions.y
}
