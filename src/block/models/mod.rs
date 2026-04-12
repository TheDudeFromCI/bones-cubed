use bevy::prelude::*;

use crate::block::culling::Culling;
use crate::block::models::cube::CubeProperties;
use crate::tileset::material::Tileset;
use crate::utils::mesh::TerrainMesh;

pub mod cube;
pub mod face;
mod helper;

/// Defines the shape of a block, which can be used for rendering and mesh
/// generation.
#[derive(Debug, Default, Clone)]
pub enum BlockModel {
    /// An empty block with no geometry.
    #[default]
    Empty,

    /// A standard cube block.
    Cube(CubeProperties),

    /// A custom block with user-defined geometry.
    ///
    /// For occlusion culling purposes, custom blocks are treated as empty, not
    /// culling any faces of surrounding geometry.
    Custom,
}

impl BlockModel {
    /// Appends the geometry of this block shape to the given mesh, applying the
    /// given transform and culling.
    pub fn append_model(&self, culling: Culling, transform: Transform, mesh: &mut TerrainMesh) {
        match self {
            BlockModel::Empty => {}
            BlockModel::Cube(properties) => cube::build(properties, culling, transform, mesh),
            BlockModel::Custom => {
                todo!("Custom block shape rendering is not yet implemented");
            }
        }
    }

    /// Returns true if this block shape has no geometry and should not be
    /// rendered.
    pub fn is_empty(&self) -> bool {
        matches!(self, BlockModel::Empty)
    }

    /// Gets the tileset used by this block shape, if any.
    pub fn tileset(&self) -> Option<&Handle<Tileset>> {
        match self {
            BlockModel::Empty | BlockModel::Custom => None,
            BlockModel::Cube(properties) => Some(&properties.tileset),
        }
    }
}
