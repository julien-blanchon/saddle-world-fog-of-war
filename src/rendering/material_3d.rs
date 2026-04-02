use bevy::{
    asset::Asset, pbr::Material, prelude::*, reflect::TypePath, render::alpha::AlphaMode,
    render::render_resource::AsBindGroup, shader::ShaderRef,
};

use super::{FOG_OVERLAY_3D_SHADER_HANDLE, material_2d::FogRenderUniform};

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub(crate) struct FogProjectionMaterial3d {
    #[uniform(0)]
    pub settings: FogRenderUniform,
    #[texture(1)]
    #[sampler(2)]
    pub state_texture: Handle<Image>,
}

impl Material for FogProjectionMaterial3d {
    fn fragment_shader() -> ShaderRef {
        FOG_OVERLAY_3D_SHADER_HANDLE.into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }

    fn depth_bias(&self) -> f32 {
        16.0
    }
}
