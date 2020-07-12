
use super::{image::ImageRon, Image};
use std::{path::PathBuf, sync::Arc};

// Texture represents data on the GPU.
pub struct Texture {
    pub(crate) path: PathBuf,
    pub inner: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub extent: wgpu::Extent3d,
}

impl Texture {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>, image: Arc<Image>, image_ron: Option<Arc<ImageRon>>, path: PathBuf) -> Self {

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

#[cfg(test)]
mod tests {

    use crate::{assets::{image::ImageRon, Image}, graphics::resources::GPUResourceManager};
    use super::super::new_asset_manager::AssetManager;
    use std::sync::Arc;

    #[test]
    fn should_create_texture() {
        // env_logger::Builder::from_default_env()
        //     .filter_level(log::LevelFilter::Warn)
        //     .filter_module("harmony", log::LevelFilter::Info)
        //     .init();

        let (_, arc_device, arc_queue) = futures::executor::block_on(async {
            let (needed_features, unsafe_features) =
                (wgpu::Features::empty(), wgpu::UnsafeFeatures::disallow());

            let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
            let adapter = instance
                .request_adapter(
                    &wgpu::RequestAdapterOptions {
                        power_preference: wgpu::PowerPreference::Default,
                        compatible_surface: None,
                    },
                    unsafe_features,
                )
                .await
                .unwrap();

            let adapter_features = adapter.features();
            let (device, queue) = adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        features: adapter_features & needed_features,
                        limits: wgpu::Limits::default(),
                        shader_validation: true,
                    },
                    None,
                )
                .await
                .unwrap();
            let arc_device = Arc::new(device);
            let arc_queue = Arc::new(queue);
            
            (adapter, arc_device, arc_queue)
        });        

        let gpu_resource_manager = GPUResourceManager::new(&arc_device);

        let mut asset_manager = AssetManager::new(arc_device, arc_queue);
        asset_manager.register::<Image>();
        asset_manager.register::<ImageRon>();
        asset_manager.load::<Image, _>("./assets/core/white.png");
        dbg!("First maintain!");
        asset_manager.maintain(&gpu_resource_manager); 

        std::thread::sleep(std::time::Duration::from_secs(1));
        dbg!("Second maintain!");
        asset_manager.maintain(&gpu_resource_manager);

        let texture_status = asset_manager.get_texture("./assets/core/white.png");

        match texture_status {
            async_filemanager::LoadStatus::Loaded(_) => {
                dbg!("We have wgpu texture!");
            },
            _ => { panic!("Texture did not load successfully"); }
        }

        
    }
}