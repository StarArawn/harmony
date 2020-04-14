pub struct Renderer {
    surface: wgpu::Surface,
    pub size: winit::dpi::PhysicalSize<u32>,
    adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub swap_chain: wgpu::SwapChain,
    pub window: winit::window::Window,
    pub sc_desc: wgpu::SwapChainDescriptor,
}

impl Renderer {
    pub async fn new(window: winit::window::Window, size: winit::dpi::PhysicalSize<u32>, surface: wgpu::Surface) -> Self {
        let adapter = wgpu::Adapter::request(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: Some(&surface),
            },
            wgpu::BackendBit::PRIMARY,
        )
        .await
        .unwrap();
    
        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
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

        Self {
            surface,
            size,
            adapter,
            device,
            queue,
            swap_chain,
            window,
            sc_desc,
        }
    }

    pub fn render(&mut self) -> wgpu::SwapChainOutput {
        let output = self.swap_chain.get_next_texture().unwrap();

        output
    }
}