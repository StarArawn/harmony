use serde::{Deserialize, Serialize};
use std::{io, sync::Arc, path::{Path, PathBuf}};

#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum ImageFormat {
    SRGB,
    RGB,
    HDR16,
    HDR32,
}

impl Into<wgpu::TextureFormat> for ImageFormat {
    fn into(self) -> wgpu::TextureFormat {
        match self {
            ImageFormat::HDR16 => wgpu::TextureFormat::Rgba16Float,
            ImageFormat::HDR32 => wgpu::TextureFormat::Rgba32Float,
            ImageFormat::RGB => wgpu::TextureFormat::Rgba8Unorm,
            ImageFormat::SRGB => wgpu::TextureFormat::Rgba8UnormSrgb,
        }
    }
}

#[derive(Eq, PartialEq, Hash, Debug, Clone, Serialize, Deserialize)]
pub struct ImageInfo {
    pub path: PathBuf,
    pub format: ImageFormat,
}

impl ImageInfo {
    pub fn new<P: AsRef<Path>>(path: P, format: ImageFormat) -> Self { Self { path: path.as_ref().into(), format } }
}

pub(crate) struct ImageData {
    pub image_info: Arc<ImageInfo>,
    pub bytes: Vec<u8>,
}

fn create_texture(
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    texture_extent: wgpu::Extent3d,
    format: wgpu::TextureFormat,
    bytes: &Vec<u8>,
) -> (wgpu::Texture, wgpu::Sampler) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        size: texture_extent,
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
        bytes,
        wgpu::TextureDataLayout {
            offset: 0,
            // TODO: Figure out a better method of detecting bytes per row.
            bytes_per_row: if format == wgpu::TextureFormat::Rgba8UnormSrgb
                || format == wgpu::TextureFormat::Rgba8Unorm
            {
                4 * texture_extent.width
            } else {
                (4 * 4) * texture_extent.width
            },
            rows_per_image: 0,
        },
        texture_extent,
    );

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: None,
        address_mode_u: wgpu::AddressMode::Repeat,
        address_mode_v: wgpu::AddressMode::Repeat,
        address_mode_w: wgpu::AddressMode::Repeat,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Linear,
        lod_min_clamp: -100.0,
        lod_max_clamp: 100.0,
        compare: wgpu::CompareFunction::Undefined,
    });

    (texture, sampler)
}

impl ImageData {
    pub(crate) fn new(image_info: Arc<ImageInfo>, bytes: Vec<u8>) -> Self {
        Self { image_info, bytes }
    }

    pub fn build(&self) -> Image {
        let (image_bytes, width, height) = match self.image_info.format {
            ImageFormat::HDR16 | ImageFormat::HDR32 => {
                let decoder = image::hdr::HdrDecoder::new(self.bytes.as_slice()).unwrap();
                let metadata = decoder.metadata();
                let decoded = decoder.read_image_hdr().unwrap();

                let image_data = decoded
                    .iter()
                    .flat_map(|pixel| vec![pixel[0], pixel[1], pixel[2], 1.0])
                    .collect::<Vec<_>>();

                let image_bytes = unsafe {
                    std::slice::from_raw_parts(
                        image_data.as_ptr() as *const u8,
                        image_data.len() * 4,
                    )
                }
                .to_vec();

                (image_bytes, metadata.width, metadata.height)
            }
            ImageFormat::RGB | ImageFormat::SRGB => {
                let image = image::load_from_memory(&self.bytes).unwrap().to_rgba();
                let (width, height) = image.dimensions();

                (image.into_raw(), width, height)
            }
        };

        let texture_extent = wgpu::Extent3d {
            width,
            height,
            depth: 1,
        };

        let format: wgpu::TextureFormat = self.image_info.format.into();

        Image {
            image_info: self.image_info.clone(),
            data: image_bytes,
            extent: texture_extent,
            format,
        }
    }
}

pub struct Image {
    pub image_info: Arc<ImageInfo>,
    pub extent: wgpu::Extent3d,
    pub data: Vec<u8>,
    pub format: wgpu::TextureFormat,
}

impl Image {
    pub fn create_gpu_texture(
        &self,
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
    ) -> (wgpu::Texture, wgpu::TextureView, wgpu::Sampler) {
        let (texture, sampler) =
            create_texture(device, queue, self.extent, self.format, &self.data);

        let view = texture.create_default_view();
        (texture, view, sampler)
    }
}

impl assetmanage_rs::Asset<assetmanage_rs::MemoryLoader> for ImageData {
    type AssetSupplement = Arc<ImageInfo>;
    type ManagerSupplement = ();
    type Structure = Image; //TODO: return Image. explanation @ imageassetmanager
    fn construct(
        bytes: Vec<u8>,
        data_ass: &Self::AssetSupplement,
        (): &Self::ManagerSupplement,
    ) -> Result<Self::Structure, io::Error> {
        Ok(ImageData::new(data_ass.clone(), bytes).build())
    }
}

//impl assetmanage_rs::Asset<assetmanage_rs::MemoryLoader> for ImageInfo {
//    type AssetSupplement = PathBuf; //Relative path
//    type ManagerSupplement = ();
//    type Structure = Self;
//    fn construct(bytes: Vec<u8>, data_ass: &Self::AssetSupplement, _data_mgr: &Self::ManagerSupplement) -> Result<Self::Structure, io::Error> {
//        let mut image_info = ron::de::from_bytes::<ImageInfo>(&bytes)
//            .map_err(|e| std::io::Error::new(ErrorKind::InvalidData, e))?;
//
//        // add relative path of the ron to the imagepath inside the ron
//        let mut path = data_ass.clone();
//        path.pop();
//        image_info.file = path.join(image_info.file);
//
//        Ok(image_info)
//    }
//}
