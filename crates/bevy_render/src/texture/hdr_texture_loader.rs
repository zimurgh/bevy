use crate::{
    render_asset::RenderAssetPersistencePolicy,
    texture::{Image, TextureFormatPixelInfo},
};
use bevy_asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use wgpu::{Extent3d, TextureDimension, TextureFormat};

/// Loads HDR textures as Texture assets
#[derive(Clone, Default)]
pub struct HdrTextureLoader;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct HdrTextureLoaderSettings {
    pub cpu_persistent_access: RenderAssetPersistencePolicy,
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum HdrTextureLoaderError {
    #[error("Could load texture: {0}")]
    Io(#[from] std::io::Error),
    #[error("Could not extract image: {0}")]
    Image(#[from] image::ImageError),
}

impl AssetLoader for HdrTextureLoader {
    type Asset = Image;
    type Settings = HdrTextureLoaderSettings;
    type Error = HdrTextureLoaderError;
    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        settings: &'a Self::Settings,
        _load_context: &'a mut LoadContext,
    ) -> bevy_utils::BoxedFuture<'a, Result<Image, Self::Error>> {
        Box::pin(async move {
            let format = TextureFormat::Rgba32Float;
            debug_assert_eq!(
                format.pixel_size(),
                4 * 4,
                "Format should have 32bit x 4 size"
            );

            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let decoder = image::codecs::hdr::HdrDecoder::new(bytes.as_slice())?;
            let info = decoder.metadata();
            let rgb_data = decoder.read_image_hdr()?;
            let mut rgba_data = Vec::with_capacity(rgb_data.len() * format.pixel_size());

            for rgb in rgb_data {
                let alpha = 1.0f32;

                rgba_data.extend_from_slice(&rgb.0[0].to_ne_bytes());
                rgba_data.extend_from_slice(&rgb.0[1].to_ne_bytes());
                rgba_data.extend_from_slice(&rgb.0[2].to_ne_bytes());
                rgba_data.extend_from_slice(&alpha.to_ne_bytes());
            }

            Ok(Image::new(
                Extent3d {
                    width: info.width,
                    height: info.height,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                rgba_data,
                format,
                settings.cpu_persistent_access,
            ))
        })
    }

    fn extensions(&self) -> &[&str] {
        &["hdr"]
    }
}
