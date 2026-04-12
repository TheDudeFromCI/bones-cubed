use bevy::asset::LoadContext;
use bevy::prelude::*;

use crate::block::asset::BlockLoaderError;
use crate::block::culling::Culling;
use crate::block::models::cube::CubeProperties;
use crate::tileset::asset::TilesetLoaderSettings;
use crate::tileset::material::Tileset;
use crate::utils::asset::ContextRelativePathEtx;
use crate::utils::mesh::TerrainMesh;

pub mod cube;

/// Defines the shape of a block, which can be used for rendering and mesh
/// generation.
#[derive(Debug, Default, Clone)]
pub enum BlockModel {
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

impl BlockModel {
    /// Appends the geometry of this block shape to the given mesh, applying the
    /// given transform and culling.
    pub fn append_model(&self, culling: Culling, transform: Transform, mesh: &mut TerrainMesh) {
        match self {
            BlockModel::Empty => {}
            BlockModel::Cube(properties) => cube::build(properties, culling, transform, mesh),
            BlockModel::Custom => {
                todo!("Custom block shape rendering is not yet implemented");
            }
        }
    }

    /// Returns true if this block shape has no geometry and should not be
    /// rendered.
    pub fn is_empty(&self) -> bool {
        matches!(self, BlockModel::Empty)
    }

    /// Gets the tileset used by this block shape, if any.
    pub fn tileset(&self) -> Option<&Handle<Tileset>> {
        match self {
            BlockModel::Empty | BlockModel::Custom => None,
            BlockModel::Cube(properties) => Some(&properties.tileset),
        }
    }
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

pub(super) struct LoadedBlockTileset {
    tileset: Tileset,
    handle: Handle<Tileset>,
}

impl LoadedBlockTileset {
    /// Loads a tileset and allows for convenient access to its tile names and
    /// asset handle. This is used by the block asset loader to load the
    /// tileset for block models.
    pub(super) async fn load(
        ctx: &mut LoadContext<'_>,
        relative_path: &str,
    ) -> Result<Self, BlockLoaderError> {
        let tileset_path = ctx.get_relative_path(relative_path)?;

        // Because immediate loading bypasses the asset_server, we have to load
        // the tileset manually here to get access to its tile names. Sadly, this means
        // we have to load the tileset twice: once immediately to get the tile names,
        // and once through the asset server to get the actual asset handle.
        let tileset: Tileset = ctx
            .loader()
            .immediate()
            .with_settings(|settings: &mut TilesetLoaderSettings| {
                // Skip all the image processing to speed up loading
                settings.names_only = true;
            })
            .load(&tileset_path)
            .await?
            .take();

        // Thankfully, this loader doesn't skip the asset_server, so already loaded
        // tilesets won't need to be loaded again and again for every block.
        let handle = ctx.load(&tileset_path);

        Ok(Self { tileset, handle })
    }

    /// Gets the index of the tile with the given name in this tileset, or
    /// returns an error if the tile is not found.
    pub fn tile_index(&self, tile_name: &str) -> Result<u16, BlockLoaderError> {
        self.tileset
            .tile_index(tile_name)
            .ok_or_else(|| BlockLoaderError::UnknownTile {
                tile: tile_name.to_string(),
                tileset: self.tileset.name().to_string(),
            })
    }

    /// Gets the handle to the tileset asset.
    pub fn handle(&self) -> &Handle<Tileset> {
        &self.handle
    }
}
