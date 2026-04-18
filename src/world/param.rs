use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::block::asset::Block;
use crate::block::models::culling::Culling;
use crate::world::chunk::{BChunk, BChunkCulling};
use crate::world::pos::LocalPos;
use crate::world::remesh::RenderedChunk;

/// System parameter for safely accessing chunks and their related data for
/// reading.
#[derive(SystemParam)]
pub struct BChunkReader<'w, 's> {
    blocks: Res<'w, Assets<Block>>,
    chunks: Query<
        'w,
        's,
        (
            &'static BChunk,
            Option<&'static BChunkCulling>,
            Option<&'static RenderedChunk>,
        ),
    >,
}

impl<'w, 's> BChunkReader<'w, 's> {
    /// Gets a reader for the specified chunk entity.
    ///
    /// This will return an error if the chunk entity does not exist in the
    /// world or does not have a [`BChunk`] component.
    pub fn chunk<'a>(
        &'a self,
        chunk_entity: Entity,
    ) -> Result<BChunkReaderInner<'w, 'a>, BChunkReaderError> {
        let Ok((chunk, culling, rendered)) = self.chunks.get(chunk_entity) else {
            return Err(BChunkReaderError::ChunkNotFound(chunk_entity));
        };

        Ok(BChunkReaderInner {
            blocks: &self.blocks,
            chunk,
            culling,
            rendered,
        })
    }
}

/// A helper struct that provides access to a chunk and its related data for
/// reading from a single chunk safely.
pub struct BChunkReaderInner<'w, 'a> {
    blocks: &'a Res<'w, Assets<Block>>,
    chunk: &'a BChunk,
    culling: Option<&'a BChunkCulling>,
    rendered: Option<&'a RenderedChunk>,
}

impl<'w, 'a> BChunkReaderInner<'w, 'a> {
    /// Checks if the chunk has been marked as dirty and needs to be remeshed.
    ///
    /// If the chunk does not have a [`RenderedChunk`] component, this will
    /// always return `false`.
    pub fn is_dirty(&self) -> bool {
        self.rendered.map_or(false, |r| r.is_dirty)
    }

    /// Checks if the chunk is empty, meaning that all blocks in the chunk are
    /// of a block type with an empty model.
    pub fn is_empty(&self) -> bool {
        self.chunk.unique_block_types().all(|b| {
            self.blocks
                .get(b)
                .map_or(false, |block| block.model().is_empty())
        })
    }

    /// Gets the block type at the given position in the chunk.
    pub fn get_block(&self, pos: LocalPos) -> &Handle<Block> {
        self.chunk.get_block(pos)
    }

    /// Gets the culling value at the given position in the chunk, if the chunk
    /// has a [`BChunkCulling`] component.
    pub fn get_culling(&self, pos: LocalPos) -> Option<Culling> {
        self.culling.as_ref().map(|c| c.get_culling(pos))
    }

    /// Checks if the chunk has any block types that are still loading, meaning
    /// that the block type is not present in the `Assets<Block>` resource.
    pub fn is_loading(&self) -> bool {
        self.chunk
            .unique_block_types()
            .any(|block_type| self.blocks.get(block_type).is_none())
    }
}

/// Error type for operations on a [`BChunkReader`].
#[derive(Debug, thiserror::Error)]
pub enum BChunkReaderError {
    /// The specified chunk entity was not found in the world.
    #[error("chunk entity {0:?} not found")]
    ChunkNotFound(Entity),
}

/// System parameter for safely accessing chunks and their related data for
/// mutating safely.
#[derive(SystemParam)]
pub struct BChunkWriter<'w, 's> {
    blocks: Res<'w, Assets<Block>>,
    chunks: Query<
        'w,
        's,
        (
            &'static mut BChunk,
            Option<&'static mut BChunkCulling>,
            Option<&'static mut RenderedChunk>,
        ),
    >,
}

impl<'w, 's> BChunkWriter<'w, 's> {
    /// Gets a writer for the specified chunk entity.
    ///
    /// This will return an error if the chunk entity does not exist in the
    /// world or does not have a [`BChunk`] component.
    pub fn chunk_mut<'a>(
        &'a mut self,
        chunk_entity: Entity,
    ) -> Result<BChunkWriterInner<'w, 'a>, BChunkWriterError> {
        let Ok((chunk, culling, rendered)) = self.chunks.get_mut(chunk_entity) else {
            return Err(BChunkWriterError::ChunkNotFound(chunk_entity));
        };

        Ok(BChunkWriterInner {
            blocks: &self.blocks,
            chunk,
            culling,
            rendered,
        })
    }
}

