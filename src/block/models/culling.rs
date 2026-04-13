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

    #[inline(always)]
    fn try_occlude(&mut self, a: FaceOcclusionShape, b: FaceOcclusionShape, face_flag: Culling) {
        match a.is_occluded_by(b) {
            OcclusionResult::Occluded => self.insert(face_flag),
            OcclusionResult::NotOccluded => {}
            OcclusionResult::Unknown => self.insert(Culling::UNKNOWN),
        }
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
        let up = up.map_or(FaceOcclusionShape::Unknown, |b| b.occludes_down());
        let down = down.map_or(FaceOcclusionShape::Unknown, |b| b.occludes_up());
        let north = north.map_or(FaceOcclusionShape::Unknown, |b| b.occludes_south());
        let south = south.map_or(FaceOcclusionShape::Unknown, |b| b.occludes_north());
        let east = east.map_or(FaceOcclusionShape::Unknown, |b| b.occludes_west());
        let west = west.map_or(FaceOcclusionShape::Unknown, |b| b.occludes_east());

        if up == FaceOcclusionShape::Full
            && down == FaceOcclusionShape::Full
            && north == FaceOcclusionShape::Full
            && south == FaceOcclusionShape::Full
            && east == FaceOcclusionShape::Full
            && west == FaceOcclusionShape::Full
        {
            // includes center
            return Culling::full();
        }

        let mut culling = Culling::empty();
        culling.try_occlude(block.occludes_up(), up, Culling::POS_Y);
        culling.try_occlude(block.occludes_down(), down, Culling::NEG_Y);
        culling.try_occlude(block.occludes_north(), north, Culling::POS_Z);
        culling.try_occlude(block.occludes_south(), south, Culling::NEG_Z);
        culling.try_occlude(block.occludes_east(), east, Culling::POS_X);
        culling.try_occlude(block.occludes_west(), west, Culling::NEG_X);
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

    /// The face has an unknown occlusion shape.
    Unknown,
}

impl FaceOcclusionShape {
    /// Returns the occlusion shape for a given block model and face.
    #[inline(always)]
    pub fn is_occluded_by(&self, other: FaceOcclusionShape) -> OcclusionResult {
        match (self, other) {
            (_, FaceOcclusionShape::Unknown) => OcclusionResult::Unknown,
            (FaceOcclusionShape::Unknown, _) => OcclusionResult::Unknown,
            (_, FaceOcclusionShape::None) => OcclusionResult::NotOccluded,
            (_, FaceOcclusionShape::Full) => OcclusionResult::Occluded,
        }
    }
}

/// The result of an occlusion check for a block face.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OcclusionResult {
    /// The face is fully occluded and should not be rendered.
    Occluded,

    /// The face is not occluded and should be rendered.
    NotOccluded,

    /// The occlusion state of the face is unknown (usually due to BlockModels
    /// still loading) and could not be calculated.
    Unknown,
}
