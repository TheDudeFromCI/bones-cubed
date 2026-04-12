use bitflags::bitflags;

use crate::block::models::BlockModel;

bitflags! {
    /// Defines which faces of a block should be culled.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Culling: u8 {
        const POS_X   = 0b00000001;
        const NEG_X   = 0b00000010;
        const POS_Y   = 0b00000100;
        const NEG_Y   = 0b00001000;
        const POS_Z   = 0b00010000;
        const NEG_Z   = 0b00100000;
        const CENTER  = 0b01000000;

        /// A special flag indicating that the block's culling state is unknown
        /// and should be recalculated.
        const UNKNOWN = 0b10000000;
    }
}

impl Culling {
    /// Returns a Culling value with all faces set to be culled.
    pub const fn full() -> Self {
        Culling::from_bits_retain(0b01111111)
    }

    pub fn calculate_culling(
        block: &BlockModel,
        up: Option<&BlockModel>,
        down: Option<&BlockModel>,
        north: Option<&BlockModel>,
        south: Option<&BlockModel>,
        east: Option<&BlockModel>,
        west: Option<&BlockModel>,
    ) -> Culling {
        let mut culling = Culling::empty();

        let up = up.map_or(FaceOcclusionShape::None, |b| b.occludes_down());
        let down = down.map_or(FaceOcclusionShape::None, |b| b.occludes_up());
        let north = north.map_or(FaceOcclusionShape::None, |b| b.occludes_south());
        let south = south.map_or(FaceOcclusionShape::None, |b| b.occludes_north());
        let east = east.map_or(FaceOcclusionShape::None, |b| b.occludes_west());
        let west = west.map_or(FaceOcclusionShape::None, |b| b.occludes_east());

        if up == FaceOcclusionShape::Full
            && down == FaceOcclusionShape::Full
            && north == FaceOcclusionShape::Full
            && south == FaceOcclusionShape::Full
            && east == FaceOcclusionShape::Full
            && west == FaceOcclusionShape::Full
        {
            culling |= Culling::CENTER;
        }

        if block.occludes_up().is_occluded_by(up) {
            culling |= Culling::POS_Y;
        }
        if block.occludes_down().is_occluded_by(down) {
            culling |= Culling::NEG_Y;
        }
        if block.occludes_north().is_occluded_by(north) {
            culling |= Culling::POS_Z;
        }
        if block.occludes_south().is_occluded_by(south) {
            culling |= Culling::NEG_Z;
        }
        if block.occludes_east().is_occluded_by(east) {
            culling |= Culling::POS_X;
        }
        if block.occludes_west().is_occluded_by(west) {
            culling |= Culling::NEG_X;
        }

        culling
    }
}

/// Defines how a block faces occludes adjacent blocks for the purposes of
/// occlusion culling and mesh generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FaceOcclusionShape {
    /// The face does not occlude the adjacent block at all.
    None,

    /// The face fully occludes the adjacent block.
    Full,
}

impl FaceOcclusionShape {
    /// Returns the occlusion shape for a given block model and face.
    pub fn is_occluded_by(&self, other: FaceOcclusionShape) -> bool {
        match (self, other) {
            (FaceOcclusionShape::None, _) => false,
            (FaceOcclusionShape::Full, FaceOcclusionShape::None) => false,
            (FaceOcclusionShape::Full, FaceOcclusionShape::Full) => true,
        }
    }
}