/// A helper struct that provides access to a chunk and its related data for
/// mutating a single chunk safely.
pub struct BChunkWriterInner<'w, 'a> {
    blocks: &'a Res<'w, Assets<Block>>,
    chunk: Mut<'a, BChunk>,
    culling: Option<Mut<'a, BChunkCulling>>,
    rendered: Option<Mut<'a, RenderedChunk>>,
}

impl<'w, 'a> BChunkWriterInner<'w, 'a> {
    /// Checks if the chunk has been marked as dirty and needs to be remeshed.
    ///
    /// If the chunk does not have a [`RenderedChunk`] component, this will
    /// always return `false`.
    pub fn is_dirty(&self) -> bool {
        self.rendered.as_ref().map_or(false, |r| r.is_dirty)
    }

    /// Marks the chunk as dirty or not dirty, indicating whether it needs to be
    /// remeshed.
    ///
    /// If the chunk does not have a [`RenderedChunk`] component, this function
    /// will do nothing.
    pub fn set_dirty(&mut self, dirty: bool) {
        if let Some(rendered) = self.rendered.as_mut() {
            rendered.is_dirty = dirty;
        }
    }

    /// Gets the block at the given position in the chunk.
    pub fn get_block(&self, pos: LocalPos) -> &Handle<Block> {
        self.chunk.get_block(pos)
    }

    /// Sets the block at the given position in the chunk to the given block
    /// type.
    ///
    /// This function will also update the culling information for the block if
    /// the chunk has a [`BChunkCulling`] component, and mark the chunk as dirty
    /// if it has a [`RenderedChunk`] component. (Chunks is not marked as dirty
    /// if the block at the given position is already of the given block type.)
    ///
    /// If the chunk contains blocks that are still loading, culling information
    /// will be marked as `UNKNOWN`, and calculated during the next chunk
    /// remesh.
    pub fn set_block(&mut self, pos: LocalPos, block: &Handle<Block>) {
        if !self.chunk.set_block_unchecked(pos, block) {
            return;
        }

        if let Some(rendered) = self.rendered.as_mut() {
            rendered.is_dirty = true;
        }

        if let Some(culling) = self.culling.as_mut() {
            // TODO: This performs up to 49 lookups in the block array.
            // Needs optimization.

            culling.recalculate_culling_at(&self.blocks, &self.chunk, pos);

            if let Some(pos) = pos.try_add(IVec3::X) {
                culling.recalculate_culling_at(&self.blocks, &self.chunk, pos);
            }

            if let Some(pos) = pos.try_add(-IVec3::X) {
                culling.recalculate_culling_at(&self.blocks, &self.chunk, pos);
            }

            if let Some(pos) = pos.try_add(IVec3::Y) {
                culling.recalculate_culling_at(&self.blocks, &self.chunk, pos);
            }

            if let Some(pos) = pos.try_add(-IVec3::Y) {
                culling.recalculate_culling_at(&self.blocks, &self.chunk, pos);
            }

            if let Some(pos) = pos.try_add(IVec3::Z) {
                culling.recalculate_culling_at(&self.blocks, &self.chunk, pos);
            }

            if let Some(pos) = pos.try_add(-IVec3::Z) {
                culling.recalculate_culling_at(&self.blocks, &self.chunk, pos);
            }
        }
    }

    /// Recalculates the culling information for the block at the given position
    /// in the chunk, if the chunk has a [`BChunkCulling`] component.
    ///
    /// If there was a change in the culling value, this will also mark the
    /// chunk as dirty if it has a [`RenderedChunk`] component.
    pub fn recalculate_culling_at(&mut self, pos: LocalPos) {
        let Some(culling) = self.culling.as_mut() else {
            return;
        };

        if !culling.recalculate_culling_at(&self.blocks, &self.chunk, pos) {
            return;
        }

        if let Some(rendered) = self.rendered.as_mut() {
            rendered.is_dirty = true;
        }
    }

    /// Checks if the chunk has any block types that are still loading, meaning
    /// that the block type is not present in the `Assets<Block>` resource.
    pub fn is_loading(&self) -> bool {
        self.chunk
            .unique_block_types()
            .any(|block_type| self.blocks.get(block_type).is_none())
    }
}

/// Error type for operations on a [`BChunkWriter`].
#[derive(Debug, thiserror::Error)]
pub enum BChunkWriterError {
    /// The specified chunk entity was not found in the world.
    #[error("chunk entity {0:?} not found")]
    ChunkNotFound(Entity),
}
