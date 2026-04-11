use bevy::prelude::*;

pub mod asset;
pub mod culling;
mod filelayout;
mod models;
pub mod shape;

/// The main plugin for the block module, which registers the necessary assets
/// and systems for working with blocks.
pub struct BlockPlugin;
impl Plugin for BlockPlugin {
    fn build(&self, app_: &mut App) {
        app_.init_asset::<asset::Block>()
            .init_asset_loader::<asset::BlockAssetLoader>();
    }
}
