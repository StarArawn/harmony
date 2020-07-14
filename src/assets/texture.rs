
use super::{image::ImageRon, Image};
use std::{path::PathBuf, sync::Arc};

// Texture represents data on the GPU.
pub struct Texture {
    pub(crate) path: PathBuf,
    pub inner: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub extent: wgpu::Extent3d,
}

impl std::fmt::Debug for Texture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Texture")
         .field("path", &self.path)
         .field("extent", &self.extent)
         .finish()
    }
}

impl Texture {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>, image: Arc<Image>, image_ron: Option<ImageRon>, path: PathBuf) -> Self {

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

        let view = texture.create_default_view();

        Texture {
            path,
            inner: texture,
            view,
            extent,
        }
    }
}
