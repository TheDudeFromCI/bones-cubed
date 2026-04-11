//! This module implement the tileset system, which is responsible for loading
//! and managing tilesets in the game. A tileset is a collection of textures
//! that can be used to render block models.
//!
//! While a world can contain multiple tilesets, each block may only be
//! associated with one tileset. In addition, blocks from multiple tilesets will
//! always generate as separate meshes. However, using a single, large tileset
//! may result in lots of unused textures taking up memory, so finding a balance
//! between the number of tilesets and the number of textures in each tileset is
//! important for performance.

use bevy::asset::LoadContext;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::shader::load_shader_library;

use crate::tileset::asset::MaterialInitializer;
use crate::tileset::material::TilesetMaterial;

pub mod asset;
mod filelayout;
pub mod material;

/// The main plugin for the tileset system.
pub struct TilesetPlugin;
impl Plugin for TilesetPlugin {
    fn build(&self, app: &mut App) {
        app.register_tileset_material::<material::DefaultTilesetMaterial>();

        let constructor = app
            .world_mut()
            .remove_resource::<TilesetMaterialConstructor>()
            .unwrap();

        app.init_asset::<material::Tileset>()
            .register_asset_loader(asset::TilesetLoader {
                materials: constructor.materials,
            });

        load_shader_library!(app, "shader.wgsl");
        load_shader_library!(app, "prepass.wgsl");
    }
}

pub trait RegisterTilesetMaterialExt {
    fn register_tileset_material<M>(&mut self) -> &mut Self
    where
        M: TilesetMaterial,
        MaterialPlugin<M>: Plugin;
}

impl RegisterTilesetMaterialExt for App {
    fn register_tileset_material<M>(&mut self) -> &mut Self
    where
        M: TilesetMaterial,
        MaterialPlugin<M>: Plugin,
    {
        if !self.get_added_plugins::<TilesetPlugin>().is_empty() {
            panic!("Tileset materials must be registered before adding the TilesetPlugin!");
        }

        if let Some(mut constructor) = self
            .world_mut()
            .get_resource_mut::<TilesetMaterialConstructor>()
        {
            constructor.add_material::<M>();
        } else {
            let mut constructor = TilesetMaterialConstructor::default();
            constructor.add_material::<M>();
            self.insert_resource(constructor);
        }

        self.add_plugins(MaterialPlugin::<M>::default())
            .add_systems(
                Update,
                (
                    material::apply_tileset_material::<M>
                        .in_set(TilesetSystemSet::UpdateMaterialReference),
                    material::update_tileset_material::<M>
                        .in_set(TilesetSystemSet::UpdateMaterialReference),
                    material::remove_tileset_material::<M>
                        .in_set(TilesetSystemSet::UpdateMaterialReference),
                ),
            )
    }
}

/// System sets for the tileset plugin.
#[derive(Debug, SystemSet, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TilesetSystemSet {
    /// Update the material reference of entities using tilesets.
    UpdateMaterialReference,
}

#[derive(Default, Resource)]
struct TilesetMaterialConstructor {
    materials: HashMap<Box<str>, MaterialInitializer>,
}

impl TilesetMaterialConstructor {
    /// Adds a material to the constructor, allowing it to be used in tileset
    /// files.
    pub fn add_material<M: TilesetMaterial + 'static>(&mut self) {
        if self.materials.contains_key(M::name()) {
            panic!(
                "A material with the name '{}' is already registered!",
                M::name()
            );
        }

        let init = Box::new(|settings, ctx: &mut LoadContext<'_>| {
            let material = M::init(settings);
            ctx.add_labeled_asset("material".to_owned(), material)
                .untyped()
        });
        self.materials.insert(M::name().into(), init);
    }
}
