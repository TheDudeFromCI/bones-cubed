use bevy::render::alpha::AlphaMode;
use serde::{Deserialize, Serialize};

/// The file layout of a tileset, which is used when loading a tileset from a
/// file.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "Tileset")]
pub struct TilesetFileLayout {
    /// The name of the tileset.
    pub name: String,

    /// The size of the tileset in pixels. All tiles must be the same size.
    /// Size is a square and must be a power of two (e.g. 16, 32, 64, 128,
    /// etc.). The maximum size is 1024.
    pub size: u32,

    /// The alpha mode of the tileset, which determines how the tileset is
    /// rendered. This value may be overriden by the material settings of the
    /// tileset material.
    #[serde(default)]
    pub alpha_mode: AlphaModeLayout,

    /// The name of the material to use for the tileset. If not specified, the
    /// default material is used.
    #[serde(default)]
    pub material: Option<String>,

    /// The list of tiles in the tileset. The maximum number of tiles is 65536.
    pub tiles: Vec<TileLayout>,
}

/// The alpha mode of a tileset, which determines how the tileset is rendered.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AlphaModeLayout {
    /// The tileset is fully opaque.
    #[default]
    Opaque,

    /// The tileset uses alpha masking, where pixels with an alpha value below
    /// 50% are fully transparent and pixels with an alpha value of 50% or above
    /// are fully opaque.
    Mask,

    /// The tileset uses full alpha blending.
    Blend,
}

impl From<AlphaModeLayout> for AlphaMode {
    fn from(layout: AlphaModeLayout) -> Self {
        match layout {
            AlphaModeLayout::Opaque => AlphaMode::Opaque,
            AlphaModeLayout::Mask => AlphaMode::Mask(0.5),
            AlphaModeLayout::Blend => AlphaMode::Blend,
        }
    }
}

/// The file layout for a tile definition.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TileLayout {
    /// The name of the tile.
    pub name: String,

    /// The texture of the tile, which is a path to the tile's texture relative
    /// to the tileset file.
    ///
    /// Texture must be the same size as the tileset.
    pub texture: String,
}
