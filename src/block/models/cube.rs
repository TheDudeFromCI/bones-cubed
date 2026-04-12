use bevy::asset::{Handle, LoadContext};
use bevy::color::Color;
use bevy::math::{Vec2, Vec3};
use bevy::transform::components::Transform;

use crate::block::asset::BlockLoaderError;
use crate::block::culling::Culling;
use crate::block::filelayout::CubePropertiesLayout;
use crate::block::models::{BlockModel, FaceProperties, LoadedBlockTileset};
use crate::tileset::material::Tileset;
use crate::utils::mesh::{TerrainMesh, TerrainQuad, TerrainVertex};

#[derive(Debug, Clone)]
pub struct CubeProperties {
    /// The tileset to use for this cube.
    pub tileset: Handle<Tileset>,

    /// The properties of the top face of the cube.
    pub top_face: FaceProperties,

    /// The properties of the bottom face of the cube.
    pub bottom_face: FaceProperties,

    /// The properties of the north face of the cube.
    pub north_face: FaceProperties,

    /// The properties of the south face of the cube.
    pub south_face: FaceProperties,

    /// The properties of the east face of the cube.
    pub east_face: FaceProperties,

    /// The properties of the west face of the cube.
    pub west_face: FaceProperties,
}

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

pub(crate) async fn load(
    properties: CubePropertiesLayout,
    ctx: &mut LoadContext<'_>,
) -> Result<BlockModel, BlockLoaderError> {
    let tileset = LoadedBlockTileset::load(ctx, &properties.tileset).await?;

    Ok(BlockModel::Cube(CubeProperties {
        tileset: tileset.handle().clone(),

        top_face: FaceProperties {
            texture_layer: tileset.tile_index(&properties.top.tile)?,
            rotation: properties.top.tex_rotation(),
        },
        bottom_face: FaceProperties {
            texture_layer: tileset.tile_index(&properties.bottom.tile)?,
            rotation: properties.bottom.tex_rotation(),
        },
        north_face: FaceProperties {
            texture_layer: tileset.tile_index(&properties.north.tile)?,
            rotation: properties.north.tex_rotation(),
        },
        south_face: FaceProperties {
            texture_layer: tileset.tile_index(&properties.south.tile)?,
            rotation: properties.south.tex_rotation(),
        },
        east_face: FaceProperties {
            texture_layer: tileset.tile_index(&properties.east.tile)?,
            rotation: properties.east.tex_rotation(),
        },
        west_face: FaceProperties {
            texture_layer: tileset.tile_index(&properties.west.tile)?,
            rotation: properties.west.tex_rotation(),
        },
    }))
}
