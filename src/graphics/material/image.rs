use std::{fs, io, sync::Arc};

pub struct Image {
    pub name: String,
    pub texture: wgpu::Texture,
    pub extent: wgpu::Extent3d,
    pub sampler: wgpu::Sampler,
    pub view: wgpu::TextureView,
    pub format: wgpu::TextureFormat,
}

impl Image {
    pub fn new<T>(
        device: &wgpu::Device,
        queue: Arc<wgpu::Queue>,
        path: T,
        file_name: T,
    ) -> Self
    where
        T: Into<String>,
    {
        let path = path.into();

        let (image_bytes, texture_extent, format) = if path.ends_with(".hdr") {
            Self::create_hdr_image(path)
        } else if path.to_lowercase().contains("_normal")
            || path.to_lowercase().contains("metallic")
        {
            Self::create_normal_image(path)
        } else {
            Self::create_color_image(path)
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
            label: None,
        });

        let temp_buf = device.create_buffer_with_data(&image_bytes, wgpu::BufferUsage::COPY_SRC);

        queue.write_texture(
            wgpu::TextureCopyView {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &image_bytes,
            wgpu::TextureDataLayout {
                offset: 0,
                bytes_per_row: if format == wgpu::TextureFormat::Rgba8UnormSrgb
                    || format == wgpu::TextureFormat::Rgba8Unorm
                {
                    4 * texture_extent.width
                } else {
                    (4 * 4) * texture_extent.width
                },
                rows_per_image: 0,
            },
            texture_extent
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
            ..Default::default()
        });

        let view = texture.create_default_view();

        Self {
            name: file_name.into().clone(),
            texture,
            extent: texture_extent,
            sampler,
            view,
            format,
        }
    }

    fn create_normal_image(path: String) -> (Vec<u8>, wgpu::Extent3d, wgpu::TextureFormat) {
        let img = image::open(&path)
            .unwrap_or_else(|_| panic!("Image: Unable to open the file: {}", path))
            .to_rgba();
        let (width, height) = img.dimensions();
        let texture_extent = wgpu::Extent3d {
            width,
            height,
            depth: 1,
        };

        let image_bytes: Vec<u8> = img.into_raw();

        (image_bytes, texture_extent, wgpu::TextureFormat::Rgba8Unorm)
    }

    fn create_color_image(path: String) -> (Vec<u8>, wgpu::Extent3d, wgpu::TextureFormat) {
        let img = image::open(&path)
            .unwrap_or_else(|_| panic!("Image: Unable to open the file: {}", path))
            .to_rgba();
        let (width, height) = img.dimensions();
        let texture_extent = wgpu::Extent3d {
            width,
            height,
            depth: 1,
        };

        let image_bytes: Vec<u8> = img.into_raw();

        // TODO: Fix loading of images. We should use SRGB for textures and Unorm for roughness/normal maps/etc.
        // Should be done with a material loader perhaps?
        (
            image_bytes,
            texture_extent,
            wgpu::TextureFormat::Rgba8UnormSrgb,
        )
    }

    fn create_hdr_image(path: String) -> (Vec<u8>, wgpu::Extent3d, wgpu::TextureFormat) {
        // Load the image
        let decoder =
            image::hdr::HdrDecoder::new(io::BufReader::new(fs::File::open(&path).unwrap()))
                .unwrap();
        let metadata = decoder.metadata();
        let decoded = decoder.read_image_hdr().unwrap();

        let (w, h) = (metadata.width, metadata.height);

        let texture_extent = wgpu::Extent3d {
            width: w,
            height: h,
            depth: 1,
        };

        let image_data = decoded
            .iter()
            .flat_map(|pixel| vec![pixel[0], pixel[1], pixel[2], 1.0])
            .collect::<Vec<_>>();

        let image_bytes = unsafe {
            std::slice::from_raw_parts(image_data.as_ptr() as *const u8, image_data.len() * 4)
        }
        .to_vec();

        (
            image_bytes,
            texture_extent,
            wgpu::TextureFormat::Rgba32Float,
        )
    }
}
