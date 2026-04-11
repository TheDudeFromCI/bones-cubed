use bevy::color::Color;
use bevy::math::{Vec2, Vec3};
use bevy::transform::components::Transform;

use crate::block::culling::Culling;
use crate::block::shape::CubeProperties;
use crate::utils::mesh::{TerrainMesh, TerrainQuad, TerrainVertex};

pub(crate) fn build(
    properties: &CubeProperties,
    culling: Culling,
    transform: Transform,
    mesh: &mut TerrainMesh,
) {
    if !culling.contains(Culling::POS_Y) {
        mesh.add_quad(
            TerrainQuad(
                TerrainVertex {
                    position: Vec3::new(0.0, 1.0, 0.0),
                    uv: properties.top_face.rotation.matrix * Vec2::new(0.0, 0.0),
                    normal: Vec3::Y,
                    color: Color::WHITE,
                    layer: properties.top_face.texture_layer as u32,
                },
                TerrainVertex {
                    position: Vec3::new(0.0, 1.0, 1.0),
                    uv: properties.top_face.rotation.matrix * Vec2::new(0.0, 1.0),
                    normal: Vec3::Y,
                    color: Color::WHITE,
                    layer: properties.top_face.texture_layer as u32,
                },
                TerrainVertex {
                    position: Vec3::new(1.0, 1.0, 1.0),
                    uv: properties.top_face.rotation.matrix * Vec2::new(1.0, 1.0),
                    normal: Vec3::Y,
                    color: Color::WHITE,
                    layer: properties.top_face.texture_layer as u32,
                },
                TerrainVertex {
                    position: Vec3::new(1.0, 1.0, 0.0),
                    uv: properties.top_face.rotation.matrix * Vec2::new(1.0, 0.0),
                    normal: Vec3::Y,
                    color: Color::WHITE,
                    layer: properties.top_face.texture_layer as u32,
                },
            ) * transform,
        );
    }

    if !culling.contains(Culling::NEG_Y) {
        mesh.add_quad(
            TerrainQuad(
                TerrainVertex {
                    position: Vec3::new(1.0, 0.0, 1.0),
                    uv: properties.bottom_face.rotation.matrix * Vec2::new(1.0, 1.0),
                    normal: Vec3::NEG_Y,
                    color: Color::WHITE,
                    layer: properties.bottom_face.texture_layer as u32,
                },
                TerrainVertex {
                    position: Vec3::new(0.0, 0.0, 1.0),
                    uv: properties.bottom_face.rotation.matrix * Vec2::new(0.0, 1.0),
                    normal: Vec3::NEG_Y,
                    color: Color::WHITE,
                    layer: properties.bottom_face.texture_layer as u32,
                },
                TerrainVertex {
                    position: Vec3::new(0.0, 0.0, 0.0),
                    uv: properties.bottom_face.rotation.matrix * Vec2::new(0.0, 0.0),
                    normal: Vec3::NEG_Y,
                    color: Color::WHITE,
                    layer: properties.bottom_face.texture_layer as u32,
                },
                TerrainVertex {
                    position: Vec3::new(1.0, 0.0, 0.0),
                    uv: properties.bottom_face.rotation.matrix * Vec2::new(1.0, 0.0),
                    normal: Vec3::NEG_Y,
                    color: Color::WHITE,
                    layer: properties.bottom_face.texture_layer as u32,
                },
            ) * transform,
        );
    }

    if !culling.contains(Culling::POS_Z) {
        mesh.add_quad(
            TerrainQuad(
                TerrainVertex {
                    position: Vec3::new(0.0, 0.0, 1.0),
                    uv: properties.north_face.rotation.matrix * Vec2::new(0.0, 1.0),
                    normal: Vec3::NEG_Z,
                    color: Color::WHITE,
                    layer: properties.north_face.texture_layer as u32,
                },
                TerrainVertex {
                    position: Vec3::new(1.0, 0.0, 1.0),
                    uv: properties.north_face.rotation.matrix * Vec2::new(1.0, 1.0),
                    normal: Vec3::NEG_Z,
                    color: Color::WHITE,
                    layer: properties.north_face.texture_layer as u32,
                },
                TerrainVertex {
                    position: Vec3::new(1.0, 1.0, 1.0),
                    uv: properties.north_face.rotation.matrix * Vec2::new(1.0, 0.0),
                    normal: Vec3::NEG_Z,
                    color: Color::WHITE,
                    layer: properties.north_face.texture_layer as u32,
                },
                TerrainVertex {
                    position: Vec3::new(0.0, 1.0, 1.0),
                    uv: properties.north_face.rotation.matrix * Vec2::new(0.0, 0.0),
                    normal: Vec3::NEG_Z,
                    color: Color::WHITE,
                    layer: properties.north_face.texture_layer as u32,
                },
            ) * transform,
        )
    }

    if !culling.contains(Culling::NEG_Z) {
        mesh.add_quad(
            TerrainQuad(
                TerrainVertex {
                    position: Vec3::new(1.0, 1.0, 0.0),
                    uv: properties.south_face.rotation.matrix * Vec2::new(0.0, 0.0),
                    normal: Vec3::NEG_Z,
                    color: Color::WHITE,
                    layer: properties.south_face.texture_layer as u32,
                },
                TerrainVertex {
                    position: Vec3::new(1.0, 0.0, 0.0),
                    uv: properties.south_face.rotation.matrix * Vec2::new(0.0, 1.0),
                    normal: Vec3::NEG_Z,
                    color: Color::WHITE,
                    layer: properties.south_face.texture_layer as u32,
                },
                TerrainVertex {
                    position: Vec3::new(0.0, 0.0, 0.0),
                    uv: properties.south_face.rotation.matrix * Vec2::new(1.0, 1.0),
                    normal: Vec3::NEG_Z,
                    color: Color::WHITE,
                    layer: properties.south_face.texture_layer as u32,
                },
                TerrainVertex {
                    position: Vec3::new(0.0, 1.0, 0.0),
                    uv: properties.south_face.rotation.matrix * Vec2::new(1.0, 0.0),
                    normal: Vec3::NEG_Z,
                    color: Color::WHITE,
                    layer: properties.south_face.texture_layer as u32,
                },
            ) * transform,
        );
    }

    if !culling.contains(Culling::POS_X) {
        mesh.add_quad(
            TerrainQuad(
                TerrainVertex {
                    position: Vec3::new(1.0, 0.0, 1.0),
                    uv: properties.east_face.rotation.matrix * Vec2::new(0.0, 1.0),
                    normal: Vec3::X,
                    color: Color::WHITE,
                    layer: properties.east_face.texture_layer as u32,
                },
                TerrainVertex {
                    position: Vec3::new(1.0, 0.0, 0.0),
                    uv: properties.east_face.rotation.matrix * Vec2::new(1.0, 1.0),
                    normal: Vec3::X,
                    color: Color::WHITE,
                    layer: properties.east_face.texture_layer as u32,
                },
                TerrainVertex {
                    position: Vec3::new(1.0, 1.0, 0.0),
                    uv: properties.east_face.rotation.matrix * Vec2::new(1.0, 0.0),
                    normal: Vec3::X,
                    color: Color::WHITE,
                    layer: properties.east_face.texture_layer as u32,
                },
                TerrainVertex {
                    position: Vec3::new(1.0, 1.0, 1.0),
                    uv: properties.east_face.rotation.matrix * Vec2::new(0.0, 0.0),
                    normal: Vec3::X,
                    color: Color::WHITE,
                    layer: properties.east_face.texture_layer as u32,
                },
            ) * transform,
        );
    }

    if !culling.contains(Culling::NEG_X) {
        mesh.add_quad(
            TerrainQuad(
                TerrainVertex {
                    position: Vec3::new(0.0, 0.0, 0.0),
                    uv: properties.west_face.rotation.matrix * Vec2::new(0.0, 1.0),
                    normal: Vec3::NEG_X,
                    color: Color::WHITE,
                    layer: properties.west_face.texture_layer as u32,
                },
                TerrainVertex {
                    position: Vec3::new(0.0, 0.0, 1.0),
                    uv: properties.west_face.rotation.matrix * Vec2::new(1.0, 1.0),
                    normal: Vec3::NEG_X,
                    color: Color::WHITE,
                    layer: properties.west_face.texture_layer as u32,
                },
                TerrainVertex {
                    position: Vec3::new(0.0, 1.0, 1.0),
                    uv: properties.west_face.rotation.matrix * Vec2::new(1.0, 0.0),
                    normal: Vec3::NEG_X,
                    color: Color::WHITE,
                    layer: properties.west_face.texture_layer as u32,
                },
                TerrainVertex {
                    position: Vec3::new(0.0, 1.0, 0.0),
                    uv: properties.west_face.rotation.matrix * Vec2::new(0.0, 0.0),
                    normal: Vec3::NEG_X,
                    color: Color::WHITE,
                    layer: properties.west_face.texture_layer as u32,
                },
            ) * transform,
        );
    }
}
