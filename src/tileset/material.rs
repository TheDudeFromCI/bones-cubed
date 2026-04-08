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

/// The default name for a tileset that does not have a name.
pub const DEFAULT_TILESET_NAME: &str = "<Unnamed Tileset>";

/// The default shader path for tilesets.
pub const DEFAULT_SHADER_PATH: &str = "embedded://bones_cubed/tileset/shader.wgsl";

/// The default prepass shader path for tilesets.
pub const DEFAULT_PREPASS_SHADER_PATH: &str = "embedded://bones_cubed/tileset/prepass.wgsl";

/// The tileset asset, which contains the material and blend mode for a tileset.
#[derive(Default, Clone, Asset, TypePath)]
pub struct Tileset<S: TilesetMaterial = DefaultTilesetMaterial> {
    /// The name of this tileset.
    name: Option<String>,

    /// The list of tile names defined in this tileset.
    tile_names: Vec<Box<str>>,

    /// The material used to render this tileset.
    material: Handle<S>,
}

impl<S: TilesetMaterial> Tileset<S> {
    /// Create a new tileset with the given name, tile names, and material.
    pub fn new(name: Option<String>, material: Handle<S>, tile_names: Vec<Box<str>>) -> Self {
        Self {
            name,
            tile_names,
            material,
        }
    }

    /// Get the display name of the tileset, or a default name if it is not set.
    pub fn name(&self) -> &str {
        self.name.as_deref().unwrap_or(DEFAULT_TILESET_NAME)
    }

    /// Get the names of the tiles in the tileset.
    pub fn tile_names(&self) -> &[Box<str>] {
        &self.tile_names
    }

    /// Get the index of a tile by its name, or `None` if it does not exist.
    ///
    /// Note that this method is O(n) in the number of tiles, so it is not
    /// recommended to use it frequently. Indices never change after loading,
    /// so it is recommended to cache the index of a tile.
    pub fn tile_index(&self, name: &str) -> Option<usize> {
        self.tile_names.iter().position(|n| n.as_ref() == name)
    }

    /// Get the material of the tileset.
    pub fn material(&self) -> &Handle<S> {
        &self.material
    }
}

impl<S: TilesetMaterial> fmt::Display for Tileset<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(name) = &self.name {
            name.fmt(f)
        } else {
            DEFAULT_TILESET_NAME.fmt(f)
        }
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

    /// Get the file extension for this type of tileset material. Each material
    /// must have a unique file extension to avoid conflicts when loading
    /// tilesets.
    ///
    /// Usually, this should start with `tiles`. For example: `tiles.water`
    /// All tileset files `*.tiles.water` will be loaded with this material.
    ///
    /// `.tiles` is reserved for the default tileset material, so custom
    /// materials should not use it as a suffix to avoid conflicts.
    fn file_extension() -> &'static str;
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

    fn file_extension() -> &'static str {
        "tiles"
    }
}

/// A component for entities that use a tileset, which stores the handle to the
/// tileset asset.
#[derive(Debug, Clone, Component, Reflect)]
pub struct UseTileset<M: TilesetMaterial>(pub Handle<Tileset<M>>);

impl<M: TilesetMaterial> AsAssetId for UseTileset<M> {
    type Asset = Tileset<M>;

    fn as_asset_id(&self) -> AssetId<Self::Asset> {
        self.0.id()
    }
}

pub(super) fn apply_tileset_material<M: TilesetMaterial>(
    tilesets: Res<Assets<Tileset<M>>>,
    missing_material: Query<(Entity, &UseTileset<M>), Without<MeshMaterial3d<M>>>,
    mut commands: Commands,
) {
    for (entity, use_tileset) in &missing_material {
        let Some(tileset) = tilesets.get(&use_tileset.0) else {
            // tileset still loading
            continue;
        };

        commands
            .entity(entity)
            .insert(MeshMaterial3d(tileset.material().clone()));
    }
}

pub(super) fn update_tileset_material<M: TilesetMaterial>(
    tilesets: Res<Assets<Tileset<M>>>,
    mut query: Query<(&UseTileset<M>, &mut MeshMaterial3d<M>), Changed<UseTileset<M>>>,
) {
    for (use_tileset, mut material) in &mut query {
        let Some(tileset) = tilesets.get(&use_tileset.0) else {
            // tileset still loading
            continue;
        };

        *material = MeshMaterial3d(tileset.material().clone());
    }
}

pub(super) fn remove_tileset_material<M: TilesetMaterial>(
    mut commands: Commands,
    query: Query<Entity, (With<MeshMaterial3d<M>>, Without<UseTileset<M>>)>,
) {
    for entity in query.iter() {
        commands.entity(entity).remove::<MeshMaterial3d<M>>();
    }
}
