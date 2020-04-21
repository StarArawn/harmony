use crate::{graphics::{RenderTarget, RenderGraph}, Application};

pub const SPEC_CUBEMAP_MIP_LEVELS: u32 = 6;

#[derive(Debug)]
pub struct Skybox {
    pub size: f32,
    pub(crate) cubemap_texture: wgpu::Texture,
    pub(crate) cubemap_view: wgpu::TextureView,
    pub(crate) cubemap_sampler: wgpu::Sampler,
    pub(crate) cubemap_bind_group: Option<wgpu::BindGroup>,
}

impl Skybox {
    pub fn new<T>(app: &mut Application, texture: T, size: f32) -> Self where T: Into<String> {
        // Create a new render graph for this process..
        let mut graph = RenderGraph::new(&app.renderer.device);
        
        let cube_map_target = RenderTarget::new(&app.renderer.device, size, size * 6.0, 1, 1, wgpu::TextureFormat::Rgba32Float, wgpu::TextureUsage::COPY_SRC | wgpu::TextureUsage::OUTPUT_ATTACHMENT);

        let cube_projection_pipeline_desc =
            crate::graphics::pipelines::equirectangular::CubeProjectionPipelineDesc::new(texture.into(), size);
        graph.add(&app.asset_manager, &mut app.renderer, "cube_projection", cube_projection_pipeline_desc, vec![], false, Some(cube_map_target), false);

        let irradiance_size = 64.0;
        let irradiance_target = RenderTarget::new(&app.renderer.device, irradiance_size, irradiance_size * 6.0, 1, 1, wgpu::TextureFormat::Rgba32Float, wgpu::TextureUsage::COPY_SRC | wgpu::TextureUsage::OUTPUT_ATTACHMENT);
        let irradiance_pipeline_desc = crate::graphics::pipelines::irradiance::IrradiancePipelineDesc::new(irradiance_size);
        graph.add(&app.asset_manager, &mut app.renderer, "irradiance", irradiance_pipeline_desc, vec!["cube_projection"], false, Some(irradiance_target), true);


        let specular_size = 64;
        // Add in a pass per mip level.
        for i in 0..SPEC_CUBEMAP_MIP_LEVELS {
            let res = (specular_size / 2u32.pow(i)) as f32;
            let specular_target = RenderTarget::new(&app.renderer.device, res, res * 6.0, 1, 1, wgpu::TextureFormat::Rgba32Float, wgpu::TextureUsage::COPY_SRC | wgpu::TextureUsage::OUTPUT_ATTACHMENT);
            let specular_pipeline_desc = crate::graphics::pipelines::specular::SpecularPipelineDesc::new(i, res);
            graph.add(&app.asset_manager, &mut app.renderer, format!("specular_{}", i), specular_pipeline_desc, vec!["irradiance"], false, Some(specular_target), true);
        }

        // We need to convert our regular texture map to a cube texture map with 6 faces.
        // Should be straight forward enough if we use equirectangular projection.
        // First we need a custom pipeline that will run in here to do the conversion.
        //let output = app.renderer.swap_chain.get_next_texture().unwrap();
        let mut command_buffers = graph.render(&mut app.renderer, &mut app.asset_manager, None, None);

        let cube_map = RenderTarget::new(
            &app.renderer.device,
            specular_size as f32,
            specular_size as f32,
            6,
            SPEC_CUBEMAP_MIP_LEVELS,
            wgpu::TextureFormat::Rgba32Float,
            wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        );
        
        let mut encoder = app.renderer.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        // Pull out mipmaps for specular and combine them into 1 image.
        for mip_level in 0..SPEC_CUBEMAP_MIP_LEVELS {
            let output = graph.pull_render_target(format!("specular_{}", mip_level));
            let res = (specular_size / 2u32.pow(mip_level)) as f32;
            for i in 0..6 {
                encoder.copy_texture_to_texture(
                    wgpu::TextureCopyView {
                        texture: &output.texture,
                        mip_level: 0,
                        array_layer: 0,
                        origin: wgpu::Origin3d {
                            x: 0,
                            y: res as u32 * i,
                            z: 0,
                        },
                    },
                    wgpu::TextureCopyView {
                        texture: &cube_map.texture,
                        mip_level,
                        array_layer: i,
                        origin: wgpu::Origin3d::ZERO,
                    },
                    wgpu::Extent3d {
                        width: res as u32,
                        height: res as u32,
                        depth: 1,
                    },
                );
            }
        }

        command_buffers.push(encoder.finish());

        // Add copy texture copy command buffer and push to all command buffers to the queue
        app.renderer.queue.submit(&command_buffers);

        app.renderer.device.poll(wgpu::Maintain::Wait);

        let cubemap_view = cube_map.texture.create_view(&wgpu::TextureViewDescriptor {
            format: wgpu::TextureFormat::Rgba32Float,
            dimension: wgpu::TextureViewDimension::Cube,
            aspect: wgpu::TextureAspect::default(),
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            array_layer_count: 6,
        });
        let cubemap_texture = cube_map.texture;

        let cubemap_sampler = app.renderer.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: wgpu::CompareFunction::Undefined,
        });

        Self {
            size,
            cubemap_texture,
            cubemap_view,
            cubemap_sampler,
            cubemap_bind_group: None,
        }
    }

    pub(crate) fn create_bind_group(&mut self, device: &wgpu::Device, material_layout: &wgpu::BindGroupLayout) {
        let bind_group = device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &material_layout,
                bindings: &[
                    wgpu::Binding {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(
                            &self.cubemap_view,
                        ),
                    },
                    wgpu::Binding {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(
                            &self.cubemap_sampler,
                        ),
                    },
                ],
                label: None,
            });
        self.cubemap_bind_group = Some(bind_group);
    }
}
