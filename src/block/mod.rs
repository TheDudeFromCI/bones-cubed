use bevy::prelude::*;

pub mod asset;
mod filelayout;
pub mod list;
pub mod models;
pub mod rendered;

/// The main plugin for the block module, which registers the necessary assets
/// and systems for working with blocks.
pub struct BlockPlugin;
impl Plugin for BlockPlugin {
    fn build(&self, app_: &mut App) {
        app_.init_asset::<asset::Block>()
            .init_asset_loader::<asset::BlockAssetLoader>()
            .init_resource::<list::BlockList>()
            .register_type::<rendered::RenderedBlock>()
            .add_systems(
                Update,
                (
                    list::check_block_list_loading.in_set(BlockSystemSet::CheckBlockListLoading),
                    rendered::add_rendered_mesh.in_set(BlockSystemSet::UpdateRenderedBlocks),
                    rendered::finish_loading_rendered_block
                        .in_set(BlockSystemSet::UpdateRenderedBlocks),
                ),
            );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub enum BlockSystemSet {
    /// A system set for checking the loading status of blocks in the block
    /// list.
    CheckBlockListLoading,

    /// A system set for adding meshes to entities with a [`RenderedBlock`]
    /// component when the component is added or modified.
    UpdateRenderedBlocks,
}
