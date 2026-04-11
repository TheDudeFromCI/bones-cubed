use bevy::math::{Mat2, Vec2};
use bevy::transform::components::Transform;

use crate::block::culling::Culling;
use crate::block::models;
use crate::utils::mesh::TerrainMesh;

/// Defines the shape of a block, which can be used for rendering and mesh
/// generation.
#[derive(Debug, Default, Clone)]
pub enum BlockShape {
    /// An empty block with no geometry.
    #[default]
    Empty,

    /// A standard cube block.
    Cube(CubeProperties),

    /// A custom block with user-defined geometry.
    ///
    /// For occlusion culling purposes, custom blocks are treated as empty, not
    /// culling any faces of surrounding geometry.
    Custom,
}

impl BlockShape {
    pub fn append_model(&self, culling: Culling, transform: Transform, mesh: &mut TerrainMesh) {
        match self {
            BlockShape::Empty => {}
            BlockShape::Cube(properties) => {
                models::cube::build(properties, culling, transform, mesh)
            }
            BlockShape::Custom => {
                todo!("Custom block shape rendering is not yet implemented");
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct CubeProperties {
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

/// The properties of a block face, which determine how the face is rendered.
#[derive(Debug, Clone, Copy)]
pub struct FaceProperties {
    /// The index of the texture layer to use for this face.
    pub texture_layer: u16,

    /// The rotation to apply to the texture on this face.
    pub rotation: TextureRotation,
}

/// The rotation and mirroring to apply to a block face's texture.
#[derive(Debug, Clone, Copy)]
pub struct TextureRotation {
    /// The uv transformation matrix to apply to the texture coordinates of this
    /// face.
    pub matrix: Mat2,
}

impl TextureRotation {
    /// Creates a new `TextureRotation` with the identity transformation.
    pub fn identity() -> Self {
        Self {
            matrix: Mat2::IDENTITY,
        }
    }

    /// Applies a horizontal mirroring to the texture.
    pub fn mirror_x(&mut self) {
        self.matrix *= Mat2::from_cols(Vec2::new(-1.0, 0.0), Vec2::new(0.0, 1.0));
    }

    /// Applies a vertical mirroring to the texture.
    pub fn mirror_y(&mut self) {
        self.matrix *= Mat2::from_cols(Vec2::new(1.0, 0.0), Vec2::new(0.0, -1.0));
    }

    /// Rotates the texture by the given angle in degrees.
    pub fn rotate(&mut self, angle: f32) {
        let rad = angle.to_radians();
        self.matrix *= Mat2::from_angle(rad);
    }
}

impl Default for TextureRotation {
    fn default() -> Self {
        Self::identity()
    }
}
