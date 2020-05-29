
use std::sync::mpsc::{Receiver, Sender};
use std::{path::PathBuf, sync::Arc};
use crate::{ graphics::material::image::{Image, ImageData, ImageInfo}};
use assetmanage_rs::*;
use futures::stream::{FuturesUnordered,StreamExt};
use futures::task::Poll;
use async_std::future::poll_fn;
pub(crate) type ImageManager = Manager<GPUImageHandle, GPUImageSource>;


pub(crate) struct GPUImageHandle {
    //image: Arc<Image>,
    //texture: Option<wgpu::Texture>,
    //view: Option<wgpu::TextureView>,
    //base_mip_layer: u32,
    //sampler_hash: u32,
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

impl Source for GPUImageSource{
    type Input = Arc<Image>;
    type Output = GPUImageHandle;
    fn load(item: Self::Input) -> Result<Self::Output, Box<dyn std::error::Error>> {
        //Input is the Arc<Image>/Texture that will be loaded to the GPU
        //Output will be the GPUImageHandle that will be returned
        Ok(GPUImageHandle{})
    }
}

pub(crate) struct GPUImageLoader {
    to_load: Receiver<(usize, PathBuf, <Self as Loader>::TransferSupplement)>,
    loaded: Vec<Sender<(PathBuf, <<Self as Loader>::Source as Source>::Output)>>,
    image_asset_manager: Manager<ImageData, MemoryLoader>,
}

impl GPUImageLoader{
    /// run the async load loop
    #[allow(unused)]
    pub async fn run(mut self) {
        let mut gpu_loading = FuturesUnordered::new();
        let mut still_loading = Vec::new();

        let fut_generator = |id, path, image | async move {
            (id, path, <<Self as Loader>::Source as Source>::load(image))
        };
        //let mut to_load = FuturesUnordered::new();
        loop {
            self.to_load.try_iter()
            .collect::<Vec<(usize, PathBuf, <Self as Loader>::TransferSupplement)>>()
            .into_iter()
            .for_each(|(id, p,t)| {
                match self.image_asset_manager.status(&p){
                    Some(load_status) => match load_status{
                        LoadStatus::NotLoaded => self.image_asset_manager.load(&p, ()).unwrap(),
                        LoadStatus::Loading => still_loading.push((id, p)),
                        LoadStatus::Loaded => {
                            let image = self.image_asset_manager.get(&p).unwrap();
                            gpu_loading.push(fut_generator(id, p, image));
                        },
                    },
                    None => {
                        self.image_asset_manager.insert(&p,t);
                        still_loading.push((id, p))
                    }
                }
            });
            still_loading = still_loading.into_iter()
            .filter_map(|(id,p)| {
                match self.image_asset_manager.get(&p){
                    Some(image) =>{gpu_loading.push(fut_generator(id, p, image)); None},
                    None => Some((id, p)),
                }}).collect();

            if let Some((manager_idx, path, Ok(output))) = gpu_loading.next().await {
                if let Some(sender) = self.loaded.get_mut(manager_idx) {
                    if sender.send((path, output)).is_err() {}
                }
            }
        }
    }
}

impl assetmanage_rs::Loader for GPUImageLoader{
    type Source = GPUImageSource;
    type LoaderSupplement = Manager<ImageData, MemoryLoader>;
    type TransferSupplement = Arc<ImageInfo>;
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
        let image_file_manager = builder.create_manager::<ImageData>((device, queue));
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
        assert!(image_manager.load(&abs_image_path, Arc::new(ImageInfo::new(ImageFormat::SRGB))).is_ok());
        println!("{:?}",image_manager.status(&abs_image_path));
        let t = image_manager.get_blocking(&abs_image_path);
        assert!(t.is_some());
    }
}