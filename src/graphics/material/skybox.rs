use nalgebra_glm::Vec3;

use crate::{
    graphics::{
        resources::{GPUResourceManager, RenderTarget},
        RenderGraph,
    },
    Application, AssetManager,
};
use std::sync::Arc;

pub const SPEC_CUBEMAP_MIP_LEVELS: u32 = 6;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SkyboxType {
    ClearColor,
    HdrCubemap,
    RealTime,
}

pub struct Skybox {
    pub size: f32,
    pub skybox_type: SkyboxType,
    pub clear_color: Vec3,
    pub(crate) color_texture: Option<wgpu::Texture>,
    pub(crate) color_view: Option<wgpu::TextureView>,
    pub(crate) cubemap_sampler: Option<wgpu::Sampler>,
    pub(crate) cubemap_bind_group: Option<wgpu::BindGroup>,
    pub(crate) pbr_bind_group: Option<wgpu::BindGroup>,
}

impl Skybox {
    pub fn new_hdr<T>(app: &mut Application, texture: T, size: f32) -> Self
    where
        T: Into<String>,
    {
        // Create a new render graph for this process..
        let mut graph = { RenderGraph::new(&mut app.resources, false) };

        let asset_manager = app.resources.get::<AssetManager>().unwrap();
        let device = app.resources.get::<Arc<wgpu::Device>>().unwrap();
        let sc_desc = app.resources.get::<wgpu::SwapChainDescriptor>().unwrap();
        let resource_manager = app.resources.get::<Arc<GPUResourceManager>>().unwrap();

        let cube_map_target = RenderTarget::new(
            &device,
            size,
            size * 6.0,
            1,
            1,
            wgpu::TextureFormat::Rgba32Float,
            wgpu::TextureUsage::COPY_SRC | wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        );

        let cube_projection_pipeline_desc =
            crate::graphics::pipelines::equirectangular::CubeProjectionPipelineDesc::new(
                texture.into(),
                size,
            );
        graph.add(
            &asset_manager,
            &device,
            &sc_desc,
            resource_manager.clone(),
            "cube_projection",
            cube_projection_pipeline_desc,
            vec![],
            false,
            Some(cube_map_target),
            false,
        );

        // We need to convert our regular texture map to a cube texture map with 6 faces.
        // Should be straight forward enough if we use equirectangular projection.
        // First we need a custom pipeline that will run in here to do the conversion.
        // let output = app.renderer.swap_chain.get_next_texture().unwrap();
        let command_buffer = graph.render_one_time(
            &device,
            &asset_manager,
            resource_manager.clone(),
            &mut app.current_scene.world,
            None,
            None,
        );
        // Push to all command buffers to the queue
        let queue = app.resources.get::<Arc<wgpu::Queue>>().unwrap();
        queue.submit(vec![command_buffer]);

        // Note that we're not calling `.await` here.
        // let buffer_future = output_buffer.map_read(0, (specular_brdf_size * specular_brdf_size) as u64 * std::mem::size_of::<u32>() as u64);

        device.poll(wgpu::Maintain::Wait);

        // futures::executor::block_on(Self::save(buffer_future));

        let color = graph.pull_render_target("cube_projection");

        let color_view = color.texture.create_view(&wgpu::TextureViewDescriptor {
            label: None,
            format: wgpu::TextureFormat::Rgba32Float,
            dimension: wgpu::TextureViewDimension::Cube,
            aspect: wgpu::TextureAspect::default(),
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            array_layer_count: 6,
        });

        let cubemap_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        Self {
            size,
            color_texture: Some(color.texture),
            color_view: Some(color_view),
            cubemap_sampler: Some(cubemap_sampler),
            cubemap_bind_group: None,
            pbr_bind_group: None,
            clear_color: Vec3::zeros(),
            skybox_type: SkyboxType::HdrCubemap,
        }
    }

    pub fn create_clear_color(color: Vec3) -> Self {
        Self {
            size: 0.0,
            color_texture: None,
            color_view: None,
            cubemap_sampler: None,
            cubemap_bind_group: None,
            pbr_bind_group: None,
            clear_color: color,
            skybox_type: SkyboxType::ClearColor,
        }
    }

    pub fn create_realtime() -> Self {
        Self {
            size: 0.0,
            color_texture: None,
            color_view: None,
            cubemap_sampler: None,
            cubemap_bind_group: None,
            pbr_bind_group: None,
            clear_color: Vec3::new(0.0, 0.0, 0.0),
            skybox_type: SkyboxType::RealTime,
        }
    }

    pub(crate) fn create_realtime_bind_group(
        &mut self,
        device: &wgpu::Device,
        asset_manager: &AssetManager,
        material_layout: Arc<wgpu::BindGroupLayout>,
    ) {
        let rayleigh_texture = asset_manager.get_texture("rayleigh.hdr");
        let mie_texture = asset_manager.get_texture("mie.hdr");

        let rayleigh_texture = futures::executor::block_on(rayleigh_texture.get_async()).unwrap();
        let mie_texture = futures::executor::block_on(mie_texture.get_async()).unwrap();

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &material_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&rayleigh_texture.view),
                },
                wgpu::Binding {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&mie_texture.view),
                },
            ],
            label: None,
        });
        self.cubemap_bind_group = Some(bind_group);
    }

    pub(crate) fn create_bind_group2(
        &mut self,
        device: &wgpu::Device,
        material_layout: Arc<wgpu::BindGroupLayout>,
    ) {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &material_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(self.color_view.as_ref().unwrap()),
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(
                        self.cubemap_sampler.as_ref().unwrap(),
                    ),
                },
            ],
            label: None,
        });
        self.cubemap_bind_group = Some(bind_group);
    }

    // async fn save(
    //     buffer_future: impl futures::Future<
    //         Output = Result<wgpu::BufferReadMapping, wgpu::BufferAsyncErr>,
    //     >,
    // ) {
    //     if let Ok(mapping) = buffer_future.await {
    //         let mut png_encoder = png::Encoder::new(File::create("save.png").unwrap(), 128, 128);
    //         png_encoder.set_depth(png::BitDepth::Eight);
    //         png_encoder.set_color(png::ColorType::RGBA);
    //         png_encoder
    //             .write_header()
    //             .unwrap()
    //             .write_image_data(mapping.as_slice())
    //             .unwrap();
    //     }
    // }
}
