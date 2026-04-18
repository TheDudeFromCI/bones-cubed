use std::f32::consts::FRAC_PI_4;

use bevy::dev_tools::fps_overlay::FpsOverlayPlugin;
use bevy::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bones_cubed::BonesCubedPlugin;
use bones_cubed::block::list::BlockList;
use bones_cubed::world::chunk::BChunk;
use bones_cubed::world::param::BChunkWriter;
use bones_cubed::world::picking::{B3Camera, PickableChunk, PickableWorld};
use bones_cubed::world::pos::{ChunkPos, LocalPos};
use bones_cubed::world::remesh::RenderedChunk;
use bones_cubed::world::world::BWorld;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PanOrbitCameraPlugin)
        .add_plugins(FpsOverlayPlugin::default())
        .add_plugins(BonesCubedPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (place_block, destroy_block))
        .run();
}

fn setup(
    asset_server: Res<AssetServer>,
    mut block_list: ResMut<BlockList>,
    mut commands: Commands,
) {
    commands.spawn((
        B3Camera, /* Mark this camera as a 'Boned Cubed Camera' so it can be used by the picking
                   * system. */
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

    // keep the blocks loaded
    block_list.add_folder(asset_server.load_folder("blocks"));

    let air = asset_server.load("blocks/air.block");
    let dirt = asset_server.load("blocks/dirt.block");
    let grass = asset_server.load("blocks/grass.block");

    let world = commands
        .spawn((
            BWorld::default(),
            PickableWorld::default(),
            Visibility::default(),
        ))
        .id();

    for cx in -1 ..= 1 {
        for cz in -1 ..= 1 {
            let mut chunk = BChunk::new(ChunkPos::new(cx, 0, cz), air.clone());
            for x in 0 .. 16 {
                for z in 0 .. 16 {
                    chunk.set_block_unchecked(LocalPos::new(x, 0, z), &dirt);
                    chunk.set_block_unchecked(LocalPos::new(x, 1, z), &grass);
                }
            }

            commands.spawn((
                ChildOf(world),
                chunk,
                PickableChunk,
                RenderedChunk::default(),
            ));
        }
    }
}

fn place_block(
    buttons: Res<ButtonInput<KeyCode>>,
    asset_server: Res<AssetServer>,
    worlds: Query<(&PickableWorld, &BWorld)>,
    mut chunk_editor: BChunkWriter,
) {
    if !buttons.just_pressed(KeyCode::KeyE) {
        return;
    }

    for (pickable, world) in worlds.iter() {
        let Some(hovered) = pickable.hovered else {
            continue;
        };

        let pos = hovered.pos.shift(hovered.face);

        let chunk_pos: ChunkPos = pos.into();
        let Some(chunk_entity) = world.get_chunk(chunk_pos) else {
            continue;
        };

        let Ok(mut chunk) = chunk_editor.chunk_mut(chunk_entity) else {
            continue;
        };

        let debug = asset_server.load("blocks/debug.block");

        let local_pos = pos.local_pos();
        chunk.set_block(local_pos, &debug);
    }
}

fn destroy_block(
    buttons: Res<ButtonInput<KeyCode>>,
    asset_server: Res<AssetServer>,
    worlds: Query<(&PickableWorld, &BWorld)>,
    mut chunk_editor: BChunkWriter,
) {
    if !buttons.just_pressed(KeyCode::KeyQ) {
        return;
    }

    for (pickable, world) in worlds.iter() {
        let Some(hovered) = pickable.hovered else {
            continue;
        };

        let chunk_pos = hovered.pos.chunk_pos();
        let Some(chunk_entity) = world.get_chunk(chunk_pos) else {
            continue;
        };

        let Ok(mut chunk) = chunk_editor.chunk_mut(chunk_entity) else {
            continue;
        };

        let air = asset_server.load("blocks/air.block");

        let local_pos = hovered.pos.local_pos();
        chunk.set_block(local_pos, &air);
    }
}
