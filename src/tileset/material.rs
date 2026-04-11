use std::any::TypeId;
use std::fmt;

use bevy::asset::AsAssetId;
use bevy::mesh::{MeshVertexAttribute, MeshVertexBufferLayoutRef, VertexFormat};
use bevy::pbr::{MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup,
    RenderPipelineDescriptor,
    SpecializedMeshPipelineError,
};
use bevy::shader::ShaderRef;

/// A vertex attribute that stores the texture index to use for that face.
pub const ATTRIBUTE_UV_LAYER: MeshVertexAttribute =
    MeshVertexAttribute::new("UvLayer", 4039395644538880, VertexFormat::Uint32);

/// The default shader path for tilesets.
pub const DEFAULT_SHADER_PATH: &str = "embedded://bones_cubed/tileset/shader.wgsl";

/// The default prepass shader path for tilesets.
pub const DEFAULT_PREPASS_SHADER_PATH: &str = "embedded://bones_cubed/tileset/prepass.wgsl";

/// The tileset asset, which contains the material and blend mode for a tileset.
#[derive(Clone, Asset, TypePath)]
pub struct Tileset {
    /// The name of this tileset.
    name: String,

    /// The list of tile names defined in this tileset.
    tile_names: Vec<Box<str>>,

    /// The material of the tileset to use for rendering.
    material: UntypedHandle,

    /// The name of the material type, used for error messages when the material
    /// type is incorrect.
    material_name: Box<str>,
}

impl Tileset {
    /// Create a new tileset with the given name and tiles, and the material
    /// handle.
    pub fn new<M: TilesetMaterial>(
        name: String,
        tile_names: Vec<Box<str>>,
        material: Handle<M>,
    ) -> Self {
        Self {
            name,
            tile_names,
            material: material.into(),
            material_name: M::name().into(),
        }
    }

    /// Create a new tileset with the given name and tile names, but with an
    /// untyped material handle.
    pub(super) fn new_untyped(
        name: String,
        tile_names: Vec<Box<str>>,
        material: UntypedHandle,
        material_name: Box<str>,
    ) -> Self {
        Self {
            name,
            tile_names,
            material,
            material_name,
        }
    }

    /// Get the display name of the tileset, or a default name if it is not set.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the names of the tiles in the tileset.
    pub fn tile_names(&self) -> &[Box<str>] {
        &self.tile_names
    }

    /// Get the index of a tile by its name, or `None` if it does not exist.
    ///
    /// Note that this method is O(n) over the number of tiles, so it is not
    /// recommended to use it frequently. Indices never change after loading,
    /// so it is recommended to cache the index of a tile.
    pub fn tile_index(&self, name: &str) -> Option<u16> {
        self.tile_names
            .iter()
            .position(|n| n.as_ref() == name)
            .map(|i| i as u16)
    }

    /// Get the material of the tileset.
    pub fn material(&self) -> &UntypedHandle {
        &self.material
    }

    /// Get the name of the material type of the tileset.
    pub fn material_name(&self) -> &str {
        &self.material_name
    }
}

impl fmt::Display for Tileset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.name().fmt(f)
    }
}

/// A trait for defining tileset materials.
pub trait TilesetMaterial: Material + Default {
    /// Initialize the material with the given settings.
    ///
    /// This is called from the
    /// [`TilesetLoader`](crate::tileset::asset::TilesetLoader) based of the
    /// settings provided in the file.
    fn init(settings: TilesetMaterialSettings) -> Self;

    /// Get the unique name of the tileset material. This is used to verify that
    /// the correct material type is used for a tileset, and to provide better
    /// error messages when the material type is incorrect.
    fn name() -> &'static str;
}

/// The settings for initializing a tileset material, which is used to create a
/// tileset material from the properties in a `.tiles` file.
#[derive(Debug, Clone)]
pub struct TilesetMaterialSettings {
    /// The tileset texture image of the material.
    pub texture: Handle<Image>,

    /// The alpha mode of the material.
    pub alpha_mode: AlphaMode,
}

