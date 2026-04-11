use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext, LoadDirectError, RenderAssetUsages};
use bevy::image::{
    ImageAddressMode,
    ImageFilterMode,
    ImageFormatSetting,
    ImageLoaderSettings,
    ImageSampler,
};
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDataOrder, TextureDimension, TextureFormat};
use ron::extensions::Extensions;
use serde::{Deserialize, Serialize};

use crate::tileset::filelayout::TilesetFileLayout;
use crate::tileset::material::{DefaultTilesetMaterial, Tileset, TilesetMaterialSettings};
use crate::utils::asset::{ContextRelativePathEtx, RelativePathError};

pub(crate) type MaterialInitializer =
    Box<dyn Fn(TilesetMaterialSettings, &mut LoadContext<'_>) -> UntypedHandle + Send + Sync>;

/// The asset loader for tilesets, which loads tileset assets from `.tiles`
/// files.
#[derive(Default, TypePath)]
pub struct TilesetLoader {
    pub(super) materials: HashMap<Box<str>, MaterialInitializer>,
}

impl TilesetLoader {
    /// Gets the material for the tileset, using the configured material type if
    /// it is set, or the default material if it is not set.
    fn get(
        &self,
        name: &str,
        ctx: &mut LoadContext<'_>,
        settings: TilesetMaterialSettings,
    ) -> Option<UntypedHandle> {
        let constructor = self.materials.get(name)?;
        Some(constructor(settings, ctx))
    }
}

impl AssetLoader for TilesetLoader {
    type Asset = Tileset;
    type Settings = TilesetLoaderSettings;
    type Error = TilesetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        settings: &Self::Settings,
        ctx: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = vec![];
        reader.read_to_end(&mut bytes).await?;

        let layout: TilesetFileLayout = ron::Options::default()
            .with_default_extension(Extensions::UNWRAP_NEWTYPES)
            .with_default_extension(Extensions::IMPLICIT_SOME)
            .with_default_extension(Extensions::UNWRAP_VARIANT_NEWTYPES)
            .from_bytes(&bytes)?;

        let material_name = layout.material.unwrap_or("default".to_string());

        validate_size(layout.size)?;
        validate_tile_count(layout.tiles.len())?;

        if settings.names_only {
            let tiles = layout
                .tiles
                .into_iter()
                .map(|tile| tile.name.into())
                .collect::<Vec<Box<str>>>();

            let material = Handle::<DefaultTilesetMaterial>::default().untyped();
            let tileset = Tileset::new_untyped(layout.name, tiles, material, material_name.into());

            debug!(
                "Partially loaded {} from {} with {} tiles)",
                tileset.name(),
                ctx.path(),
                tileset.tile_names().len(),
            );

            return Ok(tileset);
        }

        let mut image_data: Vec<u8> = Vec::new();

        let mut tiles = Vec::new();
        for tile in layout.tiles {
            let path = ctx.get_relative_path(&tile.texture)?;
            let loaded_tile = ctx
                .loader()
                .with_settings(image_settings)
                .immediate()
                .load(path)
                .await?;
            let tile_image: Image = loaded_tile.take();

            validate_tile_size(&tile.name, &tile_image, layout.size)?;
            generate_mip_maps(&tile.name, &tile_image, &mut image_data)?;
            tiles.push(tile.name.into());
        }

        if tiles.len() == 1 {
            // Bevy texture arrays require at least 2 layers, so if there is
            // only 1 tile, we duplicate it to create a 2-layer texture array.
            let image_data_clone = image_data.clone();
            image_data.extend_from_slice(image_data_clone.as_slice());
        }

        let layer_count = (tiles.len() as u32).max(2);
        let texture_array = image(image_data, layout.size, layer_count);
        let img_handle = ctx.add_labeled_asset("texture".to_owned(), texture_array);

        let Some(material) = self.get(
            &material_name,
            ctx,
            TilesetMaterialSettings {
                texture: img_handle,
                alpha_mode: layout.alpha_mode.into(),
            },
        ) else {
            return Err(TilesetLoaderError::UnknownMaterial(material_name));
        };

        let tileset = Tileset::new_untyped(layout.name, tiles, material, material_name.into());

        info!(
            "Loaded {} from {} with {} tiles and alpha mode {:?} (material: {})",
            tileset.name(),
            ctx.path(),
            tileset.tile_names().len(),
            layout.alpha_mode,
            tileset.material_name(),
        );

        Ok(tileset)
    }

    fn extensions(&self) -> &[&str] {
        &["tiles"]
    }
}

