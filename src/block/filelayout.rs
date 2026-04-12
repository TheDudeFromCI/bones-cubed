use serde::Deserialize;

use crate::block::models::face::TextureRotation;

#[derive(Debug, Default, Deserialize)]
#[serde(rename = "Block")]
pub struct BlockFileLayout {
    /// The name of the block type.
    pub name: String,

    /// The model of the block.
    pub model: BlockModelLayout,
}

#[derive(Debug, Default, Deserialize)]
pub enum BlockModelLayout {
    /// An empty block with no geometry.
    #[default]
    Empty,

    /// A standard cube block.
    Cube(CubePropertiesLayout),

    /// A custom block with user-defined geometry.
    Custom,
}

#[derive(Debug, Default, Deserialize)]
pub struct CubePropertiesLayout {
    /// The path to the tileset to use for this block type, relative to the
    /// block asset file.
    pub tileset: String,

    /// The layout of the top face of the cube.
    pub top: BlockTextureLayout,

    /// The layout of the bottom face of the cube.
    pub bottom: BlockTextureLayout,

    /// The layout of the north face of the cube.
    pub north: BlockTextureLayout,

    /// The layout of the south face of the cube.
    pub south: BlockTextureLayout,

    /// The layout of the east face of the cube.
    pub east: BlockTextureLayout,

    /// The layout of the west face of the cube.
    pub west: BlockTextureLayout,
}

/// The layout of a block face's texture, which determines how the texture is
/// applied to the face.
#[derive(Debug, Default, Deserialize)]
pub struct BlockTextureLayout {
    /// The name of the tile to use for this block face.
    pub tile: String,

    /// The rotation to apply to the texture on this face.
    #[serde(default)]
    pub rotation: TextureRotationLayout,

    /// Whether to mirror the texture on this face across the X axis.
    #[serde(default)]
    pub mirror_x: bool,

    /// Whether to mirror the texture on this face across the Y axis.
    #[serde(default)]
    pub mirror_y: bool,
}

impl BlockTextureLayout {
    /// Computes the final texture rotation for this block face, taking into
    /// account the specified rotation and mirroring.
    pub fn tex_rotation(&self) -> TextureRotation {
        let mut rotation = TextureRotation::identity();

        if self.mirror_x {
            rotation.mirror_x();
        }

        if self.mirror_y {
            rotation.mirror_y();
        }

        match self.rotation {
            TextureRotationLayout::Rotate0 => {}
            TextureRotationLayout::Rotate90 => rotation.rotate(90.0),
            TextureRotationLayout::Rotate180 => rotation.rotate(180.0),
            TextureRotationLayout::Rotate270 => rotation.rotate(270.0),
        }
        rotation
    }
}

/// The rotation to apply to a block face's texture.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum TextureRotationLayout {
    /// No rotation.
    #[default]
    #[serde(rename = "0")]
    Rotate0,

    /// Rotate the texture 90 degrees clockwise.
    #[serde(rename = "90")]
    Rotate90,

    /// Rotate the texture 180 degrees clockwise.
    #[serde(rename = "180")]
    Rotate180,

    /// Rotate the texture 270 degrees clockwise.
    #[serde(rename = "270")]
    Rotate270,
}
