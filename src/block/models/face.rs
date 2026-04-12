use bevy::math::{Mat2, Vec2};

/// The properties of a block face, which determine how the face is rendered.
#[derive(Debug, Clone, Copy)]
pub struct FaceProperties {
    /// The index of the texture layer to use for this face.
    pub texture_layer: u16,

    /// The rotation to apply to the texture on this face.
    pub rotation: TextureRotation,
}

/// The rotation and mirroring to apply to a block face's texture.
#[derive(Debug, Clone, Copy)]
pub struct TextureRotation {
    /// The uv transformation matrix to apply to the texture coordinates of this
    /// face.
    pub matrix: Mat2,
}

impl TextureRotation {
    /// Creates a new `TextureRotation` with the identity transformation.
    pub fn identity() -> Self {
        Self {
            matrix: Mat2::IDENTITY,
        }
    }

    /// Applies a horizontal mirroring to the texture.
    pub fn mirror_x(&mut self) {
        self.matrix *= Mat2::from_cols(Vec2::new(-1.0, 0.0), Vec2::new(0.0, 1.0));
    }

    /// Applies a vertical mirroring to the texture.
    pub fn mirror_y(&mut self) {
        self.matrix *= Mat2::from_cols(Vec2::new(1.0, 0.0), Vec2::new(0.0, -1.0));
    }

    /// Rotates the texture by the given angle in degrees.
    pub fn rotate(&mut self, angle: f32) {
        let rad = angle.to_radians();
        self.matrix *= Mat2::from_angle(rad);
    }
}

impl Default for TextureRotation {
    fn default() -> Self {
        Self::identity()
    }
}