/// The default tileset material, which uses the default tileset shader and
/// prepass shader.
#[derive(Debug, Default, Clone, Asset, TypePath, AsBindGroup)]
pub struct DefaultTilesetMaterial {
    /// The tileset texture image of the material.
    #[texture(0, dimension = "2d_array")]
    #[sampler(1)]
    texture: Handle<Image>,

    /// The alpha mode of the material.
    alpha_mode: AlphaMode,
}

impl Material for DefaultTilesetMaterial {
    fn vertex_shader() -> ShaderRef {
        DEFAULT_SHADER_PATH.into()
    }

    fn fragment_shader() -> ShaderRef {
        DEFAULT_SHADER_PATH.into()
    }

    fn prepass_vertex_shader() -> ShaderRef {
        DEFAULT_PREPASS_SHADER_PATH.into()
    }

    fn prepass_fragment_shader() -> ShaderRef {
        DEFAULT_PREPASS_SHADER_PATH.into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }

    fn specialize(
        _: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout.0.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_NORMAL.at_shader_location(1),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(2),
            ATTRIBUTE_UV_LAYER.at_shader_location(3),
            Mesh::ATTRIBUTE_COLOR.at_shader_location(4),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}

impl TilesetMaterial for DefaultTilesetMaterial {
    fn init(settings: TilesetMaterialSettings) -> Self {
        Self {
            texture: settings.texture,
            alpha_mode: settings.alpha_mode,
        }
    }

    fn name() -> &'static str {
        "default"
    }
}

/// A component for entities that use a tileset, which stores the handle to the
/// tileset asset.
#[derive(Debug, Clone, Component, Reflect)]
pub struct UseTileset(pub Handle<Tileset>);

impl AsAssetId for UseTileset {
    type Asset = Tileset;

    fn as_asset_id(&self) -> AssetId<Self::Asset> {
        self.0.id()
    }
}

/// Applies tileset materials to the entities that are currently missing the
/// correct material.
pub(super) fn apply_tileset_material<M: TilesetMaterial>(
    mut type_id_cache: Local<Option<TypeId>>,
    tilesets: Res<Assets<Tileset>>,
    missing_material: Query<(Entity, &UseTileset), Without<MeshMaterial3d<M>>>,
    mut commands: Commands,
) {
    if type_id_cache.is_none() {
        *type_id_cache = Some(TypeId::of::<M>());
    }

    for (entity, use_tileset) in &missing_material {
        let Some(tileset) = tilesets.get(&use_tileset.0) else {
            // tileset still loading
            continue;
        };

        if tileset.material().type_id() != type_id_cache.unwrap() {
            // Wrong material type.
            continue;
        }

        let material_handle = tileset.material().clone().typed::<M>();
        commands
            .entity(entity)
            .insert(MeshMaterial3d(material_handle));
    }
}

pub(super) fn update_tileset_material<M: TilesetMaterial>(
    mut type_id_cache: Local<Option<TypeId>>,
    tilesets: Res<Assets<Tileset>>,
    mut query: Query<(Entity, &UseTileset), Changed<UseTileset>>,
    mut commands: Commands,
) {
    if type_id_cache.is_none() {
        *type_id_cache = Some(TypeId::of::<M>());
    }

    for (entity, use_tileset) in &mut query {
        let Some(tileset) = tilesets.get(&use_tileset.0) else {
            // tileset still loading
            continue;
        };

        if tileset.material().type_id() != type_id_cache.unwrap() {
            commands.entity(entity).remove::<MeshMaterial3d<M>>();
        } else {
            let material_handle = tileset.material().clone().typed::<M>();
            commands
                .entity(entity)
                .insert(MeshMaterial3d(material_handle));
        }
    }
}

pub(super) fn remove_tileset_material<M: TilesetMaterial>(
    mut commands: Commands,
    query: Query<Entity, (With<MeshMaterial3d<M>>, Without<UseTileset>)>,
) {
    for entity in query.iter() {
        commands.entity(entity).remove::<MeshMaterial3d<M>>();
    }
}