/// Validates that the given tile size is valid for a tileset. A valid tile size
/// must be a power of two between 1 and 1024, inclusive. Returns an error if
/// the tile size is invalid.
fn validate_size(size: u32) -> Result<(), TilesetLoaderError> {
    if size < 1 {
        return Err(TilesetLoaderError::TileSizeTooSmall(size));
    }

    if size > 1024 {
        return Err(TilesetLoaderError::TileSizeTooLarge(size));
    }

    if !size.is_power_of_two() {
        return Err(TilesetLoaderError::SizeNotPowerOfTwo(size));
    }

    Ok(())
}

/// Validates that the given tile count is valid for a tileset. A valid tile
/// count must be between 1 and 65536, inclusive. Returns an error if the tile
/// count is invalid.
fn validate_tile_count(count: usize) -> Result<(), TilesetLoaderError> {
    if count == 0 {
        return Err(TilesetLoaderError::EmptyTileset);
    }

    if count > u16::MAX as usize {
        return Err(TilesetLoaderError::TooManyTiles(count));
    }

    Ok(())
}

/// Configures the image loader settings for loading tileset textures.
fn image_settings(settings: &mut ImageLoaderSettings) {
    settings.format = ImageFormatSetting::FromExtension;
    settings.asset_usage = RenderAssetUsages::MAIN_WORLD;
    settings.texture_format = Some(TextureFormat::Rgba8UnormSrgb);
}

/// Creates an image from the given data, size, and number of tiles. The image
/// is a texture array with the given size and number of layers, and the data is
/// arranged in layer-major order. The image is configured with mipmaps and
/// nearest filtering.
fn image(data: Vec<u8>, size: u32, tiles: u32) -> Image {
    let mipmaps = mipmaps(size);

    let mut texture_array = Image {
        data: Some(data),
        data_order: TextureDataOrder::LayerMajor,
        asset_usage: RenderAssetUsages::RENDER_WORLD,
        ..default()
    };
    texture_array.asset_usage = RenderAssetUsages::RENDER_WORLD;
    texture_array.texture_descriptor.mip_level_count = mipmaps;
    texture_array.texture_descriptor.dimension = TextureDimension::D2;
    texture_array.texture_descriptor.format = TextureFormat::Rgba8UnormSrgb;
    texture_array.sampler = ImageSampler::nearest();
    texture_array.texture_descriptor.size = Extent3d {
        width: size,
        height: size,
        depth_or_array_layers: tiles,
    };

    let sampler_descriptor = texture_array.sampler.get_or_init_descriptor();
    sampler_descriptor.address_mode_u = ImageAddressMode::Repeat;
    sampler_descriptor.address_mode_v = ImageAddressMode::Repeat;
    sampler_descriptor.lod_max_clamp = mipmaps as f32 - 1.0;
    sampler_descriptor.mipmap_filter = ImageFilterMode::Linear;

    texture_array
}

/// Gets the total number of mipmap levels for a texture of the given size,
/// including the original size.
///
/// Minimum mipmap size is 4x4.
fn mipmaps(size: u32) -> u32 {
    let log = (size as f32).log2() - 1.0;
    let floor = log.floor() as u32;
    floor.max(1)
}

/// Validates that the size of the image is square and matches the expected
/// size, if it is set. If the expected size is not set, it will be set to the
/// size of the image. Returns an error if the image is not square or if the
/// size does not match the expected size.
fn validate_tile_size(
    tile_name: &str,
    image: &Image,
    expected: u32,
) -> Result<(), TilesetLoaderError> {
    if image.width() != expected || image.height() != expected {
        return Err(TilesetLoaderError::WrongSize {
            name: tile_name.to_owned(),
            width: image.width(),
            height: image.height(),
            expected,
        });
    }

    Ok(())
}

