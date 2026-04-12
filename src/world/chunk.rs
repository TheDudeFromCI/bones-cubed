use bevy::platform::collections::HashMap;
use bevy::prelude::*;

use crate::block::asset::Block;
use crate::block::culling::Culling;

/// A chunk of blocks in the world, represented as a 16x16x16 grid of block
/// types.
#[derive(Debug, Component)]
#[require(Transform, Visibility)]
pub struct BChunk {
    /// A small data container that tracks the unique block types present in
    /// this chunk.
    unique: UniqueBlockTypes,

    /// The blocks in this chunk, stored as a flat vector of length 4096
    /// (16x16x16).
    blocks: Vec<LocalBlock>,

    /// A flag indicating whether this chunk has been modified since the last
    /// time it was remeshed.
    dirty: bool,
}

impl BChunk {
    /// Creates a new chunk filled with the given block type.
    pub fn new(fill: Handle<Block>) -> Self {
        Self {
            unique: UniqueBlockTypes {
                types: {
                    let mut map = HashMap::default();
                    map.insert(fill.clone(), 16 * 16 * 16);
                    map
                },
            },
            blocks: vec![
                LocalBlock {
                    handle: fill,
                    culling: Culling::empty(),
                };
                16 * 16 * 16
            ],
            dirty: true,
        }
    }

    /// Gets the block at the given position in this chunk.
    pub fn get_block(&self, pos: IVec3) -> &LocalBlock {
        let index = index(pos);
        &self.blocks[index]
    }

    /// Sets the block at the given position to the given block type.
    pub fn set_block(&mut self, pos: IVec3, block: &Handle<Block>) {
        let index = index(pos);

        if self.blocks[index].handle == *block {
            return;
        }

        self.unique.decrement(&self.blocks[index].handle);
        self.blocks[index].handle = block.clone();
        self.unique.increment(block);

        //TODO: update culling information here instead of in the remesh system

        self.dirty = true;
    }

    /// Checks if this chunk is dirty, meaning that it has been modified since
    /// the last time it was remeshed.
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Clears the dirty flag on this chunk, indicating that it has been
    /// remeshed.
    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    /// Gets an iterator over the unique block types present in this chunk.
    pub fn unique_block_types(&self) -> impl Iterator<Item = &Handle<Block>> {
        self.unique.types.keys()
    }
}

/// A block in a chunk, along with its culling information.
#[derive(Debug, Clone, Component)]
pub struct LocalBlock {
    /// The type of this block, represented as a handle to a Block asset.
    pub handle: Handle<Block>,

    /// The culling information for this block, which determines which faces of
    /// the block should be rendered.
    pub culling: Culling,
}

/// A small data container that tracks the unique block types present in a
/// chunk, along with their counts.
#[derive(Debug, Default)]
struct UniqueBlockTypes {
    /// A map of block types to the number of times they appear in a chunk. This
    /// is used to efficiently track which block types are present in a chunk,
    /// which can be useful for optimization purposes such as waiting for blocks
    /// to load before remeshing a chunk.
    types: HashMap<Handle<Block>, usize>,
}

impl UniqueBlockTypes {
    /// Increments the count of the given block type, adding it to the map if it
    /// is not already present.
    fn increment(&mut self, block: &Handle<Block>) {
        if !self.types.contains_key(block) {
            self.types.insert(block.clone(), 1);
        } else {
            *self.types.get_mut(block).unwrap() += 1;
        }
    }

    /// Decrements the count of the given block type, and removes it from the
    /// map if the count reaches zero.
    fn decrement(&mut self, block: &Handle<Block>) {
        if let Some(count) = self.types.get_mut(block) {
            *count -= 1;
            if *count == 0 {
                self.types.remove(block);
            }
        }
    }
}

/// Converts a block position within a chunk to an index in the blocks vector.
fn index(pos: IVec3) -> usize {
    let IVec3 { x, y, z } = pos;
    ((x & 15) + (y & 15) * 16 + (z & 15) * 16 * 16) as usize
}
