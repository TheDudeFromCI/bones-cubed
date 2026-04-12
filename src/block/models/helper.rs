use bevy::asset::{Handle, LoadContext};

use crate::block::asset::BlockLoaderError;
use crate::tileset::asset::TilesetLoaderSettings;
use crate::tileset::material::Tileset;
use crate::utils::asset::ContextRelativePathEtx;

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
