
use futures::stream::{FuturesUnordered, StreamExt};
use std::sync::mpsc::{Receiver, Sender};
use std::{path::PathBuf, sync::Arc};
use crate::graphics::material::image::ImageBuilder;

pub struct GPUImageHandle {
    // TODO: Rename ImageBuilder?
    image: Arc<ImageBuilder>,
    texture: Option<wgpu::Texture>,
    view: Option<wgpu::TextureView>,
    base_mip_layer: u32,
    sampler_hash: u32,
}

pub(crate) struct ImageLoadPacket {
    image_key: usize,
    asset_path: PathBuf,
    image: Arc<ImageBuilder>,
}

pub struct GPUImageManagerLoader {
    to_load: Receiver<ImageLoadPacket>,
    loaded: Sender<(usize, GPUImageHandle)>,
}
