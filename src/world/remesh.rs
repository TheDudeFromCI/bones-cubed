use bevy::ecs::batching::BatchingStrategy;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::utils::Parallel;

use crate::block::asset::Block;
use crate::block::models::culling::Culling;
use crate::tileset::material::{Tileset, UseTileset};
use crate::utils::mesh::TerrainMesh;
use crate::world::chunk::BChunk;

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct NeedsRemesh;

pub(super) fn find_dirty_chunks(
    blocks: Res<Assets<Block>>,
    mut chunks: Query<(Entity, &mut BChunk)>,
    mut commands: Commands,
) {
    'chunk_loop: for (entity, mut chunk) in chunks.iter_mut() {
        if chunk.is_dirty() {
            if chunk
                .unique_block_types()
                .any(|block_type| blocks.get(block_type).is_none())
            {
                // Blocks are still loading. Wait until the next frame to check again.
                continue 'chunk_loop;
            }

            chunk.clear_dirty();
            commands.entity(entity).insert(NeedsRemesh);
        }
    }
}

pub(super) fn remesh_chunks(
    blocks: Res<Assets<Block>>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    chunks: Query<(Entity, &BChunk), With<NeedsRemesh>>,
    par_commands: ParallelCommands,
    mut commands: Commands,
) {
    if chunks.is_empty() {
        return;
    }

    for (entity, _) in chunks.iter() {
        commands
            .entity(entity)
            .remove::<NeedsRemesh>()
            .despawn_children();
    }

    let mut queue: Parallel<Vec<(Entity, Handle<Tileset>, TerrainMesh)>> = Parallel::default();

    chunks
        .par_iter()
        .batching_strategy(BatchingStrategy::fixed(1))
        .for_each_init(
            || queue.borrow_local_mut(),
            |out, (entity, chunk)| {
                let mut meshes = TilesetMeshesMap::default();
                let mut culling_cache_events = Vec::new();

                let get_model = |pos: IVec3| {
                    chunk
                        .try_get_block(pos)
                        .and_then(|block| blocks.get(&block.handle).map(|b| b.model()))
                };

                for z in 0 .. 16 {
                    for y in 0 .. 16 {
                        for x in 0 .. 16 {
                            let pos = IVec3::new(x, y, z);

                            let local_block = chunk.get_block(pos);
                            let Some(block) = blocks.get(&local_block.handle) else {
                                warn_once!(
                                    "block {:?} not found for remeshing chunk {:?}",
                                    local_block.handle,
                                    entity
                                );
                                continue;
                            };

                            let mut culling = local_block.culling;

                            if local_block.culling.contains(Culling::UNKNOWN) {
                                let up = get_model(pos + IVec3::Y);
                                let down = get_model(pos - IVec3::Y);
                                let north = get_model(pos + IVec3::Z);
                                let south = get_model(pos - IVec3::Z);
                                let east = get_model(pos + IVec3::X);
                                let west = get_model(pos - IVec3::X);

                                culling = Culling::calculate_culling(
                                    block.model(),
                                    up,
                                    down,
                                    north,
                                    south,
                                    east,
                                    west,
                                );

                                culling_cache_events.push(UpdateCullingMessage {
                                    entity,
                                    pos,
                                    culling,
                                });
                            }

                            let model = block.model();

                            let Some(tileset) = model.tileset() else {
                                continue;
                            };

                            let mesh = meshes.get(tileset);
                            let transform = Transform::from_translation(pos.as_vec3());

                            model.append_model(culling, transform, mesh);
                        }
                    }
                }

                par_commands.command_scope(|mut commands| {
                    for event in culling_cache_events {
                        commands.trigger(event);
                    }
                });

                for (tileset, mesh) in meshes.map.into_iter() {
                    out.push((entity, tileset.clone(), mesh));
                }
            },
        );

    for (entity, tileset, mesh) in queue.drain() {
        commands.spawn((
            ChildOf(entity),
            UseTileset(tileset),
            Mesh3d(mesh_assets.add(mesh)),
        ));
    }
}

#[derive(Default)]
struct TilesetMeshesMap {
    map: HashMap<Handle<Tileset>, TerrainMesh>,
}

impl TilesetMeshesMap {
    fn get(&mut self, tileset: &Handle<Tileset>) -> &mut TerrainMesh {
        if !self.map.contains_key(tileset) {
            self.map.insert(tileset.clone(), TerrainMesh::new());
        }

        self.map
            .get_mut(tileset)
            .expect("tileset mesh must exist after insertion")
    }
}

/// Message sent when a block's culling information is updated during the remesh
/// to cache the culling information.
#[derive(Debug, EntityEvent)]
pub struct UpdateCullingMessage {
    entity: Entity,
    pos: IVec3,
    culling: Culling,
}

/// Updates the culling cache for a block in a chunk when an
/// [`UpdateCullingMessage`] is received.
pub(super) fn update_culling_cache(
    update: On<UpdateCullingMessage>,
    mut chunks: Query<&mut BChunk>,
) {
    let Ok(mut chunk) = chunks.get_mut(update.entity) else {
        warn!(
            "Chunk {:?} not found for updating culling cache at pos {:?}",
            update.entity, update.pos
        );
        return;
    };

    chunk.get_block_mut(update.pos).culling = update.culling;
}
