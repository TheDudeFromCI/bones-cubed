use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::pbr::{MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup,
    RenderPipelineDescriptor,
    SpecializedMeshPipelineError,
};
use bevy::shader::ShaderRef;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bones_cubed::BonesCubedPlugin;
use bones_cubed::mesh::{TerrainMesh, TerrainQuad, TerrainVertex};
use bones_cubed::tileset::RegisterTilesetMaterialExt;
use bones_cubed::tileset::material::{
    ATTRIBUTE_UV_LAYER,
    DefaultTilesetMaterial,
    Tileset,
    TilesetMaterial,
    TilesetMaterialSettings,
    UseTileset,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BonesCubedPlugin)
        .add_plugins(PanOrbitCameraPlugin)
        .register_tileset_material::<RainbowMaterial>()
        .add_systems(Startup, setup)
        .add_systems(Update, update_time)
        .run();
}

/// Our custom tileset material.
#[derive(Debug, Default, Clone, Asset, TypePath, AsBindGroup)]
struct RainbowMaterial {
    /// The tileset texture image of the material.
    #[texture(0, dimension = "2d_array")]
    #[sampler(1)]
    texture: Handle<Image>,

    /// The time uniform for the shader.
    #[uniform(2)]
    time: f32,
}

impl TilesetMaterial for RainbowMaterial {
    fn init(settings: TilesetMaterialSettings) -> Self {
        info!(
            "Initializing RainbowMaterial with texture: {:?}",
            settings.texture
        );
        Self {
            texture: settings.texture,
            time: 0.0,
        }
    }

    fn file_extension() -> &'static str {
        "tiles.rainbow"
    }
}

impl Material for RainbowMaterial {
    // Our custom shader asset:

    fn vertex_shader() -> ShaderRef {
        "shaders/rainbow.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/rainbow.wgsl".into()
    }

    // Because tilesets use additional vertex attributes, the prepass shaders
    // need to be overriden as well.

    // You can copy and paste these methods for most custom tileset materials,
    // unless you modify the vertex positions or normals in the shader.

    fn prepass_vertex_shader() -> ShaderRef {
        DefaultTilesetMaterial::prepass_vertex_shader()
    }

    fn prepass_fragment_shader() -> ShaderRef {
        DefaultTilesetMaterial::prepass_fragment_shader()
    }

    // The default Bevy material implementation of this method will not work for
    // BonesCubed, as it doesn't include the necessary vertex attributes.

    // You can copy and paste this method for most custom tileset materials,
    // unless you need to need to write your own terrain generation logic that
    // requires additional vertex attributes.

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

fn setup(asset_server: Res<AssetServer>, mut meshes: ResMut<Assets<Mesh>>, mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        PanOrbitCamera::default(),
    ));

    let tileset_handle: Handle<Tileset<RainbowMaterial>> =
        asset_server.load("tilesets/rainbow/rainbow.tiles.rainbow");

    let mut terrain = TerrainMesh::new();
    terrain.add_quad(TerrainQuad(
        TerrainVertex {
            position: Vec3::new(-0.5, 0.0, -0.5),
            uv: Vec2::new(0.0, 0.0),
            layer: 0,
            normal: Vec3::Y,
            color: Color::srgb(1.0, 1.0, 1.0),
        },
        TerrainVertex {
            position: Vec3::new(-0.5, 0.0, 0.5),
            uv: Vec2::new(0.0, 1.0),
            layer: 0,
            normal: Vec3::Y,
            color: Color::srgb(1.0, 1.0, 1.0),
        },
        TerrainVertex {
            position: Vec3::new(0.5, 0.0, 0.5),
            uv: Vec2::new(1.0, 1.0),
            layer: 0,
            normal: Vec3::Y,
            color: Color::srgb(1.0, 1.0, 1.0),
        },
        TerrainVertex {
            position: Vec3::new(0.5, 0.0, -0.5),
            uv: Vec2::new(1.0, 0.0),
            layer: 0,
            normal: Vec3::Y,
            color: Color::srgb(1.0, 1.0, 1.0),
        },
    ));

    commands.spawn((
        Transform::default(),
        Mesh3d(meshes.add(terrain)),
        UseTileset(tileset_handle),
    ));
}

fn update_time(time: Res<Time>, mut materials: ResMut<Assets<RainbowMaterial>>) {
    let seconds = time.elapsed_secs();
    for (_, material) in materials.iter_mut() {
        material.time = seconds;
    }
}
