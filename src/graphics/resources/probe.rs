use super::RenderTarget;

#[derive(Debug, Copy, Clone)]
pub enum ProbeFormat {
    RGBA16,
    RGBA32,
}

impl Into<wgpu::TextureFormat> for ProbeFormat {
    fn into(self) -> wgpu::TextureFormat {
        match self {
            ProbeFormat::RGBA16 => wgpu::TextureFormat::Rgba16Float,
            ProbeFormat::RGBA32 => wgpu::TextureFormat::Rgba32Float,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ProbeQuality {
    Low,
    Medium,
    High,
}

impl ProbeQuality {
    pub(crate) fn get_irradiance_resoultion(&self) -> u32 {
        match self {
            ProbeQuality::Low => 64,
            ProbeQuality::Medium => 128,
            ProbeQuality::High => 256,
        }
    }

    pub(crate) fn get_probe_resoultion(&self) -> u32 {
        match self {
            ProbeQuality::Low => 512,
            ProbeQuality::Medium => 1024,
            ProbeQuality::High => 2048,
        }
    }

    pub(crate) fn get_specular_resoultion(&self) -> u32 {
        match self {
            ProbeQuality::Low => 256,
            ProbeQuality::Medium => 512,
            ProbeQuality::High => 1024,
        }
    }

    pub(crate) fn get_sample_count(&self) -> u32 {
        match self {
            ProbeQuality::Low => 512,
            ProbeQuality::Medium => 1024,
            ProbeQuality::High => 2048,
        }
    }
}

pub struct Probe {
    pub quality: ProbeQuality,
    pub format: ProbeFormat,
    sample_offset: u32,
    samples_per_frame: u32,
    sample_count: u32,
    scale: f32,
    irradiance_resoultion: u32,
    specular_resoultion: u32,
    probe_cube: RenderTarget,
    irradiance_target: RenderTarget,
    specular_target: RenderTarget,
    has_rendered: bool,
}

impl Probe {
    pub fn new(device: &wgpu::Device, quality: ProbeQuality, format: ProbeFormat) -> Self {
        let sample_offset = 0;
        let samples_per_frame = 512;
        let sample_count = 1024;
        let scale = 2.0;
        let probe_resoultion = quality.get_probe_resoultion();
        let irradiance_resoultion = quality.get_irradiance_resoultion();
        let specular_resoultion = quality.get_specular_resoultion();

        let wgpu_format: wgpu::TextureFormat = format.into();
        let probe_cube = RenderTarget::new(device, probe_resoultion as f32, probe_resoultion as f32, 6, 1, wgpu_format, wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_SRC | wgpu::TextureUsage::OUTPUT_ATTACHMENT);
        let irradiance_target = RenderTarget::new(device, irradiance_resoultion as f32, irradiance_resoultion as f32, 6, 1, wgpu_format, wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_SRC | wgpu::TextureUsage::OUTPUT_ATTACHMENT);
        let specular_target = RenderTarget::new(device, specular_resoultion as f32, specular_resoultion as f32, 6, 1, wgpu_format, wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_SRC | wgpu::TextureUsage::OUTPUT_ATTACHMENT);

        Self {
            quality,
            format,
            sample_offset,
            samples_per_frame,
            sample_count,
            scale,
            irradiance_resoultion,
            specular_resoultion,
            probe_cube,
            irradiance_target,
            specular_target,
            has_rendered: false,
        }
    }


}