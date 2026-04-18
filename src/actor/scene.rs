use bevy::asset::AsAssetId;
use bevy::prelude::*;

use crate::actor::asset::Actor;

#[derive(Debug, Component, Reflect, Deref)]
#[require(Transform, Visibility)]
pub struct ActorInstance(pub Handle<Actor>);

impl AsAssetId for ActorInstance {
    type Asset = Actor;

    fn as_asset_id(&self) -> AssetId<Self::Asset> {
        self.0.id()
    }
}

pub(super) fn build_scene_on_asset_load(
    actors: Res<Assets<Actor>>,
    actors_query: Query<(Entity, &ActorInstance)>,
    mut asset_events: MessageReader<AssetEvent<Actor>>,
    mut commands: Commands,
) {
    for ev in asset_events.read() {
        if let AssetEvent::LoadedWithDependencies { id } = ev {
            for (entity, actor) in actors_query
                .iter()
                .filter(|(_, actor)| actor.as_asset_id() == *id)
            {
                let Some(actor_asset) = actors.get(&actor.0) else {
                    error!("Failed to get ActorAsset for actor {:?}", actor);
                    continue;
                };

                let name = Name::new(actor_asset.name().to_string());

                commands
                    .entity(entity)
                    .insert((SceneRoot(actor_asset.scene().clone()), Visibility::Hidden))
                    .insert_if_new(name);
            }
        }
    }
}

pub(super) fn build_scene_from_existing_asset(
    actors: Res<Assets<Actor>>,
    actors_query: Query<(Entity, &ActorInstance), Added<ActorInstance>>,
    mut commands: Commands,
) {
    for (entity, actor) in &actors_query {
        let Some(actor_asset) = actors.get(&actor.0) else {
            // Still loading.
            continue;
        };

        let name = Name::new(actor_asset.name().to_string());

        commands
            .entity(entity)
            .insert((SceneRoot(actor_asset.scene().clone()), Visibility::Hidden))
            .insert_if_new(name);
    }
}
