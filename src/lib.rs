//! The core plugin for Bones Cubed.

use bevy::prelude::*;

pub mod actor;
pub mod mesh;
pub mod tileset;
pub(crate) mod utils;

pub struct BonesCubedPlugin;
impl Plugin for BonesCubedPlugin {
    fn build(&self, app_: &mut App) {
        app_.add_plugins(actor::ActorPlugin)
            .add_plugins(tileset::TilesetPlugin);
    }
}
