use std::f32::consts::FRAC_PI_4;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bones_cubed::BonesCubedPlugin;
use bones_cubed::actor::anim::ActorAnimation;
use bones_cubed::actor::scene::ActorInstance;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(BonesCubedPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, spawn_random)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 15_000.,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, 0.0, -FRAC_PI_4)),
    ));
}

fn spawn_random(
    mut count: Local<u32>,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    if time.elapsed_secs() < *count as f32 * 1.6 || *count >= 5 {
        return;
    }

    commands.spawn((
        ActorInstance(asset_server.load("proto.actor")),
        Transform::from_xyz(*count as f32 * 2.0 - 5.0, 0.0, 0.0),
        ActorAnimation::new("walk"),
    ));

    *count += 1;
}
