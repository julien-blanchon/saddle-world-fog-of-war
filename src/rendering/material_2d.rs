use bevy::{
    asset::Asset,
    prelude::*,
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderType},
    shader::ShaderRef,
    sprite_render::{AlphaMode2d, Material2d},
};

use crate::components::FogPalette;

use super::FOG_OVERLAY_2D_SHADER_HANDLE;

#[derive(Clone, Copy, Debug, ShaderType)]
pub(crate) struct FogRenderUniform {
    pub hidden_color: Vec4,
    pub explored_color: Vec4,
    pub visible_color: Vec4,
    pub opacity: f32,
    pub edge_softness: f32,
    pub _padding: Vec2,
}

impl FogRenderUniform {
    pub fn new(palette: FogPalette, opacity: f32, edge_softness: f32) -> Self {
        Self {
            hidden_color: Vec4::from_array(palette.hidden.to_f32_array()),
            explored_color: Vec4::from_array(palette.explored.to_f32_array()),
            visible_color: Vec4::from_array(palette.visible.to_f32_array()),
            opacity,
            edge_softness,
            _padding: Vec2::ZERO,
        }
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub(crate) struct FogOverlayMaterial2d {
    #[uniform(0)]
    pub settings: FogRenderUniform,
    #[texture(1)]
    #[sampler(2)]
    pub state_texture: Handle<Image>,
}

impl Material2d for FogOverlayMaterial2d {
    fn fragment_shader() -> ShaderRef {
        FOG_OVERLAY_2D_SHADER_HANDLE.into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}
