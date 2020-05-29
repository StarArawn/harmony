
use std::sync::mpsc::{Receiver, Sender};
use std::{path::PathBuf, sync::Arc};
use crate::{ graphics::material::image::{Image, ImageData, ImageInfo}};
use assetmanage_rs::*;
use futures::stream::{FuturesUnordered,StreamExt};
pub(crate) type ImageManager = Manager<GPUImageHandle, GPUImageSource>;


pub(crate) struct GPUImageHandle {
    pub(crate) image: Arc<Image>,
    pub(crate) texture: wgpu::Texture,
    pub(crate) view: wgpu::TextureView,
    pub(crate) sampler: wgpu::Sampler, // TODO: remove sampler and put it someplace else?
    pub(crate) base_mip_layer: u32,
    pub(crate) sampler_hash: u32,
}
impl Asset<GPUImageLoader> for GPUImageHandle{
    type ManagerSupplement = ();
    type AssetSupplement = (); //asset unique data
    type Structure = GPUImageHandle;
    fn construct(
        data_load: GPUImageHandle,
        _data_ass: &Self::AssetSupplement,
        _data_mgr: &Self::ManagerSupplement,
    ) -> Result<Self::Structure, std::io::Error> {
        Ok(data_load)
    }
}

pub(crate) struct GPUImageSource;

impl Source for GPUImageSource {
    type Input = (Arc<Image>, Arc<wgpu::Device>, Arc<wgpu::Queue>);
    type Output = GPUImageHandle;
    fn load((image, device, queue): Self::Input) -> Result<Self::Output, Box<dyn std::error::Error>> {
        let (texture, view, sampler) = image.create_gpu_texture(device, queue);
        
        let handle = GPUImageHandle {
            image,
            texture,
            view,
            sampler,
            base_mip_layer: 0,
            sampler_hash: 0, // TODO: Use this instead of sampler.
        };

        Ok(handle)
    }
}

pub(crate) struct GPUImageLoader {
    to_load: Receiver<(usize, PathBuf, <Self as Loader>::TransferSupplement)>,
    loaded: Vec<Sender<(PathBuf, <<Self as Loader>::Source as Source>::Output)>>,
    image_asset_manager: Manager<ImageData, MemoryLoader>,
}

impl GPUImageLoader {
    /// run the async load loop
    #[allow(unused)]
    pub async fn run(mut self) {
        let mut gpu_loading = FuturesUnordered::new();
        let mut still_loading = Vec::new();

        let fut_generator = |id, path, image, device, queue| async move {
            (id, path, <<Self as Loader>::Source as Source>::load((image, device, queue)))
        };

        //let mut to_load = FuturesUnordered::new();
        loop {
            self.to_load.try_iter()
            .collect::<Vec<(usize, PathBuf, <Self as Loader>::TransferSupplement)>>()
            .into_iter()
            .for_each(|(id, p,(t_imageinfo, t_device, t_queue))| {
                match self.image_asset_manager.status(&p){
                    Some(load_status) => match load_status{
                        LoadStatus::NotLoaded => {
                            self.image_asset_manager.load(&p, ()).unwrap();
                            still_loading.push((id, p, t_device, t_queue));
                        },
                        LoadStatus::Loading => still_loading.push((id, p, t_device, t_queue)),
                        LoadStatus::Loaded => {
                            let image = self.image_asset_manager.get(&p).unwrap();
                            gpu_loading.push(fut_generator(id, p, image, t_device, t_queue ));
                        },
                    },
                    None => {
                        self.image_asset_manager.insert(&p, t_imageinfo);
                        self.image_asset_manager.load(&p, ()).unwrap();
                        still_loading.push((id, p, t_device, t_queue))
                    }
                }
            });
            still_loading = still_loading.into_iter()
            .filter_map(|(id, p, t_device, t_queue)| {
                match self.image_asset_manager.get(&p){
                    Some(image) => { 
                        log::warn!("{:?}, went to GPULoader", &p);
                        gpu_loading.push(fut_generator(id, p, image, t_device, t_queue)); 
                        None 
                    },
                    None => Some((id, p, t_device, t_queue)),
                }}).collect();
            

            if let Some((manager_idx, path, Ok(output))) = gpu_loading.next().await {
                if let Some(sender) = self.loaded.get_mut(manager_idx) {
                    if sender.send((path, output)).is_err() {
                        log::warn!("Could not send")
                    }
                }
            }
            self.image_asset_manager.maintain();
        }
    }
}

impl assetmanage_rs::Loader for GPUImageLoader{
    type Source = GPUImageSource;
    type LoaderSupplement = Manager<ImageData, MemoryLoader>;
    type TransferSupplement = (Arc<ImageInfo>,Arc<wgpu::Device>, Arc<wgpu::Queue>);
    fn new(
        to_load: Receiver<(usize, PathBuf, Self::TransferSupplement)>,
        loaded: Vec<Sender<(PathBuf, <Self::Source as Source>::Output)>>,
        image_asset_manager: Self::LoaderSupplement,
    ) -> Self {
        Self {to_load, loaded, image_asset_manager}
    }
}

#[cfg(test)]
mod tests{
    use super::{GPUImageHandle, ImageInfo};
    use std::{path::PathBuf, sync::Arc};
    use crate::graphics::material::image::{ImageFormat, ImageData};

    #[test]
    fn initial(){
        env_logger::init_from_env(
            env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "warn"),
        );

        let (device, queue) = async_std::task::block_on(async {
            let instance = wgpu::Instance::new(); 
            let adapter = instance
                .request_adapter(
                    &wgpu::RequestAdapterOptions {
                        power_preference: wgpu::PowerPreference::Default,
                        compatible_surface: None,
                    },
                    wgpu::BackendBit::PRIMARY,
                )
                .await
                .unwrap();

            let (device, queue) = adapter
                .request_device(&wgpu::DeviceDescriptor {
                    extensions: wgpu::Extensions {
                        anisotropic_filtering: false,
                    },
                    limits: wgpu::Limits::default(),
                }, None)
                .await
                .unwrap();
            let arc_device = Arc::new(device);
            let arc_queue = Arc::new(queue);

            (arc_device, arc_queue)
        });

        let mut asset_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        asset_path.push("assets");

        let mut builder = assetmanage_rs::Builder::new();
        let image_file_manager = builder.create_manager::<ImageData>(());
        let file_loader = builder.finish_loader(());
        async_std::task::spawn(file_loader.run());

        let mut builder = assetmanage_rs::Builder::new();
        let mut image_manager = builder.create_manager::<GPUImageHandle>(());
        let gpu_loader = builder.finish_loader(image_file_manager);
        async_std::task::spawn(gpu_loader.run());


        let mut rel_image_path = PathBuf::new();
        rel_image_path.push("core");
        rel_image_path.push("white.png");
        let abs_image_path = asset_path.join(&rel_image_path);
        image_manager.insert(&abs_image_path,());
        println!("{:?}",image_manager.status(&abs_image_path));
        assert!(image_manager.load(&abs_image_path, (Arc::new(ImageInfo::new(ImageFormat::SRGB)), device, queue)).is_ok());
        println!("{:?}",image_manager.status(&abs_image_path));
        let t = image_manager.get_blocking(&abs_image_path);
        assert!(t.is_some());
    }
}