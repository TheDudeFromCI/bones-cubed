//! The Actor plugin is responsible for loading and handling actors in the
//! framework. An actor is a 3D model with a skeleton and animations than can
//! move and interact within the world.
//!
//! [`Actor`](asset::Actor) assets are loaded from `.actor` files which define
//! the properties of the actor. These assets can than be instantiated in the
//! world using the [`ActorInstance`](scene::ActorInstance) component. When an
//! instance is spawned, the actor model is built from the asset and added to
//! the world.

use bevy::prelude::*;

pub mod anim;
pub mod asset;
pub mod scene;

/// Plugin for the actor system. This plugin is responsible for loading and
/// handling actors in the framework.
pub struct ActorPlugin;
impl Plugin for ActorPlugin {
    fn build(&self, app_: &mut App) {
        app_.init_asset::<asset::Actor>()
            .init_asset_loader::<asset::ActorAssetLoader>()
            .register_type::<scene::ActorInstance>()
            .add_systems(
                Update,
                (
                    anim::prepare_anim_players.in_set(ActorSystemSets::PrepareAnimPlayers),
                    anim::play_animation.in_set(ActorSystemSets::PlayAnimation),
                    scene::build_scene_on_asset_load.in_set(ActorSystemSets::BuildActorModel),
                    scene::build_scene_from_existing_asset.in_set(ActorSystemSets::BuildActorModel),
                ),
            )
            .configure_sets(
                Update,
                (
                    ActorSystemSets::BuildActorModel.before(ActorSystemSets::PrepareAnimPlayers),
                    ActorSystemSets::PrepareAnimPlayers.before(ActorSystemSets::PlayAnimation),
                ),
            );
    }
}

/// System sets for the actor plugin. These are used to order the systems in the
/// update stage.
#[derive(Debug, SystemSet, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActorSystemSets {
    /// System set for building the actor model from the asset.
    ///
    /// Systems in this set are only run once when spawning a new
    /// [`ActorInstance`](scene::ActorInstance) or when the asset is loaded.
    BuildActorModel,

    /// System set for preparing the animation players.
    ///
    /// Systems in this set are responsible for setting up [`AnimationPlayer`]s
    /// before they are used.
    PrepareAnimPlayers,

    /// System set for playing queued animations.
    PlayAnimation,
}
