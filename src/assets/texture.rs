
use super::{image::ImageRon, Image};
use std::sync::Arc;

// Texture represents data on the GPU.
pub struct Texture {
    pub inner: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub extent: wgpu::Extent3d,
}

impl Texture {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>, image: Image, image_ron: Option<ImageRon>) -> Self {

        let extent = wgpu::Extent3d {
            width: image.width,
            height: image.height,
            depth: 1,
        };

        let format = if image_ron.is_some() {
            image_ron.unwrap().format.into()
        } else {
            // Default to Rgba8UnormSrgb
            wgpu::TextureFormat::Rgba8UnormSrgb
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
            label: None,
        });
        queue.write_texture(
            wgpu::TextureCopyView {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &image.data[..],
            wgpu::TextureDataLayout {
                offset: 0,
                bytes_per_row: (image.data.len() as f64 / extent.height as f64) as u32,
                rows_per_image: extent.height,
            },
            extent,
        );

        Texture {
            inner: texture,
            view: texture.create_default_view(),
            extent,
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::{assets::{image::ImageRon, Image}};
    use super::super::new_asset_manager::AssetManager;

    #[test]
    fn should_create_texture() {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Warn)
            .filter_module("harmony", log::LevelFilter::Info)
            .init();

        let mut asset_manager = AssetManager::new();
        asset_manager.register::<Image>();
        asset_manager.register::<ImageRon>();
        asset_manager.load::<Image, _>("./assets/core/white.png");

        
    }
}