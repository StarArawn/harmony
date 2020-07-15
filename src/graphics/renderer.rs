use super::resources::GPUResourceManager;
use legion::systems::resource::Resources;
use std::sync::Arc;

pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
pub const FRAME_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

pub struct DepthTexture(pub wgpu::TextureView);

pub struct Renderer {
    pub(crate) surface: wgpu::Surface,
    pub size: winit::dpi::PhysicalSize<u32>,
    adapter: wgpu::Adapter,
    pub(crate) swap_chain: wgpu::SwapChain,
    pub(crate) window: winit::window::Window,
}

impl Renderer {
    pub(crate) async fn new(
        window: winit::window::Window,
        size: winit::dpi::PhysicalSize<u32>,
        resources: &mut Resources,
    ) -> Self {
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(&window) };

        let (needed_features, unsafe_extensions) =
            (wgpu::Features::empty(), wgt::UnsafeFeatures::disallow());

        let adapter = instance
            .request_adapter(
                &wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::HighPerformance,
                    compatible_surface: Some(&surface),
                },
                unsafe_extensions,
            )
            .await
            .unwrap();

        let adapter_features = adapter.features();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: adapter_features & needed_features,
                    limits: wgpu::Limits::default(),
                    shader_validation: true,
                },
                None,
            )
            .await
            .unwrap();

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: FRAME_FORMAT,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: sc_desc.width,
                height: sc_desc.height,
                depth: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH_FORMAT,
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            label: None,
        });
        let device = Arc::new(device);

        resources.insert(Arc::new(GPUResourceManager::new(device.clone())));
        resources.insert(sc_desc);
        resources.insert(Arc::new(queue));
        resources.insert(device.clone());
        resources.insert(DepthTexture(depth_texture.create_default_view()));

        Self {
            surface,
            size,
            adapter,
            swap_chain,
            window,
        }
    }

    pub(crate) fn render(&mut self) -> wgpu::SwapChainFrame {
        let output = self.swap_chain.get_next_frame().unwrap();

        output
    }
}
