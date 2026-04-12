use std::f32::consts::FRAC_PI_4;

use bevy::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bones_cubed::BonesCubedPlugin;
use bones_cubed::block::asset::Block;
use bones_cubed::block::culling::Culling;
use bones_cubed::tileset::material::UseTileset;
use bones_cubed::utils::mesh::TerrainMesh;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BonesCubedPlugin)
        .add_plugins(PanOrbitCameraPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, build_cube_model)
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

    commands.spawn(CubeModel(asset_server.load("blocks/debug.block")));
}

#[derive(Component)]
struct CubeModel(Handle<Block>);

fn build_cube_model(
    blocks: Res<Assets<Block>>,
    mut meshes: ResMut<Assets<Mesh>>,
    query: Query<(Entity, &CubeModel)>,
    mut commands: Commands,
) {
    for (entity, cube_model) in &query {
        let Some(block) = blocks.get(&cube_model.0) else {
            // block still loading
            continue;
        };

        let mut terrain = TerrainMesh::new();
        block
            .model()
            .append_model(Culling::empty(), Transform::default(), &mut terrain);

        commands.spawn((
            Transform::from_xyz(-0.5, -0.5, -0.5),
            Mesh3d(meshes.add(terrain)),
            UseTileset(
                block
                    .model()
                    .tileset()
                    .expect("Cubes always have a tileset")
                    .clone(),
            ),
        ));

        commands.entity(entity).despawn();
    }
}
