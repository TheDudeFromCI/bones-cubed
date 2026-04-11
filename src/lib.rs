//! The core plugin for Bones Cubed.

use bevy::prelude::*;

pub mod actor;
pub mod block;
pub mod tileset;
pub mod utils;

pub struct BonesCubedPlugin;
impl Plugin for BonesCubedPlugin {
    fn build(&self, app_: &mut App) {
        app_.add_plugins(actor::ActorPlugin)
            .add_plugins(tileset::TilesetPlugin)
            .add_plugins(block::BlockPlugin);
    }
}
