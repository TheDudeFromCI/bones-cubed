use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext, LoadDirectError};
use bevy::prelude::*;
use ron::extensions::Extensions;

use crate::block::filelayout::{BlockFileLayout, BlockShapeLayout};
use crate::block::shape::{BlockShape, CubeProperties, FaceProperties};
use crate::tileset::asset::TilesetLoaderSettings;
use crate::tileset::material::Tileset;
use crate::utils::asset::{ContextRelativePathEtx, RelativePathError};

/// An asset that defines the properties of a block type.
#[derive(Debug, Clone, Asset, TypePath)]
pub struct Block {
    /// The name of this block type.
    name: Box<str>,

    /// The shape of this block type, which determines how it is rendered and
    /// culled.
    shape: BlockShape,

    /// The tileset to use for this block type.
    tileset: Handle<Tileset>,
}

impl Block {
    /// Creates a new Block asset with the given name, shape, and tileset.
    pub fn new(name: impl Into<Box<str>>, shape: BlockShape, tileset: Handle<Tileset>) -> Self {
        Self {
            name: name.into(),
            shape,
            tileset,
        }
    }

    /// Gets the display name of this block asset.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Gets the shape of this block asset.
    pub fn shape(&self) -> &BlockShape {
        &self.shape
    }

    /// Gets the tileset used by this block asset.
    pub fn tileset(&self) -> &Handle<Tileset> {
        &self.tileset
    }
}

/// A loader for Block asset files.
#[derive(Debug, Default, TypePath)]
pub struct BlockAssetLoader;
impl AssetLoader for BlockAssetLoader {
    type Asset = Block;
    type Settings = ();
    type Error = BlockLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _: &Self::Settings,
        ctx: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = vec![];
        reader.read_to_end(&mut bytes).await?;

        let layout: BlockFileLayout = ron::Options::default()
            .with_default_extension(Extensions::UNWRAP_NEWTYPES)
            .with_default_extension(Extensions::IMPLICIT_SOME)
            .with_default_extension(Extensions::UNWRAP_VARIANT_NEWTYPES)
            .from_bytes(&bytes)?;

        let tileset_path = ctx.get_relative_path(&layout.tileset)?;

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
        let tileset_handle: Handle<Tileset> = ctx.load(&tileset_path);

        let tile = |name: &str| -> Result<u16, Self::Error> {
            tileset
                .tile_index(name)
                .ok_or_else(|| BlockLoaderError::UnknownTile {
                    tile: name.to_string(),
                    tileset: tileset.name().to_string(),
                })
        };

        let shape = match layout.shape {
            BlockShapeLayout::Empty => BlockShape::Empty,
            BlockShapeLayout::Cube {
                top,
                bottom,
                north,
                south,
                east,
                west,
            } => BlockShape::Cube(CubeProperties {
                top_face: FaceProperties {
                    texture_layer: tile(&top.tile)?,
                    rotation: top.tex_rotation(),
                },
                bottom_face: FaceProperties {
                    texture_layer: tile(&bottom.tile)?,
                    rotation: bottom.tex_rotation(),
                },
                north_face: FaceProperties {
                    texture_layer: tile(&north.tile)?,
                    rotation: north.tex_rotation(),
                },
                south_face: FaceProperties {
                    texture_layer: tile(&south.tile)?,
                    rotation: south.tex_rotation(),
                },
                east_face: FaceProperties {
                    texture_layer: tile(&east.tile)?,
                    rotation: east.tex_rotation(),
                },
                west_face: FaceProperties {
                    texture_layer: tile(&west.tile)?,
                    rotation: west.tex_rotation(),
                },
            }),
            BlockShapeLayout::Custom { .. } => todo!("Custom block shapes are not yet supported"),
        };

        Ok(Block::new(layout.name, shape, tileset_handle))
    }

    fn extensions(&self) -> &[&str] {
        &["block"]
    }
}

/// Errors that can occur during block asset loading.
#[derive(Debug, thiserror::Error)]
pub enum BlockLoaderError {
    /// An error occurred while reading the block file.
    #[error("Failed to read block file: {0}")]
    Io(#[from] std::io::Error),

    /// An error occurred while parsing the block file.
    #[error("Failed to parse block file: {0}")]
    ParsingError(#[from] ron::de::SpannedError),

    /// Errors from the tileset loader.
    #[error("Tileset loading error: {0}")]
    TilesetLoaderError(#[from] LoadDirectError),

    /// A tile specified in the block file was not found in the tileset.
    #[error("Tile '{tile}' not found in tileset '{tileset}'")]
    UnknownTile {
        /// The name of the tile that was not found.
        tile: String,

        /// The name of the tileset that was searched.
        tileset: String,
    },

    /// An error occurred while resolving a relative path from the block file.
    #[error("Relative path error: {0}")]
    PathError(#[from] RelativePathError),
}