/// Generates mipmaps for the given image and appends them to the given data
/// vector. The image data is expected to be in RGBA8 format. The resulting
/// mipmaps, including the original size, are appended to the data vector in
/// layer-major order. Returns an error if the image has no data.
fn generate_mip_maps(
    tile_name: &str,
    image: &Image,
    data: &mut Vec<u8>,
) -> Result<(), TilesetLoaderError> {
    let Some(src_data) = &image.data else {
        return Err(TilesetLoaderError::ImageHasNoData(tile_name.to_owned()));
    };

    let mut size = image.width() as usize;
    let mipmaps = mipmaps(size as u32);

    let mut to_reserve = 0;
    for i in 0 .. mipmaps {
        to_reserve += src_data.len() / 4usize.pow(i);
    }
    data.reserve(to_reserve);

    let mut offset = data.len();
    data.extend_from_slice(src_data.as_slice());

    for _ in 1 .. mipmaps {
        size /= 2;
        for y in 0 .. size {
            for x in 0 .. size {
                let mut r = 0;
                let mut g = 0;
                let mut b = 0;
                let mut a = 0;

                for j in 0 .. 2 {
                    for i in 0 .. 2 {
                        let index = ((y * 2 + j) * size * 2 + x * 2 + i) as usize * 4;
                        r += data[offset + index] as u16;
                        g += data[offset + index + 1] as u16;
                        b += data[offset + index + 2] as u16;
                        a += data[offset + index + 3] as u16;
                    }
                }

                r /= 4;
                g /= 4;
                b /= 4;
                a /= 4;

                data.push(r as u8);
                data.push(g as u8);
                data.push(b as u8);
                data.push(a as u8);
            }
        }

        offset += size * size * 4;
    }

    Ok(())
}

/// Errors that can occur when loading a tileset.
#[derive(Debug, thiserror::Error)]
pub enum TilesetLoaderError {
    /// An error occurred while reading the block file.
    #[error("Failed to read block file: {0}")]
    Io(#[from] std::io::Error),

    /// An error occurred while parsing the block file.
    #[error("Failed to parse block file: {0}")]
    ParsingError(#[from] ron::de::SpannedError),

    /// The tile image is the wrong size.
    #[error(
        "Tile '{name}' is an invalid size: found {width}x{height}, expected {expected}x{expected}"
    )]
    WrongSize {
        name: String,
        width: u32,
        height: u32,
        expected: u32,
    },

    /// Tile image has no data.
    #[error("Tile '{0}' has no image data")]
    ImageHasNoData(String),

    /// Image file not found.
    #[error("Image file not found: {0}")]
    ImageNotFound(#[from] RelativePathError),

    /// Errors from the Image loader.
    #[error("Image loading error: {0}")]
    ImageLoaderError(#[from] LoadDirectError),

    /// The tileset contains no tiles.
    #[error("Tileset contains no tiles")]
    EmptyTileset,

    /// The tileset contains too many tiles.
    #[error("Tileset contains too many tiles: {0} (maximum is 65536)")]
    TooManyTiles(usize),

    /// The tile size is smaller than the minimum size.
    #[error("Tile size must be at least 1, found {0}")]
    TileSizeTooSmall(u32),

    /// The tile size is larger than the maximum size.
    #[error("Tile size must be at most 1024, found {0}")]
    TileSizeTooLarge(u32),

    /// The tile size is not a power of two or is larger than the maximum size.
    #[error("Tile size must be a power of two, found {0}")]
    SizeNotPowerOfTwo(u32),

    /// The tileset specifies a material that was not registered.
    #[error("Tileset specifies unknown material '{0}'")]
    UnknownMaterial(String),
}

/// Settings for the tileset loader, which can be used to configure how tilesets
/// are loaded.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TilesetLoaderSettings {
    /// Whether to only load the names of the tiles in the tileset, without
    /// loading the texture or material. This can be used for validation or for
    /// tools that need to know the tile names without loading the full tileset.
    pub names_only: bool,
}
