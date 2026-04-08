use std::fmt;

use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext, LoadDirectError, RenderAssetUsages};
use bevy::gltf::GltfLoaderSettings;
use bevy::gltf::convert_coordinates::GltfConvertCoordinates;
use bevy::image::ImageSamplerDescriptor;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;

use crate::utils;
use crate::utils::asset::{ContextRelativePathEtx, RelativePathError};

/// The default name for an actor asset that does not have a name.
pub const DEFAULT_ACTOR_NAME: &str = "<Unnamed Actor Asset>";

/// The `Actor` asset represents a 3D model with a skeleton and animations that
/// can move and interact within the world.
#[derive(Debug, Asset, TypePath)]
pub struct Actor {
    /// The scene handle of this actor asset. This is the model that will be
    /// spawned when an actor instance is created from this asset.
    scene: Handle<Scene>,

    /// A mapping of animation names to their corresponding animation node
    /// indices in the animation graph.
    animations: HashMap<Box<str>, AnimationNodeIndex>,

    /// The generated animation graph for this actor asset, if it has any
    /// animations.
    anim_graph: Option<Handle<AnimationGraph>>,

    /// Additional properties of this actor asset, as defined in the asset
    /// metadata file. This can include any custom properties that are not
    /// explicitly handled by the asset loader.
    properties: HashMap<Box<str>, Box<str>>,
}

impl Actor {
    /// The key for the display name property in the actor asset metadata.
    pub const NAME_PROPERTY: &'static str = "name";

    /// The key for the GLTF model path property in the actor asset metadata.
    ///
    /// This is a relative asset path based on the location of the actor asset
    /// file. For example, if the actor asset file is located at
    /// `assets/actors/hero.actor`, and the model path is `models/hero.gltf`,
    /// the full path to the model would be `assets/actors/models/hero.gltf`.
    pub const MODEL_PROPERTY: &'static str = "model";

    /// The key for the default animation property in the actor asset metadata.
    ///
    /// This is the animation that is played by default when an actor is
    /// spawned, unless otherwise specified by an `ActorAnimation` component.
    pub const DEFAULT_ANIMATION_PROPERTY: &'static str = "default_animation";

    /// Gets the scene handle of this actor asset.
    pub fn scene(&self) -> &Handle<Scene> {
        &self.scene
    }

    /// Gets all animations of this actor asset.
    pub fn animations(&self) -> impl Iterator<Item = &str> {
        self.animations.keys().map(|k| k.as_ref())
    }

    /// Gets the animation graph handle of this actor asset, if it exists.
    pub fn anim_graph(&self) -> Option<&Handle<AnimationGraph>> {
        self.anim_graph.as_ref()
    }

    /// Gets the properties of this actor asset.
    pub fn properties(&self) -> &HashMap<Box<str>, Box<str>> {
        &self.properties
    }

    /// Gets a specific property of this actor asset by key.
    pub fn get_property(&self, key: &str) -> Option<&str> {
        self.properties.get(key).map(|value| value.as_ref())
    }

    /// Gets the animation node index of a specific animation by name.
    pub fn get_animation(&self, name: &str) -> Option<AnimationNodeIndex> {
        self.animations.get(name).copied()
    }

    /// Gets the display name of this actor asset.
    ///
    /// Defaults to "<Unnamed Actor Asset>" if the name property is not set in
    /// the asset metadata.
    pub fn name(&self) -> &str {
        self.get_property(Self::NAME_PROPERTY)
            .unwrap_or(DEFAULT_ACTOR_NAME)
    }

    /// Gets the model path of this actor asset, if specified in the asset
    /// metadata.
    pub fn model_path(&self) -> Option<&str> {
        self.get_property(Self::MODEL_PROPERTY)
    }

    /// Gets the default animation of this actor asset, if specified in the
    /// asset metadata.
    pub fn get_default_animation(&self) -> Option<(&str, AnimationNodeIndex)> {
        let default_animation_name = self.get_property(Self::DEFAULT_ANIMATION_PROPERTY)?;
        let index = self.get_animation(default_animation_name)?;
        Some((default_animation_name, index))
    }
}

impl fmt::Display for Actor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.name().fmt(f)
    }
}

#[derive(Debug, Default, TypePath)]
pub struct ActorAssetLoader;
impl AssetLoader for ActorAssetLoader {
    type Asset = Actor;
    type Settings = ();
    type Error = ActorAssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _: &Self::Settings,
        ctx: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let properties = utils::asset::parse_properties(reader).await?;
        debug!("Parsed properties: {properties:#?}");

