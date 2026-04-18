use bevy::prelude::*;

use crate::tileset::TilesetSystemSet;

pub mod chunk;
pub mod mesh;
pub mod param;
pub mod picking;
pub mod pos;
pub mod remesh;
pub mod world;

pub struct WorldPlugin;
impl Plugin for WorldPlugin {
    fn build(&self, app_: &mut App) {
        app_.add_plugins(MeshPickingPlugin)
            .register_type::<chunk::BChunk>()
            .register_type::<chunk::BChunkCulling>()
            .register_type::<remesh::RenderedChunk>()
            .register_type::<remesh::NeedsRemesh>()
            .register_type::<remesh::ChunkSubMesh>()
            .register_type::<picking::PickableChunk>()
            .add_systems(
                Update,
                (
                    remesh::find_dirty_chunks.in_set(WorldSystemSets::FindDirtyChunks),
                    remesh::remesh_chunks.in_set(WorldSystemSets::RemeshChunks),
                    picking::update_hover.in_set(WorldSystemSets::UpdateHover),
                ),
            )
            .configure_sets(
                Update,
                (
                    WorldSystemSets::FindDirtyChunks.before(WorldSystemSets::RemeshChunks),
                    WorldSystemSets::RemeshChunks.before(TilesetSystemSet::UpdateMaterialReference),
                    WorldSystemSets::RemeshChunks.before(WorldSystemSets::UpdateHover),
                ),
            )
            .add_observer(world::chunk_spawned)
            .add_observer(world::chunk_despawned)
            .add_observer(chunk::update_chunk_transform);
    }
}

/// System sets for the world plugin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub enum WorldSystemSets {
    FindDirtyChunks,
    RemeshChunks,
    UpdateHover,
}
