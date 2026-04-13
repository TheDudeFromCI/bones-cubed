use bevy::prelude::*;

use crate::block::models::cube::CubeProperties;
use crate::block::models::culling::{Culling, FaceOcclusionShape};
use crate::tileset::material::Tileset;
use crate::utils::mesh::TerrainMesh;

pub mod cube;
pub mod culling;
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
    #[inline(always)]
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

    /// Gets how this block model occludes the adjacent block above it.
    #[inline(always)]
    pub fn occludes_up(&self) -> FaceOcclusionShape {
        match self {
            Self::Empty => FaceOcclusionShape::None,
            Self::Cube(_) => FaceOcclusionShape::Full,
            Self::Custom => FaceOcclusionShape::None,
        }
    }

    /// Gets how this block model occludes the adjacent block below it.
    #[inline(always)]
    pub fn occludes_down(&self) -> FaceOcclusionShape {
        match self {
            Self::Empty => FaceOcclusionShape::None,
            Self::Cube(_) => FaceOcclusionShape::Full,
            Self::Custom => FaceOcclusionShape::None,
        }
    }

    /// Gets how this block model occludes the adjacent block to the north of
    /// it.
    #[inline(always)]
    pub fn occludes_north(&self) -> FaceOcclusionShape {
        match self {
            Self::Empty => FaceOcclusionShape::None,
            Self::Cube(_) => FaceOcclusionShape::Full,
            Self::Custom => FaceOcclusionShape::None,
        }
    }

    /// Gets how this block model occludes the adjacent block to the south of
    /// it.
    #[inline(always)]
    pub fn occludes_south(&self) -> FaceOcclusionShape {
        match self {
            Self::Empty => FaceOcclusionShape::None,
            Self::Cube(_) => FaceOcclusionShape::Full,
            Self::Custom => FaceOcclusionShape::None,
        }
    }

    /// Gets how this block model occludes the adjacent block to the east of it.
    #[inline(always)]
    pub fn occludes_east(&self) -> FaceOcclusionShape {
        match self {
            Self::Empty => FaceOcclusionShape::None,
            Self::Cube(_) => FaceOcclusionShape::Full,
            Self::Custom => FaceOcclusionShape::None,
        }
    }

    /// Gets how this block model occludes the adjacent block to the west of it.
    #[inline(always)]
    pub fn occludes_west(&self) -> FaceOcclusionShape {
        match self {
            Self::Empty => FaceOcclusionShape::None,
            Self::Cube(_) => FaceOcclusionShape::Full,
            Self::Custom => FaceOcclusionShape::None,
        }
    }
}
