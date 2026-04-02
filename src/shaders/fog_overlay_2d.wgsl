#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct FogRenderUniform {
    hidden_color: vec4<f32>,
    explored_color: vec4<f32>,
    visible_color: vec4<f32>,
    opacity: f32,
    edge_softness: f32,
    _padding: vec2<f32>,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: FogRenderUniform;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var fog_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var fog_sampler: sampler;

fn mix_fog_color(state_value: f32) -> vec4<f32> {
    let explored_gate = smoothstep(0.15, 0.55 + material.edge_softness * 0.25, state_value);
    let visible_gate = smoothstep(0.55 - material.edge_softness * 0.25, 0.95, state_value);

    let hidden_to_explored = mix(material.hidden_color, material.explored_color, explored_gate);
    let final_color = mix(hidden_to_explored, material.visible_color, visible_gate);
    return vec4<f32>(final_color.rgb, final_color.a * material.opacity);
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let sampled = textureSample(fog_texture, fog_sampler, mesh.uv).r;
    return mix_fog_color(sampled);
}
