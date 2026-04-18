use bevy::prelude::*;

/// The coordinates of a block along an infinite grid of blocks.
///
/// This may point to a block that does not exist.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Deref, DerefMut, Reflect)]
pub struct BlockPos(pub IVec3);

impl BlockPos {
    /// Creates a new block position with the given coordinates.
    #[must_use]
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self(IVec3::new(x, y, z))
    }

    /// Returns the chunk coordinates of the chunk that contains this block.
    #[must_use]
    pub fn chunk_pos(self) -> ChunkPos {
        self.into()
    }

    /// Returns the local coordinates of this block within its chunk.
    #[must_use]
    pub fn local_pos(self) -> LocalPos {
        self.into()
    }

    /// Creates a new block position by shifting this block position in the
    /// direction of the given block face.
    #[must_use]
    pub fn shift(self, face: BlockFace) -> Self {
        Self(self.0 + face.direction_vector())
    }
}

impl From<IVec3> for BlockPos {
    fn from(pos: IVec3) -> Self {
        Self(pos)
    }
}

impl std::fmt::Display for BlockPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "B({}, {}, {})", self.x, self.y, self.z)
    }
}

/// The local coordinates of a block within a chunk.
///
/// Each axis is in the range [0, 15].
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Deref, Reflect)]
pub struct LocalPos(IVec3);

impl LocalPos {
    /// Creates a new local position with the given coordinates, wrapping them
    /// into the range [0, 15] using bitwise AND.
    #[must_use]
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self(IVec3::new(x & 15, y & 15, z & 15))
    }

    /// Attempts to add the given offset to this local position, returning
    /// `None` if the result would be out of chunk bounds.
    #[must_use]
    pub fn try_add(self, other: IVec3) -> Option<Self> {
        let new_pos = self.0 + other;
        if 0 <= new_pos.x
            && new_pos.x < 16
            && 0 <= new_pos.y
            && new_pos.y < 16
            && 0 <= new_pos.z
            && new_pos.z < 16
        {
            Some(Self(new_pos))
        } else {
            None
        }
    }
}

impl From<BlockPos> for LocalPos {
    fn from(pos: BlockPos) -> Self {
        Self(IVec3::new(pos.x & 15, pos.y & 15, pos.z & 15))
    }
}

impl From<LocalPos> for IVec3 {
    fn from(pos: LocalPos) -> Self {
        Self::new(pos.x, pos.y, pos.z)
    }
}

impl From<IVec3> for LocalPos {
    fn from(pos: IVec3) -> Self {
        Self(IVec3::new(pos.x & 15, pos.y & 15, pos.z & 15))
    }
}

impl std::fmt::Display for LocalPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "L({}, {}, {})", self.x, self.y, self.z)
    }
}

/// The coordinates of a chunk along an infinite grid of chunks.
///
/// This may point to a chunk that does not exist.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Deref, DerefMut, Reflect)]
pub struct ChunkPos(pub IVec3);

impl ChunkPos {
    /// Creates a new chunk position with the given coordinates.
    #[must_use]
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self(IVec3::new(x, y, z))
    }
}

impl From<BlockPos> for ChunkPos {
    fn from(pos: BlockPos) -> Self {
        Self(IVec3::new(pos.x >> 4, pos.y >> 4, pos.z >> 4))
    }
}

impl std::fmt::Display for ChunkPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "C({}, {}, {})", self.x, self.y, self.z)
    }
}

/// The axis-aligned face of a block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum BlockFace {
    Top,
    Bottom,
    North,
    South,
    East,
    West,
}

impl BlockFace {
    /// Returns the direction vector corresponding to this block face.
    #[inline(always)]
    pub fn direction_vector(&self) -> IVec3 {
        match self {
            BlockFace::Top => IVec3::Y,
            BlockFace::Bottom => -IVec3::Y,
            BlockFace::North => IVec3::Z,
            BlockFace::South => -IVec3::Z,
            BlockFace::East => IVec3::X,
            BlockFace::West => -IVec3::X,
        }
    }

    /// Returns the normal vector corresponding to this block face.
    #[inline(always)]
    pub fn normal(&self) -> Vec3 {
        self.direction_vector().as_vec3()
    }

    /// Wraps the given normal vector into the nearest [`BlockFace`].
    ///
    /// This method assumes the vector is normalized.
    pub fn from_normal(normal: Vec3) -> Self {
        let abs = normal.abs();
        if abs.x > abs.y && abs.x > abs.z {
            if normal.x > 0.0 {
                BlockFace::East
            } else {
                BlockFace::West
            }
        } else if abs.y > abs.x && abs.y > abs.z {
            if normal.y > 0.0 {
                BlockFace::Top
            } else {
                BlockFace::Bottom
            }
        } else {
            if normal.z > 0.0 {
                BlockFace::North
            } else {
                BlockFace::South
            }
        }
    }
}
