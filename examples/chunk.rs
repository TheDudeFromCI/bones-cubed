use std::f32::consts::FRAC_PI_4;

use bevy::dev_tools::fps_overlay::FpsOverlayPlugin;
use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bones_cubed::BonesCubedPlugin;
use bones_cubed::world::chunk::BChunk;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                present_mode: PresentMode::Immediate,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(PanOrbitCameraPlugin)
        .add_plugins(FpsOverlayPlugin::default())
        .add_plugins(BonesCubedPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        PanOrbitCamera::default(),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 15_000.,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, FRAC_PI_4, -FRAC_PI_4)),
    ));

    let air = asset_server.load("blocks/air.block");
    let grass = asset_server.load("blocks/grass.block");
    let dirt = asset_server.load("blocks/dirt.block");

    let mut chunk = BChunk::new(air);

    for x in 0 .. 16 {
        for y in 0 .. 16 {
            for z in 0 .. 16 {
                let a = (x as f32 * 0.25).sin();
                let b = (z as f32 * 0.25).cos();
                let c = a + b + 3.0;
                if c > (y as f32 + 1.0) * 0.4 {
                    chunk.set_block(IVec3::new(x, y, z), &dirt);
                } else if c > y as f32 * 0.4 {
                    chunk.set_block(IVec3::new(x, y, z), &grass);
                }
            }
        }
    }

    commands.spawn((
        chunk,
        Transform::from_translation(Vec3::new(-8.0, -8.0, -8.0)),
    ));
}
