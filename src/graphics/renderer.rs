use legion::systems::resource::Resources;

pub(crate) const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

pub struct Renderer {
    pub(crate) surface: wgpu::Surface,
    pub size: winit::dpi::PhysicalSize<u32>,
    adapter: wgpu::Adapter,
    pub(crate) swap_chain: wgpu::SwapChain,
    pub(crate) window: winit::window::Window,
    pub(crate) forward_depth: wgpu::TextureView,
}

impl Renderer {
    pub(crate) async fn new(
        window: winit::window::Window,
        size: winit::dpi::PhysicalSize<u32>,
        surface: wgpu::Surface,
        resources: &mut Resources,
    ) -> Self {
        let adapter = wgpu::Adapter::request(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: Some(&surface),
            },
            wgpu::BackendBit::PRIMARY,
        )
        .await
        .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                extensions: wgpu::Extensions {
                    anisotropic_filtering: false,
                },
                limits: wgpu::Limits::default(),
            })
            .await;

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
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

        resources.insert(device);
        resources.insert(sc_desc);
        resources.insert(queue);

        Self {
            surface,
            size,
            adapter,
            swap_chain,
            window,
            forward_depth: depth_texture.create_default_view(),
        }
    }

    pub(crate) fn render(&mut self) -> wgpu::SwapChainOutput {
        let output = self.swap_chain.get_next_texture().unwrap();

        output
    }
}
