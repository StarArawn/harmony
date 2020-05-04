use std::{fs, io};

pub struct Image {
    pub(crate) name: String,
    pub(crate) texture: wgpu::Texture,
    pub(crate) size: wgpu::Extent3d,
    pub(crate) sampler: wgpu::Sampler,
    pub(crate) view: wgpu::TextureView,
    pub(crate) format: wgpu::TextureFormat,
}

impl Image {
    pub fn new_hdr(
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        path: &str,
    ) -> Self {
        // Load the image
        let decoder =
            image::hdr::HdrDecoder::new(io::BufReader::new(fs::File::open(path).unwrap())) //TODO: make unfailable
                .unwrap();
        let metadata = decoder.metadata();
        let decoded = decoder.read_image_hdr().unwrap();

        let (w, h) = (metadata.width, metadata.height);

        let size = wgpu::Extent3d {
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
        let format = wgpu::TextureFormat::Rgba32Float;

        Self::new(path.into(), device, encoder, image_bytes, size, format)
    }

    pub fn new_normal(
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        path: &str,
    ) -> Self {
        let img = image::open(path)
            .unwrap_or_else(|_| panic!("Image: Unable to open the file: {:?}", path)) //TODO: make unfailable
            .to_rgba();
        let (width, height) = img.dimensions();
        let size = wgpu::Extent3d {
            width,
            height,
            depth: 1,
        };
        let image_bytes: Vec<u8> = img.into_raw();

        let format = wgpu::TextureFormat::Rgba8Unorm;

        Self::new(path.into(), device, encoder, image_bytes, size, format)
    }

    pub fn new_color(
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        path: &str,
    ) -> Self {
        let img = image::open(path)
            .unwrap_or_else(|_| panic!("Image: Unable to open the file: {:?}", path)) //TODO: make unfailable
            .to_rgba();
        let (width, height) = img.dimensions();
        let size = wgpu::Extent3d {
            width,
            height,
            depth: 1,
        };

        let image_bytes: Vec<u8> = img.into_raw();

        let format = wgpu::TextureFormat::Rgba8UnormSrgb;

        Self::new(path.into(), device, encoder, image_bytes, size, format)
    }

    // creates a default white 1x1 texture Should be automatically added to the Image Assets
    pub fn new_white(device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder) -> Self {
        let name = String::from("white");

        let image_bytes = vec![255u8, 255u8, 255u8, 255u8];

        let size = wgpu::Extent3d {
            width: 1u32,
            height: 1u32,
            depth: 1u32,
        };
        let format = wgpu::TextureFormat::Rgba8Unorm;

        Self::new(name, device, encoder, image_bytes, size, format)
    }

    pub fn new(
        name: String,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        image_bytes: Vec<u8>,
        size: wgpu::Extent3d,
        format: wgpu::TextureFormat,
    ) -> Self {
        //TODO: dont create a new sampler for each image. Reuse
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
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

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
            label: None,
        });

        let temp_buf = device.create_buffer_with_data(&image_bytes, wgpu::BufferUsage::COPY_SRC);

        encoder.copy_buffer_to_texture(
            wgpu::BufferCopyView {
                buffer: &temp_buf,
                offset: 0,
                bytes_per_row: match format {
                    wgpu::TextureFormat::Rgba8UnormSrgb => 4 * size.width,
                    wgpu::TextureFormat::Rgba8Unorm => 4 * size.width,
                    _ => (4 * 4) * size.width,
                },
                rows_per_image: 0, //TODO: set to size.height if size.depth != 0?
            },
            wgpu::TextureCopyView {
                texture: &texture,
                mip_level: 0,
                array_layer: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            size,
        );

        let view = texture.create_default_view();

        Self {
            name,
            texture,
            size,
            sampler,
            view,
            format,
        }
    }
}
