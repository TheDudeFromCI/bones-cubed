use bevy::prelude::*;

use crate::block::asset::Block;
use crate::block::models::culling::Culling;
use crate::tileset::material::UseTileset;
use crate::world::mesh::TerrainMesh;

/// This component can be used to render the block as a standalone model.
///
/// When attached to an entity, this component will automatically convert the
/// block model into a mesh attached to the entity, alongside the tileset
/// material.
#[derive(Component, Reflect)]
pub struct RenderedBlock(pub Handle<Block>);

/// System that adds a mesh to entities with a [`RenderedBlock`] component when
/// the component is added or modified.
pub(super) fn add_rendered_mesh(
    blocks: Res<Assets<Block>>,
    mut meshes: ResMut<Assets<Mesh>>,
    rendered_blocks: Query<
        (Entity, &RenderedBlock),
        Or<(Added<RenderedBlock>, Changed<RenderedBlock>)>,
    >,
    mut commands: Commands,
) {
    for (entity, rendered_mesh) in &rendered_blocks {
        let Some(block) = blocks.get(&rendered_mesh.0) else {
            // block still loading
            continue;
        };

        update_mesh(entity, block, &mut meshes, &mut commands);
    }
}

/// System that updates the mesh of entities with a [`RenderedBlock`] component
/// when the block asset is loaded or modified.
pub(super) fn finish_loading_rendered_block(
    mut asset_events: MessageReader<AssetEvent<Block>>,
    blocks: Res<Assets<Block>>,
    mut meshes: ResMut<Assets<Mesh>>,
    rendered_blocks: Query<(Entity, &RenderedBlock)>,
    mut commands: Commands,
) {
    for ev in asset_events.read() {
        match ev {
            AssetEvent::LoadedWithDependencies { id } | AssetEvent::Modified { id } => {
                for (entity, rendered_block) in &rendered_blocks {
                    if rendered_block.0.id() == *id {
                        let Some(block) = blocks.get(&rendered_block.0) else {
                            error!("Block was modified but is not loaded");
                            continue;
                        };

                        update_mesh(entity, block, &mut meshes, &mut commands);
                    }
                }
            }
            _ => continue,
        }
    }
}

fn update_mesh(
    entity: Entity,
    block: &Block,
    meshes: &mut ResMut<Assets<Mesh>>,
    commands: &mut Commands,
) {
    let Some(tileset) = block.model().tileset() else {
        commands.entity(entity).remove::<Mesh3d>();
        return;
    };

    let mut terrain = TerrainMesh::new();
    block.model().append_model(
        Culling::empty(),
        Transform::from_xyz(-0.5, -0.5, -0.5),
        &mut terrain,
    );

    if terrain.tri_count() == 0 {
        // No geometry, so remove the mesh if it exists
        commands.entity(entity).remove::<Mesh3d>();
        return;
    }

    commands
        .entity(entity)
        .insert((Mesh3d(meshes.add(terrain)), UseTileset(tileset.clone())));
}
