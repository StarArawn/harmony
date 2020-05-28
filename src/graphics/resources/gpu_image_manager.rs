
use std::sync::mpsc::{Receiver, Sender};
use std::{path::PathBuf, sync::Arc};
use crate::graphics::material::image::{ImageBuilder};
use assetmanage_rs::*;
use futures::stream::{FuturesUnordered,StreamExt};

pub struct GPUImageHandle {
    // TODO: Rename ImageBuilder?
    image: Arc<ImageBuilder>,
    texture: Option<wgpu::Texture>,
    view: Option<wgpu::TextureView>,
    base_mip_layer: u32,
    sampler_hash: u32,
}
impl Asset<GPUImageLoader> for GPUImageHandle{
    type DataManager = ();
    type DataAsset = Arc<ImageBuilder>; //asset unique data
    type Structure = GPUImageHandle;
    fn construct(
        data_load: GPUImageHandle,
        _data_ass: &Self::DataAsset,
        _data_mgr: &Self::DataManager,
    ) -> Result<Self::Structure, std::io::Error> {
        Ok(data_load)
    }
}

pub struct GPUImageSource;

impl Source for GPUImageSource{
    type Input = Arc<ImageBuilder>;
    type Output = GPUImageHandle;
    fn load(item: Self::Input) -> Result<Self::Output, Box<dyn std::error::Error>> {
        //Input is the Arc<Image>/Texture that will be loaded to the GPU
        //Output will be the GPUImageHandle that will be returned
        todo!()
    }
}

pub struct GPUImageLoader {
    to_load: Receiver<(usize, PathBuf)>,
    loaded: Vec<Sender<(PathBuf, <<Self as assetmanage_rs::Loader>::Source as Source>::Output)>>,
    imageassetmanager: assetmanage_rs::Manager<ImageBuilder, MemoryLoader>,
}

impl GPUImageLoader{
    /// run the async load loop
    #[allow(unused)]
    pub async fn run(mut self) {
        let mut loading = FuturesUnordered::new();
        let mut to_load = FuturesUnordered::new();
        loop {
            self.to_load.try_iter().for_each(|(id, p)| {
                match self.imageassetmanager.get(p){
                    Some(asset) => {                
                        loading.push(async move {
                        (id, p, <<Self as Loader>::Source as Source>::load(asset))
                    })}
                    None => {}
                }
            });

            if let Some((manager_idx, path, Ok(bytes))) = loading.next().await {
                if let Some(sender) = self.loaded.get_mut(manager_idx) {
                    if sender.send((path, bytes)).is_err() {}
                }
            }
        }
    }
}

impl assetmanage_rs::Loader for GPUImageLoader{
    type Source = GPUImageSource;
    fn new(
        to_load: Receiver<(usize, PathBuf)>,
        loaded: Vec<Sender<(PathBuf, <Self::Source as Source>::Output)>>,
    ) -> Self {
        Self {to_load,loaded}
    }
}



#[cfg(test)]
mod tests{
    use super::GPUImageHandle;

    #[test]
    fn initial(){

        let mut builder = assetmanage_rs::Builder::new();

        let gpu_manager = builder.create_manager::<GPUImageHandle>(());

        let loader = builder.finish_loader();
        
    }
}