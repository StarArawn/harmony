#[derive(Debug)]
pub struct RenderTarget {
    pub texture: wgpu::Texture,
    pub texture_view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl RenderTarget {
    pub fn new(device: &wgpu::Device, width: f32, height: f32, format: wgpu::TextureFormat) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: width as u32,
                height: height as u32,
                depth: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: format,
            usage: wgpu::TextureUsage::SAMPLED
                | wgpu::TextureUsage::COPY_SRC
                | wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            label: None,
        });
        let texture_view = texture.create_default_view();
        Self {
            texture,
            texture_view,
            sampler: device.create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Linear,
                lod_min_clamp: -100.0,
                lod_max_clamp: 100.0,
                compare: wgpu::CompareFunction::Undefined,
            }),
        }
    }

    pub fn complete(self) -> (wgpu::Texture, wgpu::TextureView, wgpu::Sampler) {
        (self.texture, self.texture_view, self.sampler)
    }
}