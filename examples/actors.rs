use std::f32::consts::FRAC_PI_4;

use bevy::dev_tools::fps_overlay::FpsOverlayPlugin;
use bevy::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bones_cubed::BonesCubedPlugin;
use bones_cubed::actor::anim::ActorAnimation;
use bones_cubed::actor::scene::ActorInstance;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PanOrbitCameraPlugin)
        .add_plugins(FpsOverlayPlugin::default())
        .add_plugins(BonesCubedPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, spawn_random)
        .run();
}

fn setup(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        PanOrbitCamera::default(),
    ));

    commands.spawn((
        Transform::default(),
        Mesh3d(meshes.add(Mesh::from(Plane3d::new(Vec3::Y, Vec2::new(6.5, 6.5))))),
        MeshMaterial3d(materials.add(Color::srgb(0.2, 0.8, 0.2))),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 5_000.,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, FRAC_PI_4, -FRAC_PI_4)),
    ));
}

fn spawn_random(
    mut count: Local<u32>,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    if time.elapsed_secs() < *count as f32 * 1.6 || *count >= 6 {
        return;
    }

    commands.spawn((
        ActorInstance(asset_server.load("actors/proto.actor")),
        Transform::from_xyz(*count as f32 * 2.0 - 5.0, 0.0, 0.0),
        ActorAnimation::new("walk"),
    ));

    *count += 1;
}
