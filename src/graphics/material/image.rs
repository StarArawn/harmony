pub struct Image {
    pub(crate) name: String,
    pub(crate) texture: wgpu::Texture,
    pub(crate) extent: wgpu::Extent3d,
    pub(crate) sampler: wgpu::Sampler,
    pub(crate) view: wgpu::TextureView,
}

impl Image {
    pub fn new<T>(device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder, path: T, file_name: T) -> Self where T: Into<String> {
        let path = path.into();
        let img = image::open(&path).unwrap_or_else(|_| panic!("Image: Unable to open the file: {}", path)).to_rgba();
        let (width, height) = img.dimensions();
        let texture_extent = wgpu::Extent3d {
            width,
            height,
            depth: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_extent,
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
            label: None,
        });

        let image_bytes: Vec<u8> = img.into_raw();
        dbg!(&image_bytes.len());
        let temp_buf = device.create_buffer_with_data(&image_bytes, wgpu::BufferUsage::COPY_SRC);

        encoder.copy_buffer_to_texture(
            wgpu::BufferCopyView {
                buffer: &temp_buf,
                offset: 0,
                bytes_per_row: 4 * width,
                rows_per_image: 0,
            },
            wgpu::TextureCopyView {
                texture: &texture,
                mip_level: 0,
                array_layer: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            texture_extent,
        );

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: wgpu::CompareFunction::Undefined,
        });

        let view = texture.create_default_view();

        Self {
            name: file_name.into().clone(),
            texture,
            extent: texture_extent,
            sampler,
            view,
        }
    }
}