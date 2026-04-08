use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext, LoadDirectError, RenderAssetUsages};
use bevy::image::{
    ImageAddressMode,
    ImageFilterMode,
    ImageFormatSetting,
    ImageLoaderSettings,
    ImageSampler,
};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDataOrder, TextureDimension, TextureFormat};

use crate::tileset::material::{Tileset, TilesetMaterial, TilesetMaterialSettings};
use crate::utils;
use crate::utils::asset::{ContextRelativePathEtx, RelativePathError};

/// The asset loader for tilesets, which loads tileset assets from `.tiles`
/// files.
#[derive(Debug, TypePath)]
pub struct TilesetLoader<M: TilesetMaterial> {
    _marker: std::marker::PhantomData<M>,
    ext: Vec<&'static str>,
}

impl<M: TilesetMaterial> TilesetLoader<M> {
    /// The property name for the display name of a tileset.
    pub const NAME_PROPERTY: &'static str = "__NAME";

    /// The property name for the alpha mode of a tileset.
    pub const ALPHA_MODE_PROPERTY: &'static str = "__ALPHA_MODE";
}

impl<M: TilesetMaterial> Default for TilesetLoader<M> {
    fn default() -> Self {
        Self {
            _marker: std::marker::PhantomData,
            ext: vec![M::file_extension()],
        }
    }
}

impl<M: TilesetMaterial> AssetLoader for TilesetLoader<M> {
    type Asset = Tileset<M>;
    type Settings = ();
    type Error = TilesetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        ctx: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut properties = utils::asset::parse_properties(reader).await?;

        let name = properties
            .remove(TilesetLoader::<M>::NAME_PROPERTY)
            .map(|v| v.to_string());

        let alpha_mode = match properties
            .remove(TilesetLoader::<M>::ALPHA_MODE_PROPERTY)
            .map(|v| v.to_string())
        {
            Some(ref v) if v == "opaque" => AlphaMode::Opaque,
            Some(ref v) if v == "cutout" => AlphaMode::Mask(0.5),
            Some(ref v) if v == "transparent" => AlphaMode::Blend,
            None => AlphaMode::Opaque,
            Some(v) => {
                return Err(TilesetLoaderError::InvalidBlendMode(v));
            }
        };

        if properties.len() < 2 {
            return Err(TilesetLoaderError::NotEnoughTiles(properties.len()));
        }

        let mut size: Option<u32> = None;
        let mut image_data: Vec<u8> = Vec::new();

        let mut tiles = Vec::new();
        for (tile_name, tile_path) in properties.drain() {
            tiles.push(tile_name);

            let path = ctx.get_relative_path(&tile_path)?;
            let loaded_tile = ctx
                .loader()
                .with_settings(image_settings)
                .immediate()
                .load(path)
                .await?;
            let tile_image: &Image = loaded_tile.get();

            validate_tile_size(tile_image, &mut size)?;
            generate_mip_maps(tile_image, &mut image_data)?;
        }

        let texture_array = image(image_data, size.unwrap(), tiles.len() as u32);
        let img_handle = ctx.add_labeled_asset("texture".to_owned(), texture_array);

        let material = M::init(TilesetMaterialSettings {
            texture: img_handle,
            alpha_mode,
        });

        let material_handle = ctx.add_labeled_asset("material".to_owned(), material);
        let tileset = Tileset::<M>::new(name, material_handle, tiles);

        info!(
            "Loaded {} from {} with {} tiles and alpha mode {:?}",
            tileset.name(),
            ctx.path(),
            tileset.tile_names().len(),
            alpha_mode
        );

        Ok(tileset)
    }

    fn extensions(&self) -> &[&str] {
        &self.ext
    }
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
fn validate_tile_size(image: &Image, expected_size: &mut Option<u32>) -> Result<(), TileError> {
    if image.width() != image.height() {
        return Err(TileError::NotSquare {
            width: image.width(),
            height: image.height(),
        });
    }

    if let Some(size) = expected_size {
        if image.width() != *size {
            return Err(TileError::MultipleSizes {
                expected: *size,
                found: image.width(),
            });
        }
    } else {
        *expected_size = Some(image.width());
    }

    Ok(())
}

fn generate_mip_maps(image: &Image, data: &mut Vec<u8>) -> Result<(), TileError> {
    let Some(src_data) = &image.data else {
        return Err(TileError::ImageHasNoData);
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

#[derive(Debug, thiserror::Error)]
pub enum TilesetLoaderError {
    /// Invalid tileset file.
    #[error("Invalid tileset file: {0}")]
    ParserError(#[from] utils::asset::PropertyParserError),

    /// Invalid blend mode.
    #[error("Invalid blend mode: {0}")]
    InvalidBlendMode(String),

    /// Image file not found.
    #[error("Image file not found: {0}")]
    ImageNotFound(#[from] RelativePathError),

    /// Errors from the Image loader.
    #[error("Image loading error: {0}")]
    ImageLoaderError(#[from] LoadDirectError),

    /// Invalid tile.
    #[error("Invalid tile: {0}")]
    InvalidTile(#[from] TileError),

    /// Not enough tiles to generate texture array.
    #[error("Not enough tiles to generate texture array: {0} (minimum is 2)")]
    NotEnoughTiles(usize),
}

/// Errors that can occur when loading a tile.
#[derive(Debug, thiserror::Error)]
pub enum TileError {
    #[error("Tile is not square: {width}x{height}")]
    NotSquare { width: u32, height: u32 },

    /// Tile has different size than previous tiles.
    #[error("Mismatching tile sizes: expected {expected}x{expected}, found {found}x{found}")]
    MultipleSizes { expected: u32, found: u32 },

    /// Tile image has no data.
    #[error("Image has no data")]
    ImageHasNoData,
}
