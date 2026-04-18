use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext, LoadDirectError};
use bevy::prelude::*;
use ron::extensions::Extensions;

use crate::block::filelayout::{BlockFileLayout, BlockModelLayout};
use crate::block::models;
use crate::block::models::BlockModel;
use crate::utils::asset::RelativePathError;

/// An asset that defines the properties of a block type.
#[derive(Debug, Clone, Asset, TypePath)]
pub struct Block {
    /// The name of this block type.
    name: Box<str>,

    /// The model for this block type, which determines how it is rendered and
    /// culled.
    model: BlockModel,
}

impl Block {
    /// Creates a new Block asset with the given name and model.
    pub fn new(name: impl Into<Box<str>>, model: BlockModel) -> Self {
        Self {
            name: name.into(),
            model,
        }
    }

    /// Gets the display name of this block asset.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Gets the model of this block asset.
    pub fn model(&self) -> &BlockModel {
        &self.model
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

        let shape = match layout.model {
            BlockModelLayout::Empty => BlockModel::Empty,
            BlockModelLayout::Cube(properties) => models::cube::load(properties, ctx).await?,
            BlockModelLayout::Custom => todo!("Custom block shapes are not yet supported"),
        };

        info!("Loaded block asset '{}' from {:?}", layout.name, ctx.path());

        Ok(Block::new(layout.name, shape))
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