        if !properties.contains_key(Actor::NAME_PROPERTY) {
            warn!("Actor asset at {:?} is missing a name property", ctx.path());
        }

        let model_path_value = properties
            .get(Actor::MODEL_PROPERTY)
            .ok_or_else(|| ActorAssetLoaderError::MissingProperty(Actor::MODEL_PROPERTY))?;
        let model_path = ctx.get_relative_path(model_path_value)?;

        let gltf_asset = ctx
            .loader()
            .with_settings(glfw_settings)
            .immediate()
            .load(model_path)
            .await?;
        let gltf: &Gltf = gltf_asset.get();

        if gltf.scenes.is_empty() {
            return Err(ActorAssetLoaderError::NoScene);
        }

        let scene = gltf.default_scene.clone().unwrap_or(gltf.scenes[0].clone());

        let mut animations = HashMap::new();
        let mut anim_graph = None;
        if !gltf.animations.is_empty() {
            if gltf.animations.len() != gltf.named_animations.len() {
                return Err(ActorAssetLoaderError::UnnamedAnimations);
            }

            let mut anim_names = vec![];
            let mut anim_clips = vec![];
            for (name, anim) in gltf.named_animations.iter() {
                anim_names.push(name);
                anim_clips.push(anim.clone());
            }

            let default_anim_name = properties
                .get(Actor::DEFAULT_ANIMATION_PROPERTY)
                .ok_or(ActorAssetLoaderError::NoDefaultAnimation)?;

            if !anim_names
                .iter()
                .any(|name| name.as_ref() == default_anim_name.as_ref())
            {
                return Err(ActorAssetLoaderError::AnimationNotFound(
                    default_anim_name.to_string(),
                ));
            }

            let (generated_anim_graph, anim_indices) = AnimationGraph::from_clips(anim_clips);
            anim_graph = Some(ctx.add_labeled_asset("anim_graph".to_owned(), generated_anim_graph));

            for (name, index) in anim_names.into_iter().zip(anim_indices.into_iter()) {
                animations.insert(name.clone(), index);
            }
        }

        ctx.add_loaded_labeled_asset("model", gltf_asset);

        let name = properties
            .get(Actor::NAME_PROPERTY)
            .map(|v| v.to_string())
            .unwrap_or_else(|| DEFAULT_ACTOR_NAME.to_string());

        info!("Loaded Actor {} from {:?}", name, ctx.path());

        Ok(Actor {
            scene,
            animations,
            anim_graph,
            properties,
        })
    }

    fn extensions(&self) -> &[&str] {
        &["actor"]
    }
}

fn glfw_settings(settings: &mut GltfLoaderSettings) {
    settings.load_meshes = RenderAssetUsages::RENDER_WORLD;
    settings.load_materials = RenderAssetUsages::RENDER_WORLD;
    settings.load_cameras = false;
    settings.load_lights = true;
    settings.load_animations = true;
    settings.include_source = false;
    settings.default_sampler = Some(ImageSamplerDescriptor::nearest());
    settings.override_sampler = true;
    settings.convert_coordinates = Some(GltfConvertCoordinates {
        rotate_scene_entity: true,
        rotate_meshes: false,
    });
}

/// Errors that can occur during the loading of an `ActorAsset`.
#[derive(Debug, thiserror::Error)]
pub enum ActorAssetLoaderError {
    /// Invalid actor file.
    #[error("Invalid actor file: {0}")]
    ParserError(#[from] utils::asset::PropertyParserError),

    /// Required property missing from actor file.
    #[error("Required property missing from actor file: {0}")]
    MissingProperty(&'static str),

    /// GLTF file not found.
    #[error("GLTF file not found: {0}")]
    GltfNotFound(#[from] RelativePathError),

    /// Errors from the GLTF loader.
    #[error("Model loading error: {0}")]
    GltfLoaderError(#[from] LoadDirectError),

    /// No scene found in the GLTF file.
    #[error("No scene found in GLTF file")]
    NoScene,

    /// No default animation specified.
    #[error("No default animation specified in metadata file")]
    NoDefaultAnimation,

    /// Default animation is specified but not found in the GLTF file.
    #[error("No '{0}' animation found in GLTF file")]
    AnimationNotFound(String),

    /// GLTF file has animations but some are unnamed.
    #[error("GLTF file has unnamed animations")]
    UnnamedAnimations,
}
