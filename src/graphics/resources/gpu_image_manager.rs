
use std::sync::mpsc::{Receiver, Sender};
use std::{path::PathBuf, sync::Arc};
use crate::{ImageAssetManager, graphics::material::image::{Image}};
use assetmanage_rs::*;
use futures::stream::{FuturesUnordered,StreamExt};

pub(crate) struct GPUImageHandle {
    // TODO: Rename ImageBuilder?
    image: Arc<Image>,
    texture: Option<wgpu::Texture>,
    view: Option<wgpu::TextureView>,
    base_mip_layer: u32,
    sampler_hash: u32,
}
impl Asset<GPUImageLoader> for GPUImageHandle{
    type ManagerSupplement = ();
    type AssetSupplement = Arc<Image>; //asset unique data
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
        todo!()
    }
}

pub(crate) struct GPUImageLoader {
    to_load: Receiver<(usize, PathBuf)>,
    loaded: Vec<Sender<(PathBuf, <<Self as assetmanage_rs::Loader>::Source as Source>::Output)>>,
    image_asset_manager: ImageAssetManager,
}

//impl GPUImageLoader{
//    /// run the async load loop
//    #[allow(unused)]
//    pub async fn run(mut self) {
//        let mut loading = FuturesUnordered::new();
//        //let mut to_load = FuturesUnordered::new();
//        loop {
//            self.to_load.try_iter().for_each(|(id, p)| {
//                match self.imageassetmanager.get(p){
//                    Some(asset) => {                
//                        loading.push(async move {
//                        (id, p, <<Self as Loader>::Source as Source>::load(asset))
//                    })}
//                    None => {}
//                }
//            });
//
//            if let Some((manager_idx, path, Ok(bytes))) = loading.next().await {
//                if let Some(sender) = self.loaded.get_mut(manager_idx) {
//                    if sender.send((path, bytes)).is_err() {}
//                }
//            }
//        }
//    }
//}

impl assetmanage_rs::Loader for GPUImageLoader{
    type Source = GPUImageSource;
    type Supplement = ImageAssetManager;
    fn new(
        to_load: Receiver<(usize, PathBuf)>,
        loaded: Vec<Sender<(PathBuf, <Self::Source as Source>::Output)>>,
        image_asset_manager: Self::Supplement,
    ) -> Self {
        Self {to_load, loaded, image_asset_manager}
    }
}



#[cfg(test)]
mod tests{
    use super::GPUImageHandle;
    use crate::ImageAssetManager;
    use std::{path::PathBuf, sync::Arc};

    #[test]
    fn initial(){

        env_logger::init_from_env(
            env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "warn"),
        );

        async_std::task::block_on(async {
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
            let mut asset_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            asset_path.push("assets");
            let mut iam = ImageAssetManager::new(asset_path.clone(), arc_device, arc_queue);


            let mut builder = assetmanage_rs::Builder::new();
            let gpu_manager = builder.create_manager::<GPUImageHandle>(());
            let loader = builder.finish_loader(iam);


            let mut image_path = PathBuf::new();
            image_path.push("core");
            image_path.push("white.image.ron");
            
            //println!("{:?}",iam.get(&image_path).is_some());
        });


        
    }
}