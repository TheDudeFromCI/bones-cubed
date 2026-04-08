use std::f32::consts::FRAC_PI_4;

use bevy::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bones_cubed::BonesCubedPlugin;
use bones_cubed::mesh::{TerrainMesh, TerrainQuad, TerrainVertex};
use bones_cubed::tileset::material::{Tileset, UseTileset};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BonesCubedPlugin)
        .add_plugins(PanOrbitCameraPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(asset_server: Res<AssetServer>, mut meshes: ResMut<Assets<Mesh>>, mut commands: Commands) {
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

    let tileset_handle: Handle<Tileset> = asset_server.load("tilesets/overworld/overworld.tiles");

    let mut terrain = TerrainMesh::new();

    for i in 0 .. 3 {
        let offset = Vec3::X * i as f32;

        terrain.add_quad(TerrainQuad(
            TerrainVertex {
                position: Vec3::new(0.0, 0.0, 0.0) + offset,
                uv: Vec2::new(0.0, 0.0),
                layer: i,
                normal: Vec3::Y,
                color: Color::srgb(1.0, 1.0, 1.0),
            },
            TerrainVertex {
                position: Vec3::new(0.0, 0.0, 1.0) + offset,
                uv: Vec2::new(0.0, 1.0),
                layer: i,
                normal: Vec3::Y,
                color: Color::srgb(1.0, 1.0, 1.0),
            },
            TerrainVertex {
                position: Vec3::new(1.0, 0.0, 1.0) + offset,
                uv: Vec2::new(1.0, 1.0),
                layer: i,
                normal: Vec3::Y,
                color: Color::srgb(1.0, 1.0, 1.0),
            },
            TerrainVertex {
                position: Vec3::new(1.0, 0.0, 0.0) + offset,
                uv: Vec2::new(1.0, 0.0),
                layer: i,
                normal: Vec3::Y,
                color: Color::srgb(1.0, 1.0, 1.0),
            },
        ));
    }

    commands.spawn((
        Transform::default(),
        Mesh3d(meshes.add(terrain)),
        UseTileset(tileset_handle),
    ));
}
