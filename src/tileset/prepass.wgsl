#import bevy_pbr::mesh_functions::{
    mesh_position_local_to_clip,
    get_world_from_local,
    mesh_normal_local_to_world
}

struct VertexInput {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) layer: u32,
    @location(4) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) normal: vec3<f32>,
}

struct FragmentOutput {
    @location(0) color: vec4<f32>,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var texture: texture_2d_array<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var texture_sampler: sampler;

@vertex
fn vertex(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.position = mesh_position_local_to_clip(
        get_world_from_local(input.instance_index),
        vec4<f32>(input.position, 1.0),
    );
    output.normal = mesh_normal_local_to_world(input.normal, input.instance_index);
    return output;
}

@fragment
fn fragment(input: VertexOutput) -> FragmentOutput {
    var output: FragmentOutput;
    output.color =  vec4<f32>(1.0);
    return output;
}