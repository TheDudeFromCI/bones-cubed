use bevy::prelude::*;

use crate::tileset::TilesetSystemSet;

pub mod chunk;
pub mod param;
pub mod remesh;

pub struct WorldPlugin;
impl Plugin for WorldPlugin {
    fn build(&self, app_: &mut App) {
        app_.register_type::<chunk::BChunk>()
            .register_type::<chunk::BChunkCulling>()
            .register_type::<remesh::RenderedChunk>()
            .register_type::<remesh::NeedsRemesh>()
            .add_systems(
                Update,
                (
                    remesh::find_dirty_chunks.in_set(WorldSystemSets::FindDirtyChunks),
                    remesh::remesh_chunks.in_set(WorldSystemSets::RemeshChunks),
                ),
            )
            .configure_sets(
                Update,
                (
                    WorldSystemSets::FindDirtyChunks.before(WorldSystemSets::RemeshChunks),
                    WorldSystemSets::RemeshChunks.before(TilesetSystemSet::UpdateMaterialReference),
                ),
            );
    }
}

/// System sets for the world plugin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub enum WorldSystemSets {
    FindDirtyChunks,
    RemeshChunks,
}
