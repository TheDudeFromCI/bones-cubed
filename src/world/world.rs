use bevy::platform::collections::HashMap;
use bevy::prelude::*;

use crate::world::chunk::BChunk;
use crate::world::pos::ChunkPos;

/// This component acts as the root of a world, containing chunk references as
/// its children.
#[derive(Debug, Default, Component)]
#[require(Transform)]
pub struct BWorld {
    /// A map of chunk positions to their corresponding chunk entities.
    chunks: HashMap<ChunkPos, Entity>,
}

impl BWorld {
    /// Gets the entity of the chunk at the given position, if it exists.
    pub fn get_chunk(&self, pos: ChunkPos) -> Option<Entity> {
        self.chunks.get(&pos).cloned()
    }

    /// Inserts a chunk at the given position with the given entity.
    ///
    /// If a chunk already exists at that position, it will be replaced and its
    /// entity will be returned. Otherwise, `None` will be returned.
    ///
    /// This function is called internally when a chunk is spawned or despawned,
    /// and should not be called manually in most cases.
    pub fn insert_chunk(&mut self, pos: ChunkPos, entity: Entity) -> Option<Entity> {
        self.chunks.insert(pos, entity)
    }

    /// Removes the chunk at the given position, if it exists, and returns its
    /// entity.
    ///
    /// This function is called internally when a chunk is despawned, and should
    /// not be called manually in most cases.
    pub fn remove_chunk(&mut self, pos: ChunkPos) -> Option<Entity> {
        self.chunks.remove(&pos)
    }
}

/// Observer that listens for chunks being spawned, and adds them to their
/// parent world's chunk map.
pub(super) fn chunk_spawned(
    trigger: On<Add, BChunk>,
    chunks: Query<(Entity, &BChunk, &ChildOf)>,
    mut worlds: Query<&mut BWorld>,
    mut commands: Commands,
) {
    let Ok((chunk_entity, chunk, ChildOf(world_entity))) = chunks.get(trigger.entity) else {
        warn!(
            "Chunk entity {} does not have a parent world",
            trigger.entity
        );
        return;
    };

    let Ok(mut world) = worlds.get_mut(*world_entity) else {
        warn!(
            "World entity {} does not have a BWorld component",
            world_entity
        );
        return;
    };

    if let Some(old_chunk) = world.insert_chunk(chunk.pos(), chunk_entity) {
        warn!(
            "Chunk entity {} at position {} replaced chunk entity {}. Despawning old chunk.",
            chunk_entity,
            chunk.pos(),
            old_chunk
        );
        commands.entity(old_chunk).despawn();
    }
}

/// Observer that listens for chunks being despawned, and removes them from
/// their parent world's chunk map.
pub(super) fn chunk_despawned(
    trigger: On<Remove, BChunk>,
    chunks: Query<(&BChunk, &ChildOf)>,
    mut worlds: Query<&mut BWorld>,
) {
    let Ok((chunk, ChildOf(world_entity))) = chunks.get(trigger.entity) else {
        return;
    };

    let Ok(mut world) = worlds.get_mut(*world_entity) else {
        warn!(
            "World entity {} does not have a BWorld component",
            world_entity
        );
        return;
    };

    world.remove_chunk(chunk.pos());
}
