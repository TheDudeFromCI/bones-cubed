use std::f32::consts::{FRAC_PI_4, PI};

use bevy::dev_tools::fps_overlay::FpsOverlayPlugin;
use bevy::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bones_cubed::BonesCubedPlugin;
use bones_cubed::block::asset::Block;
use bones_cubed::world::chunk::BChunk;
use bones_cubed::world::param::BChunkWriter;
use bones_cubed::world::pos::{ChunkPos, LocalPos};
use bones_cubed::world::remesh::RenderedChunk;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PanOrbitCameraPlugin)
        .add_plugins(FpsOverlayPlugin::default())
        .add_plugins(BonesCubedPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}

#[derive(Resource)]
struct BlockList {
    air: Handle<Block>,
    grass: Handle<Block>,
    dirt: Handle<Block>,

    chunk: Entity,
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

    let chunk = commands
        .spawn((
            BChunk::new(ChunkPos::default(), air.clone()),
            RenderedChunk::default(),
            Transform::from_translation(Vec3::new(-8.0, -8.0, -8.0)),
        ))
        .id();

    commands.insert_resource(BlockList {
        air,
        grass,
        dirt,
        chunk,
    });
}

fn update(time: Res<Time>, block_list: Res<BlockList>, mut chunk_editor: BChunkWriter) {
    let t = time.elapsed_secs();

    let mut editor = chunk_editor.chunk_mut(block_list.chunk).unwrap();

    for x in 0 .. 16 {
        for y in 0 .. 16 {
            for z in 0 .. 16 {
                let a = (x as f32 * 0.5 + t).sin() * 0.5;
                let b = (z as f32 * 0.5 + t * PI).cos() * 0.5;
                let c = a + b + 3.0;
                let block = if c > (y as f32 + 1.0) * 0.5 {
                    &block_list.dirt
                } else if c > y as f32 * 0.5 {
                    &block_list.grass
                } else {
                    &block_list.air
                };

                let pos = LocalPos::new(x, y, z);
                editor.set_block(pos, block);
            }
        }
    }
}
