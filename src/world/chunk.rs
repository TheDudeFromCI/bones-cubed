use bevy::platform::collections::HashMap;
use bevy::prelude::*;

use crate::block::asset::Block;
use crate::block::models::BlockModel;
use crate::block::models::culling::Culling;

/// A chunk of blocks in the world, represented as a 16x16x16 grid of block
/// types.
#[derive(Debug, Component, Reflect)]
pub struct BChunk {
    /// A small data container that tracks the unique block types present in
    /// this chunk.
    unique: UniqueBlockTypes,

    /// The blocks in this chunk, stored as a flat vector of length 4096
    /// (16x16x16).
    #[reflect(ignore)]
    blocks: Vec<Handle<Block>>,
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
            blocks: vec![fill; 16 * 16 * 16],
        }
    }

    /// Gets the block at the given position in this chunk.
    ///
    /// If the position is out of bounds, the coordinates will be wrapped
    /// around. For example, a position of `(-1, 7, 17)` will be treated as
    /// `(15, 7, 1)`.
    pub fn get_block(&self, pos: IVec3) -> &Handle<Block> {
        let index = index(pos);
        &self.blocks[index]
    }

    /// Gets the block at the given position in this chunk, returning None if
    /// the position is out of bounds.
    pub fn try_get_block(&self, pos: IVec3) -> Option<&Handle<Block>> {
        if !in_bounds(pos) {
            None
        } else {
            Some(self.get_block(pos))
        }
    }

    /// Sets the block at the given position to the given block type.
    ///
    /// Returns `true` if the block was changed, or `false` if the block was
    /// already of the given block type.
    ///
    /// If the position is out of bounds, the coordinates will be wrapped
    /// around. For example, a position of `(-1, 7, 17)` will be treated as
    /// `(15, 7, 1)`.
    ///
    /// **Note:** This method *does not* update the culling information for the
    /// block. If you need to update the culling information as well, use the
    /// [`BChunkEditor`] system param to set the block instead, which will also
    /// mark the chunk as dirty for remeshing. (Note that culling information is
    /// recalculated automatically during the first remesh, meaning it is safe
    /// to call this method on a chunk before spawning it for the firs time.)
    pub fn set_block_unchecked(&mut self, pos: IVec3, block: &Handle<Block>) -> bool {
        let index = index(pos);

        if self.blocks[index] == *block {
            return false;
        }

        self.unique.decrement(&self.blocks[index]);
        self.blocks[index] = block.clone();
        self.unique.increment(block);
        true
    }

    /// Gets an iterator over the unique block types present in this chunk.
    pub fn unique_block_types(&self) -> impl Iterator<Item = &Handle<Block>> {
        self.unique.types.keys()
    }
}

/// Stores the model culling information for each block within the chunk. This
/// is used to optimize mesh generation and rendering by determining which faces
/// of a block are visible based on the adjacent blocks.
#[derive(Debug, Component, Reflect)]
pub struct BChunkCulling {
    #[reflect(ignore)]
    blocks: Vec<Culling>,
}

impl BChunkCulling {
    /// Creates a new BChunkCulling with all blocks initialized to `UNKNOWN`
    /// culling value.
    ///
    /// Any `UNKNOWN` culling values will be recalculated during remeshing, or
    /// can be manually recalculated using the [`Self::recalculate_culling_at`]
    /// method.
    pub fn new() -> Self {
        Self {
            blocks: vec![Culling::UNKNOWN; 16 * 16 * 16],
        }
    }

    /// Gets the culling information for the block at the given position in this
    /// chunk.
    ///
    /// If the position is out of bounds, the coordinates will be wrapped
    /// around. For example, a position of `(-1, 7, 17)` will be treated as
    /// `(15, 7, 1)`.
    pub fn get_culling(&self, pos: IVec3) -> Culling {
        let index = index(pos);
        self.blocks[index]
    }

    /// Gets the culling information for the block at the given position in this
    /// chunk, returning None if the position is out of bounds.
    pub fn try_get_culling(&self, pos: IVec3) -> Option<Culling> {
        if !in_bounds(pos) {
            None
        } else {
            Some(self.get_culling(pos))
        }
    }

    /// Sets the culling information for the block at the given position to the
    /// given culling value.
    ///
    /// Returns `true` if the culling value was changed, or `false` if the new
    /// culling value matches the existing culling value.
    ///
    /// If the position is out of bounds, the coordinates will be wrapped
    /// around. For example, a position of `(-1, 7, 17)` will be treated as
    /// `(15, 7, 1)`.
    ///
    /// This method does not account for the block stored at the given position,
    /// which may lead to odd rendering if used incorrectly. It is recommended
    /// to use the [`Self::recalculate_culling_at`] method instead, which will
    /// calculate the correct culling value based on the correct block models.
    pub fn set_culling_unchecked(&mut self, pos: IVec3, culling: Culling) -> bool {
        let index = index(pos);
        let old_culling = self.blocks[index];
        self.blocks[index] = culling;
        old_culling != culling
    }

    /// Recalculates the culling information for the block at the given position
    /// based on the block types of the adjacent blocks.
    ///
    /// Does nothing if the position is out of bounds.
    ///
    /// Returns true if the culling value was modified, or false if the culling
    /// value was already correct.
    pub fn recalculate_culling_at(
        &mut self,
        blocks: &Res<Assets<Block>>,
        chunk: &BChunk,
        pos: IVec3,
    ) -> bool {
        if !in_bounds(pos) {
            return false;
        }

        let get_model = |pos: IVec3| {
            chunk
                .try_get_block(pos)
                .and_then(|block| blocks.get(block).map(|b| b.model()))
        };

        let cull_value = Culling::calculate_culling(
            get_model(pos).unwrap_or(&BlockModel::Empty),
            get_model(pos + IVec3::Y),
            get_model(pos - IVec3::Y),
            get_model(pos + IVec3::Z),
            get_model(pos - IVec3::Z),
            get_model(pos + IVec3::X),
            get_model(pos - IVec3::X),
        );
        self.set_culling_unchecked(pos, cull_value)
    }
}

impl Default for BChunkCulling {
    fn default() -> Self {
        Self::new()
    }
}

/// A small data container that tracks the unique block types present in a
/// chunk, along with their counts.
#[derive(Debug, Default, Reflect)]
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

/// Checks if the given block position is within the bounds of a chunk (0 to 15
/// in each dimension).
fn in_bounds(pos: IVec3) -> bool {
    pos.x >= 0 && pos.x < 16 && pos.y >= 0 && pos.y < 16 && pos.z >= 0 && pos.z < 16
}
