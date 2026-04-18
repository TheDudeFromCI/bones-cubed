use bevy::prelude::*;

pub mod asset;
mod filelayout;
pub mod list;
pub mod models;

/// The main plugin for the block module, which registers the necessary assets
/// and systems for working with blocks.
pub struct BlockPlugin;
impl Plugin for BlockPlugin {
    fn build(&self, app_: &mut App) {
        app_.init_asset::<asset::Block>()
            .init_asset_loader::<asset::BlockAssetLoader>()
            .init_resource::<list::BlockList>()
            .add_systems(
                Update,
                list::check_block_list_loading.in_set(BlockSystemSet::CheckBlockListLoading),
            );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub enum BlockSystemSet {
    /// A system set for checking the loading status of blocks in the block
    /// list.
    CheckBlockListLoading,
}
