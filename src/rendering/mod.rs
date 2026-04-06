mod material_2d;
mod material_3d;
mod upload;

use bevy::{
    asset::{load_internal_asset, uuid_handle},
    ecs::schedule::ScheduleLabel,
    prelude::*,
    render::RenderApp,
    shader::Shader,
};

use crate::{FogOfWarSystems, resources::FogOfWarRenderAssets};

pub(crate) use material_2d::FogOverlayMaterial2d;
pub(crate) use material_3d::FogProjectionMaterial3d;

pub(crate) const FOG_OVERLAY_2D_SHADER_HANDLE: Handle<Shader> =
    uuid_handle!("85c65448-feb7-4ae7-8706-9d7cb3fbb8d3");
pub(crate) const FOG_OVERLAY_3D_SHADER_HANDLE: Handle<Shader> =
    uuid_handle!("c59e0fac-85d5-43ec-b373-8ae02058c5ef");

pub(crate) fn plugin(app: &mut App, update_schedule: impl ScheduleLabel) {
    app.init_resource::<FogOfWarRenderAssets>();

    if app.get_sub_app(RenderApp).is_none() {
        app.add_systems(
            update_schedule,
            upload::render_upload_noop.in_set(FogOfWarSystems::UploadRenderData),
        );
        return;
    }

    load_internal_asset!(
        app,
        FOG_OVERLAY_2D_SHADER_HANDLE,
        "../shaders/fog_overlay_2d.wgsl",
        Shader::from_wgsl
    );
    load_internal_asset!(
        app,
        FOG_OVERLAY_3D_SHADER_HANDLE,
        "../shaders/fog_overlay_3d.wgsl",
        Shader::from_wgsl
    );

    app.init_resource::<upload::FogRenderGeometry>()
        .add_plugins((
            bevy::sprite_render::Material2dPlugin::<FogOverlayMaterial2d>::default(),
            bevy::pbr::MaterialPlugin::<FogProjectionMaterial3d>::default(),
        ))
        .add_systems(
            update_schedule,
            (
                upload::ensure_render_geometry,
                upload::upload_images,
                upload::sync_overlay_materials_2d,
                upload::sync_projection_materials_3d,
            )
                .chain()
                .in_set(FogOfWarSystems::UploadRenderData),
        );
}
