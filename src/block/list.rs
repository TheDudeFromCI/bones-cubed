use bevy::asset::LoadedFolder;
use bevy::prelude::*;

use crate::block::asset::Block;

/// A resource that keeps blocks loaded and easily accessible for use in the
/// world.
///
/// The primary usecase of this resource is to load a number of blocks at the
/// start of the game or level so that the assets are already available and
/// ready to be used when the player needs them, preventing the need to delay
/// chunk meshing or other block-related operations while waiting for assets to
/// load.
///
/// Because this resource only stores handles to blocks, it is safe to use the
/// [`AssetServer::load`] function to retrieve block handles without needing to
/// access this resource at all, aside from checking to see if the list is fully
/// loaded.
///
/// Note: To prevent memory overhead, BlockIDs use 32 bit indexing instead of 64
/// bit, meaning this list is limited to a max 2^32 unique blocks at a time.
/// Given that this is hundreds of gigabytes of data, this should be more than
/// enough for nearly any usecase.
#[derive(Debug, Default, Resource)]
pub struct BlockList {
    /// A list of all blocks that are currently loaded and available for use.
    blocks: Vec<(Handle<Block>, BlockID)>,

    /// A list of all folders that are currently being loaded. Once a folder is
    /// finished loading, its blocks will be added to the `blocks` list and the
    /// folder will be removed from this list.
    folders: Vec<Handle<LoadedFolder>>,

    /// Whether all blocks in the list have finished loading.
    loaded: bool,

    /// A generation ID that is incremented every time the block list is
    /// cleared. This can be used to determine if the block list has changed
    /// since the last time it was checked.
    generation: u32,
}

impl BlockList {
    /// Gets an iterator over all the blocks in the list.
    ///
    /// Note the iterator may contain some blocks that are still loading. In
    /// addition, if there are still folders being loaded, this list may
    /// not contain all the blocks that will eventually be in the list.
    pub fn blocks(&self) -> impl Iterator<Item = &(Handle<Block>, BlockID)> {
        self.blocks.iter()
    }

    /// Gets the block handle for a given block ID.
    ///
    /// Returns an error if the block ID is invalid, which can happen if the
    /// block list was cleared since the block ID was generated.
    pub fn get(&self, id: BlockID) -> Result<&Handle<Block>, InvalidBlockIDError> {
        if id.generation() != self.generation {
            return Err(InvalidBlockIDError(id));
        }

        Ok(&self.blocks[id.index()].0)
    }

    /// Adds a block to the list.
    ///
    /// Note: Added a block to this list will automatically mark the list as
    /// still loading, even if the handle is already loaded. The loading status
    /// will be updated in the next frame to reflect the correct load status.
    ///
    /// Panics if the block list already contains 2^32 blocks, which should
    /// never happen in practice.
    pub fn add(&mut self, block: Handle<Block>) -> BlockID {
        if self.len() >= u32::MAX as usize {
            panic!("BlockList cannot contain more than 2^32 blocks");
        }

        let id = BlockID(self.generation, self.blocks.len() as u32);
        self.blocks.push((block, id));
        self.loaded = false;
        id
    }

    /// Returns the number of blocks in the list.
    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    /// Adds a folder of blocks to the list.
    ///
    /// If the folder contains non-block assets, they will be ignored.
    pub fn add_folder(&mut self, folder: Handle<LoadedFolder>) {
        self.folders.push(folder);
        self.loaded = false;
    }

    /// Returns `true` if any blocks are still loading.
    pub fn is_loading(&self) -> bool {
        !self.loaded
    }

    /// Clears all blocks from the list. This will also invalidate all existing
    /// block IDs.
    ///
    /// Note: This does not actually unload any assets, it just clears the list
    /// of blocks allowing for the blocks to be dropped if there are no other
    /// references to them.
    ///
    /// Panics if the generation ID overflows, which would require 2^32 clears,
    /// so this should never happen in practice.
    pub fn clear(&mut self) {
        if self.generation >= u32::MAX {
            panic!("BlockList generation ID overflow");
        }

        self.blocks.clear();
        self.folders.clear();
        self.loaded = true;
        self.generation += 1;
    }

    /// Gets the generation ID of this block list.
    ///
    /// This value is incremented every time the block list is cleared, and can
    /// be used to determine if existing BlockIDs are still valid.
    pub fn generation_id(&self) -> u32 {
        self.generation
    }
}

/// A unique identifier for a block in the block list. This is used to reference
/// blocks in a way that is more efficient than using the block handle directly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockID(u32, u32);

impl BlockID {
    /// Gets the generation ID of this block ID, which is used to determine if
    /// the block list has been cleared since the last time it was checked.
    ///
    /// When the [`BlockList`] is cleared, all previous block IDs become invalid
    /// and a new generation ID is generated.
    pub fn generation(&self) -> u32 {
        self.0
    }

    /// Gets the index of this block ID in the block list.
    pub fn index(&self) -> usize {
        self.1 as usize
    }
}

impl std::fmt::Display for BlockID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Block({})", self.1)
    }
}

/// An error type for invalid block IDs, which can occur if the block list was
/// cleared since the block ID was generated.
#[derive(Debug, thiserror::Error)]
#[error("Invalid block ID: {0}")]
pub struct InvalidBlockIDError(BlockID);

/// Checks if any blocks in the block list are still loading, and if not, marks
/// the block list as fully loaded.
pub(super) fn check_block_list_loading(
    blocks: Res<Assets<Block>>,
    folders: Res<Assets<LoadedFolder>>,
    mut block_list: ResMut<BlockList>,
) {
    if block_list.loaded {
        return;
    }

    for folder_index in (0 .. block_list.folders.len()).rev() {
        let folder_handle = &block_list.folders[folder_index];
        if let Some(folder) = folders.get(folder_handle) {
            block_list.folders.remove(folder_index);

            for asset in &folder.handles {
                let Ok(block) = asset.clone().try_typed() else {
                    continue;
                };

                block_list.add(block);
            }
        }
    }

    for (block, _) in block_list.blocks() {
        if !blocks.contains(block) {
            return;
        }
    }

    block_list.loaded = true;
}
