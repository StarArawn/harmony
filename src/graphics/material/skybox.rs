use crate::{
    graphics::{
        resources::{BindGroup, GPUResourceManager, RenderTarget},
        RenderGraph,
    },
    Application, AssetManager,
};

pub const SPEC_CUBEMAP_MIP_LEVELS: u32 = 6;

pub struct Skybox {
    pub size: f32,
    pub(crate) color_texture: wgpu::Texture,
    pub(crate) color_view: wgpu::TextureView,
    pub(crate) irradiance_texture: wgpu::Texture,
    pub(crate) irradiance_view: wgpu::TextureView,
    pub(crate) specular_texture: wgpu::Texture,
    pub(crate) specular_view: wgpu::TextureView,
    pub(crate) brdf_texture: wgpu::Texture,
    pub(crate) brdf_view: wgpu::TextureView,
    pub(crate) cubemap_sampler: wgpu::Sampler,
    pub(crate) cubemap_bind_group: Option<wgpu::BindGroup>,
    pub(crate) pbr_bind_group: Option<wgpu::BindGroup>,
}

impl Skybox {
    pub fn new<T>(app: &mut Application, texture: T, size: f32) -> Self
    where
        T: Into<String>,
    {
        // Create a new render graph for this process..
        let mut graph = { RenderGraph::new(&mut app.resources, false) };

        let asset_manager = app.resources.get::<AssetManager>().unwrap();
        let device = app.resources.get::<wgpu::Device>().unwrap();
        let sc_desc = app.resources.get::<wgpu::SwapChainDescriptor>().unwrap();
        let mut resource_manager = app.resources.get_mut::<GPUResourceManager>().unwrap();

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
            &mut resource_manager,
            "cube_projection",
            cube_projection_pipeline_desc,
            vec![],
            false,
            Some(cube_map_target),
            false,
        );

        let irradiance_size = 64.0;
        let irradiance_target = RenderTarget::new(
            &device,
            irradiance_size,
            irradiance_size * 6.0,
            1,
            1,
            wgpu::TextureFormat::Rgba32Float,
            wgpu::TextureUsage::COPY_SRC | wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        );
        let irradiance_pipeline_desc =
            crate::graphics::pipelines::irradiance::IrradiancePipelineDesc::new(irradiance_size);
        graph.add(
            &asset_manager,
            &device,
            &sc_desc,
            &mut resource_manager,
            "irradiance",
            irradiance_pipeline_desc,
            vec!["cube_projection"],
            false,
            Some(irradiance_target),
            true,
        );

        let specular_size = 128;
        // Add in a pass per mip level.
        for i in 0..SPEC_CUBEMAP_MIP_LEVELS {
            let res = (specular_size / 2u32.pow(i)) as f32;
            let specular_target = RenderTarget::new(
                &device,
                res,
                res * 6.0,
                1,
                1,
                wgpu::TextureFormat::Rgba32Float,
                wgpu::TextureUsage::COPY_SRC | wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            );
            let specular_pipeline_desc =
                crate::graphics::pipelines::specular::SpecularPipelineDesc::new(i, res);
            graph.add(
                &asset_manager,
                &&device,
                &sc_desc,
                &mut resource_manager,
                format!("specular_{}", i),
                specular_pipeline_desc,
                vec!["irradiance"],
                false,
                Some(specular_target),
                true,
            );
        }

        // Specular BRDF
        let specular_brdf_size = 256.0;
        let spec_brdf_texture = RenderTarget::new(
            &device,
            specular_brdf_size,
            specular_brdf_size,
            1,
            1,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            wgpu::TextureUsage::SAMPLED
                | wgpu::TextureUsage::COPY_SRC
                | wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        );
        let spec_brdf_pipeline_desc =
            crate::graphics::pipelines::specular_brdf::SpecularBRDFPipelineDesc::new(
                specular_brdf_size,
            );
        graph.add(
            &asset_manager,
            &device,
            &sc_desc,
            &mut resource_manager,
            "spec_brdf",
            spec_brdf_pipeline_desc,
            vec![],
            false,
            Some(spec_brdf_texture),
            false,
        );

