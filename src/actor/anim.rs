use std::time::Duration;

use bevy::animation::RepeatAnimation;
use bevy::prelude::*;

use crate::actor::asset::Actor;
use crate::actor::scene::ActorInstance;

/// Component for controlling an actor's animation.
#[derive(Debug, Component, Reflect)]
pub struct ActorAnimation {
    /// The animation to play on the next update.
    to_play: Option<AnimationInstruction>,

    /// The currently playing animation.
    playing: String,

    /// The entities of the AnimationPlayers that are playing the current
    /// animation.
    players: Vec<Entity>,
}

impl ActorAnimation {
    /// Creates a new ActorAnimation with the given default animation. The
    /// default animation will be played on startup, repeating indefinitely,
    /// until another animation is queued with `play()`.
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bones_cubed::actor::scene::ActorInstance;
    /// use bones_cubed::actor::anim::ActorAnimation;
    ///
    /// fn spawn_balloon(
    ///   asset_server: Res<AssetServer>,
    ///   mut commands: Commands,
    /// ) {
    ///   commands.spawn((
    ///     ActorInstance(asset_server.load("balloon.actor")),
    ///     ActorAnimation::new("float"), // Repeats the "float" animation until another is queued with `play()`.
    ///   ));
    /// }
    /// ```
    ///
    /// `.play()` can be called on this Component immediately to define an
    /// animation to only play once.
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bones_cubed::actor::scene::ActorInstance;
    /// use bones_cubed::actor::anim::ActorAnimation;
    ///
    /// fn spawn_zombie(
    ///   asset_server: Res<AssetServer>,
    ///   mut commands: Commands,
    /// ) {
    ///   // A default animation is still required, but because it is immediately
    ///   // overridden, it will never actually be played.
    ///   let mut anim = ActorAnimation::new("idle");
    ///
    ///   // The animation we *actually* want to spawn with.
    ///   anim.play("emerge");
    ///
    ///   commands.spawn((
    ///     ActorInstance(asset_server.load("zombie.actor")),
    ///     anim,
    ///   ));
    /// }
    /// ```
    pub fn new(default_animation: impl Into<String>) -> Self {
        Self {
            to_play: None,
            playing: default_animation.into(),
            players: Vec::new(),
        }
    }

    /// Gets the name of the currently playing animation.
    pub fn playing(&self) -> &str {
        &self.playing
    }

    /// Queues an animation to be played on the next update.
    pub fn play(&mut self, anim: impl Into<String>) -> &mut AnimationInstruction {
        self.to_play = Some(AnimationInstruction {
            animation_name: anim.into(),
            ..default()
        });
        self.to_play.as_mut().unwrap()
    }
}

/// An instruction to play an animation, used in the `to_play` field of
/// `ActorAnimation`.
#[derive(Debug, Default, Clone, PartialEq, Eq, Reflect)]
pub struct AnimationInstruction {
    /// The name of the animation to play.
    animation_name: String,

    /// Whether the animation should repeat.
    repeat: RepeatAnimation,

    /// The duration of the transition to the new animation.
    transition: Duration,
}

impl AnimationInstruction {
    /// Sets whether the animation should repeat.
    pub fn repeat(&mut self) -> &mut Self {
        self.repeat = RepeatAnimation::Forever;
        self
    }

    /// Sets the animation to repeat a specific number of times.
    pub fn repeat_times(&mut self, times: u32) -> &mut Self {
        self.repeat = RepeatAnimation::Count(times);
        self
    }

    /// Sets the duration of the transition to the new animation.
    pub fn transition_time(&mut self, duration: Duration) -> &mut Self {
        self.transition = duration;
        self
    }
}

/// A system that prepares AnimationPlayers that are descendants of an Actor to
/// play the default animation of that Actor.
pub(super) fn prepare_anim_players(
    actors: Res<Assets<Actor>>,
    mut actors_query: Query<(&ActorInstance, Option<&mut ActorAnimation>)>,
    hierarchy: Query<&ChildOf>,
    mut players: Query<(Entity, &mut AnimationPlayer, &ChildOf), Added<AnimationPlayer>>,
    mut commands: Commands,
) {
    'ev_loop: for (entity, mut player, child_of) in &mut players {
        let mut actor_id = child_of.parent();
        while !actors_query.contains(actor_id) {
            if let Ok(parent_child_of) = hierarchy.get(actor_id) {
                actor_id = parent_child_of.parent();
            } else {
                warn!(
                    "AnimationPlayer {:?} is not a descendant of an Actor, skipping",
                    entity
                );
                continue 'ev_loop;
            }
        }

        let Ok((actor, mut maybe_aanim)) = actors_query.get_mut(actor_id) else {
            error!("Failed to get Actor component for parent {:?}", actor_id);
            continue 'ev_loop;
        };

        let Some(actor_asset) = actors.get(&actor.0) else {
            error!("Failed to get ActorAsset for actor {:?}", actor);
            continue 'ev_loop;
        };

        let Some(anim_graph) = actor_asset.anim_graph() else {
            warn!(
                "Entity {} ({}) spawned without an animation graph, but has an AnimationPlayer",
                actor_id,
                actor_asset.name()
            );
            continue 'ev_loop;
        };

        let anim_index;

        if let Some(aanim) = maybe_aanim.as_mut() {
            aanim.players.push(entity);

            anim_index = match actor_asset.get_animation(&aanim.playing) {
                Some(index) => index,
                None => {
                    error!(
                        "Entity {} is currently playing animation '{}', but it does not exist",
                        actor_id, aanim.playing
                    );

                    let Some((_, def_anim_index)) = actor_asset.get_default_animation() else {
                        error!("Actor {actor_asset} is missing default animation property");
                        continue 'ev_loop;
                    };

                    def_anim_index
                }
            };
        } else {
            let Some((def_anim, def_anim_index)) = actor_asset.get_default_animation() else {
                error!("Actor {actor_asset} is missing default animation property");
                continue 'ev_loop;
            };

            commands.entity(actor_id).insert(ActorAnimation {
                to_play: None,
                playing: def_anim.to_string(),
                players: vec![entity],
            });

            anim_index = def_anim_index;
        }

        commands.entity(actor_id).insert(Visibility::Inherited);

        let mut transitions = AnimationTransitions::new();
        transitions
            .play(&mut player, anim_index, Duration::ZERO)
            .repeat();

        commands
            .entity(entity)
            .insert(AnimationGraphHandle(anim_graph.clone()))
            .insert(transitions);
    }
}

/// A system that plays queued Actor animations.
pub(super) fn play_animation(
    actors: Res<Assets<Actor>>,
    mut players_query: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
    mut actors_query: Query<(Entity, &mut ActorAnimation, &ActorInstance), Changed<ActorAnimation>>,
) {
    for (entity, mut aanim, actor) in &mut actors_query {
        let Some(to_play) = aanim.to_play.take() else {
            continue;
        };

        let Some(actor_asset) = actors.get(&actor.0) else {
            error!("Failed to get ActorAsset for actor {entity}");
            continue;
        };

        let Some(anim_index) = actor_asset.get_animation(&to_play.animation_name) else {
            error!(
                "Actor {} does not have an animation named '{}'",
                actor_asset, to_play.animation_name
            );
            continue;
        };

        let Ok((mut player, mut transitions)) = players_query.get_mut(entity) else {
            error!("Failed to get AnimationPlayer for {entity}");
            continue;
        };

        transitions
            .play(&mut player, anim_index, to_play.transition)
            .set_repeat(to_play.repeat);

        aanim.playing = to_play.animation_name;
    }
}
