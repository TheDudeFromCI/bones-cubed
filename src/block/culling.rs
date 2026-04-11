use bitflags::bitflags;

bitflags! {
    /// Defines which faces of a block should be culled.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Culling: u8 {
        const POS_X = 0b00000001;
        const NEG_X = 0b00000010;
        const POS_Y = 0b00000100;
        const NEG_Y = 0b00001000;
        const POS_Z = 0b00010000;
        const NEG_Z = 0b00100000;
    }
}
