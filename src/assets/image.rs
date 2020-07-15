use std::{convert::TryFrom, path::PathBuf};

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, Eq, PartialEq)]
pub enum ImageFormat {
    RGB,
    SRGB,
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

// Image represents data on the CPU.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Image {
    // Byte data representing the pixels of the image.
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub(crate) path: PathBuf,
}

impl TryFrom<(Option<ImageRon>, PathBuf, Vec<u8>)> for Image {
    type Error = std::io::Error;
    fn try_from(
        (image_ron, path, data): (Option<ImageRon>, PathBuf, Vec<u8>),
    ) -> Result<Self, Self::Error> {
        let format = if image_ron.is_some() {
            image_ron.unwrap().format
        } else {
            ImageFormat::SRGB
        };

        let (image, width, height) = match format {
            ImageFormat::HDR32 | ImageFormat::HDR16 => {
                // Load the hdr image
                let decoder = image::hdr::HdrDecoder::new(data.as_slice()).unwrap();
                let metadata = decoder.metadata();
                let decoded = decoder.read_image_hdr().unwrap();

                let (w, h) = (metadata.width, metadata.height);

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
                (image_bytes, w, h)
            }
            _ => {
                let image = image::load_from_memory(&data).unwrap().to_rgba();
                let (width, height) = image.dimensions();

                (image.into_raw(), width, height)
            }
        };

        Ok(Self {
            data: image,
            width,
            height,
            path,
        })
    }
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, Eq, PartialEq)]
pub struct ImageRon {
    pub format: ImageFormat,
}

impl TryFrom<(PathBuf, Vec<u8>)> for ImageRon {
    type Error = ron::de::Error;
    fn try_from((_p, v): (PathBuf, Vec<u8>)) -> Result<Self, Self::Error> {
        ron::de::from_bytes(&v)
    }
}