        // let output_buffer = app.renderer.device.create_buffer(&wgpu::BufferDescriptor {
        //     size: (specular_brdf_size * specular_brdf_size) as u64 * std::mem::size_of::<u32>() as u64,
        //     usage: wgpu::BufferUsage::MAP_READ | wgpu::BufferUsage::COPY_DST,
        //     label: None,
        // });

        // We need to convert our regular texture map to a cube texture map with 6 faces.
        // Should be straight forward enough if we use equirectangular projection.
        // First we need a custom pipeline that will run in here to do the conversion.
        // let output = app.renderer.swap_chain.get_next_texture().unwrap();
        let command_buffer = graph.render_one_time(
            &device,
            &asset_manager,
            &mut resource_manager,
            &mut app.current_scene.world,
            None,
            None,
        );

        let specular = RenderTarget::new(
            &device,
            specular_size as f32,
            specular_size as f32,
            6,
            SPEC_CUBEMAP_MIP_LEVELS,
            wgpu::TextureFormat::Rgba32Float,
            wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        );

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

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
                        texture: &specular.texture,
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

        // let output = graph.pull_render_target("spec_brdf");
        // encoder.copy_texture_to_buffer(
        //     wgpu::TextureCopyView {
        //         texture: &output.texture,
        //         mip_level: 0,
        //         array_layer: 0,
        //         origin: wgpu::Origin3d::ZERO,
        //     },
        //     wgpu::BufferCopyView {
        //         buffer: &output_buffer,
        //         offset: 0,
        //         bytes_per_row: std::mem::size_of::<u32>() as u32 * specular_brdf_size as u32,
        //         rows_per_image: 0,
        //     },
        //     wgpu::Extent3d {
        //         width: specular_brdf_size as u32,
        //         height: specular_brdf_size as u32,
        //         depth: 1,
        //     },
        // );

        // Push to all command buffers to the queue
        let queue = app.resources.get::<wgpu::Queue>().unwrap();
        queue.submit(vec![command_buffer, encoder.finish()]);

        // Note that we're not calling `.await` here.
        // let buffer_future = output_buffer.map_read(0, (specular_brdf_size * specular_brdf_size) as u64 * std::mem::size_of::<u32>() as u64);

        device.poll(wgpu::Maintain::Wait);

        // futures::executor::block_on(Self::save(buffer_future));

        let color = graph.pull_render_target("cube_projection");
        let irradiance = graph.pull_render_target("irradiance");
        let brdf = graph.pull_render_target("spec_brdf");

        let color_view = color.texture.create_view(&wgpu::TextureViewDescriptor {
            format: wgpu::TextureFormat::Rgba32Float,
            dimension: wgpu::TextureViewDimension::Cube,
            aspect: wgpu::TextureAspect::default(),
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            array_layer_count: 6,
        });

        let irradiance_view = irradiance
            .texture
            .create_view(&wgpu::TextureViewDescriptor {
                format: wgpu::TextureFormat::Rgba32Float,
                dimension: wgpu::TextureViewDimension::Cube,
                aspect: wgpu::TextureAspect::default(),
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                array_layer_count: 6,
            });

        let specular_view = specular.texture.create_view(&wgpu::TextureViewDescriptor {
            format: wgpu::TextureFormat::Rgba32Float,
            dimension: wgpu::TextureViewDimension::Cube,
            aspect: wgpu::TextureAspect::default(),
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            array_layer_count: 6,
        });

        let cubemap_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
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
            color_texture: color.texture,
            color_view: color_view,
            irradiance_texture: irradiance.texture,
            irradiance_view,
            specular_texture: specular.texture,
            specular_view,
            brdf_texture: brdf.texture,
            brdf_view: brdf.texture_view,
            cubemap_sampler,
            cubemap_bind_group: None,
            pbr_bind_group: None,
        }
    }

    pub(crate) fn create_bind_group2(
        &mut self,
        device: &wgpu::Device,
        material_layout: &wgpu::BindGroupLayout,
    ) {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &material_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.color_view),
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.cubemap_sampler),
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

    pub fn create_bind_group<'a>(
        &self,
        device: &wgpu::Device,
        pipeline_layout: &'a wgpu::BindGroupLayout,
    ) -> BindGroup {
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &pipeline_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.irradiance_view),
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&self.specular_view),
                },
                wgpu::Binding {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&self.brdf_view),
                },
            ],
            label: None,
        });

        BindGroup::new(3, bind_group)
    }
}
