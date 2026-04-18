use bevy::ecs::batching::BatchingStrategy;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::utils::Parallel;

use crate::block::asset::Block;
use crate::block::models::culling::Culling;
use crate::tileset::material::{Tileset, UseTileset};
use crate::world::chunk::{BChunk, BChunkCulling};
use crate::world::mesh::TerrainMesh;
use crate::world::picking::PickableChunk;

/// A simple component that indicates that a chunk should be rendered.
///
/// Chunks without this component will be ignored by the rendering system,
/// treating them a pure data containers.
#[derive(Debug, Component, Reflect)]
#[require(BChunkCulling, Transform, Visibility)]
pub struct RenderedChunk {
    /// Whether the chunk is dirty and needs to be remeshed. This is set to
    /// `true` by default, so that chunks will be remeshed on creation to
    /// render their fill block.
    pub is_dirty: bool,

    /// The child sub-meshes of this chunk, mapped by their entity ID. This is
    /// used to keep track of the sub-meshes that are generated for each tileset
    /// during the remeshing process.
    child_sub_meshes: HashMap<Handle<Tileset>, Entity>,
}

impl RenderedChunk {
    /// Creates a new [`RenderedChunk`] marked as dirty.
    pub const fn dirty() -> Self {
        Self {
            is_dirty: true,
            child_sub_meshes: HashMap::new(),
        }
    }

    /// Creates a new [`RenderedChunk`] marked as not dirty.
    pub const fn not_dirty() -> Self {
        Self {
            is_dirty: false,
            child_sub_meshes: HashMap::new(),
        }
    }

    /// Gets a reference to the child sub-meshes of this chunk and their
    /// corresponding tileset, mapped by their entity ID.
    pub fn get_child_sub_meshes(&self) -> &HashMap<Handle<Tileset>, Entity> {
        &self.child_sub_meshes
    }
}

impl Default for RenderedChunk {
    fn default() -> Self {
        Self::dirty()
    }
}

/// A simple marker component that indicates that the entity is a sub-mesh of a
/// chunk.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Component, Reflect)]
pub struct ChunkSubMesh;

/// A simple temporary marker component that indicates that a chunk needs to be
/// remeshed.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Component, Reflect)]
#[component(storage = "SparseSet")]
pub(crate) struct NeedsRemesh;

/// System that finds all dirty chunks and marks them for remeshing.
pub(super) fn find_dirty_chunks(
    blocks: Res<Assets<Block>>,
    mut chunks: Query<(Entity, &BChunk, &mut RenderedChunk)>,
    mut commands: Commands,
) {
    'chunk_loop: for (entity, chunk, mut rendered_chunk) in chunks.iter_mut() {
        if rendered_chunk.is_dirty {
            if chunk
                .unique_block_types()
                .any(|block_type| blocks.get(block_type).is_none())
            {
                // Blocks are still loading. Wait until the next frame to check again.
                continue 'chunk_loop;
            }

            rendered_chunk.is_dirty = false;
            commands.entity(entity).insert(NeedsRemesh);
        }
    }
}

/// System that remeshes all chunks marked with [`NeedsRemesh`].
pub(super) fn remesh_chunks(
    blocks: Res<Assets<Block>>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut chunks: Query<
        (
            Entity,
            &BChunk,
            &mut BChunkCulling,
            &mut RenderedChunk,
            Has<PickableChunk>,
        ),
        With<NeedsRemesh>,
    >,
    par_commands: ParallelCommands,
    mut commands: Commands,
) {
    if chunks.is_empty() {
        return;
    }

    for (entity, _, _, _, _) in chunks.iter() {
        commands.entity(entity).remove::<NeedsRemesh>();
    }

    let mut queue: Parallel<Vec<(Entity, Mesh)>> = Parallel::default();

    chunks
        .par_iter_mut()
        .batching_strategy(BatchingStrategy::fixed(1))
        .for_each_init(
            || queue.borrow_local_mut(),
            |out, (entity, chunk, mut chunk_culling, mut rendered, is_pickable)| {
                let mut meshes = TilesetMeshesMap::default();

                for z in 0 .. 16 {
                    for y in 0 .. 16 {
                        for x in 0 .. 16 {
                            let pos = IVec3::new(x, y, z).into();

                            let block_handle = chunk.get_block(pos);
                            let Some(block) = blocks.get(block_handle) else {
                                warn_once!(
                                    "Block {:?} not found while remeshing chunk {:?}",
                                    block_handle,
                                    entity
                                );
                                continue;
                            };

                            let mut culling = chunk_culling.get_culling(pos);
                            if culling.contains(Culling::UNKNOWN) {
                                chunk_culling.recalculate_culling_at(&blocks, chunk, pos);
                                culling = chunk_culling.get_culling(pos);
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

                meshes.map.retain(|_, mesh| !mesh.indices().is_empty());

                rendered
                    .child_sub_meshes
                    .extract_if(|tileset, _| !meshes.map.contains_key(tileset))
                    .for_each(|(_, id)| {
                        par_commands.command_scope(move |mut commands| {
                            commands.entity(id).despawn();
                        });
                    });

                for (tileset, mesh) in meshes.map.into_iter() {
                    if rendered.child_sub_meshes.contains_key(&tileset) {
                        let id = rendered.child_sub_meshes[&tileset];
                        out.push((id, mesh.into()));
                    } else {
                        let tileset_clone = tileset.clone();
                        let id = par_commands.command_scope(move |mut commands| {
                            let mut cmd = commands.spawn((
                                ChildOf(entity),
                                ChunkSubMesh,
                                UseTileset(tileset_clone),
                            ));

                            if is_pickable {
                                cmd.insert(PickableChunk);
                            }

                            cmd.id()
                        });

                        rendered.child_sub_meshes.insert(tileset, id);
                        out.push((id, mesh.into()));
                    }
                }
            },
        );

    for (entity, mesh) in queue.drain() {
        commands
            .entity(entity)
            .insert(Mesh3d(mesh_assets.add(mesh)));
    }
}

/// A simple helper struct that tracks the meshes being generated for each
/// tileset during the remeshing process, allowing us to reuse the same mesh for
/// multiple blocks of the same tileset within a chunk.
#[derive(Default)]
struct TilesetMeshesMap {
    /// The map of tileset to mesh being generated for that tileset.
    map: HashMap<Handle<Tileset>, TerrainMesh>,
}

impl TilesetMeshesMap {
    /// Gets the mesh for the given tileset, creating a new one if it doesn't
    /// exist yet.
    fn get(&mut self, tileset: &Handle<Tileset>) -> &mut TerrainMesh {
        if !self.map.contains_key(tileset) {
            self.map.insert(tileset.clone(), TerrainMesh::new());
        }

        self.map
            .get_mut(tileset)
            .expect("tileset mesh must exist after insertion")
    }
}
