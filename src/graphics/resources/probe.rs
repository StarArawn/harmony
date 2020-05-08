use bytemuck::{Pod, Zeroable};
use legion::prelude::*;
use nalgebra_glm::{Vec3, Vec4};
use std::sync::Arc;

use super::{BindGroup, GPUResourceManager, RenderTarget};
use crate::{
    graphics::{pipeline_manager::PipelineManager, RenderGraph},
    scene::components::CameraData,
    AssetManager,
};
//use crate::graphics::systems::create_render_schedule_builder;

pub struct CurrentRenderTarget(pub Option<(Arc<RenderTarget>, wgpu::TextureView)>);

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
    pub id: u32,
    pub position: Vec3,
    pub quality: ProbeQuality,
    pub format: ProbeFormat,
    sample_offset: u32,
    samples_per_frame: u32,
    sample_count: u32,
    samples_remaining: u32,
    scale: f32,
    irradiance_resoultion: u32,
    specular_resoultion: u32,
    probe_cube: Arc<RenderTarget>,
    irradiance_target: RenderTarget,
    specular_target: RenderTarget,
    brdf_texture: RenderTarget,
    pub(crate) has_rendered: bool,
}

impl Probe {
    pub(crate) fn new(
        id: u32,
        position: Vec3,
        resources: &Resources,
        quality: ProbeQuality,
        format: ProbeFormat,
    ) -> Self {
        let sample_offset = 0;
        let samples_per_frame = 1024;
        let sample_count = 1024;
        let scale = 1.0;
        let probe_resoultion = quality.get_probe_resoultion();
        let irradiance_resoultion = quality.get_irradiance_resoultion();
        let specular_resoultion = quality.get_specular_resoultion();
        let wgpu_format: wgpu::TextureFormat = format.into();

        // Create the specular workflow pipeline
        crate::graphics::pipelines::specular2::create(resources, wgpu_format);

        let brdf_texture = {
            let device = resources.get::<wgpu::Device>().unwrap();
            RenderTarget::new(
                &device,
                512.0,
                512.0,
                1,
                1,
                wgpu_format,
                wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            )
        };
        crate::graphics::pipelines::brdf::create(resources, &brdf_texture, wgpu_format);

        let device = resources.get::<wgpu::Device>().unwrap();

        // TODO: Replace this one day with the correct format. Which means building out multiple same pipelines with only different formats.
        // It really should be 16 or 32 bits per pixel for HDR..
        let mut probe_cube = RenderTarget::new(
            &device,
            probe_resoultion as f32,
            probe_resoultion as f32,
            6,
            1,
            wgpu_format,
            wgpu::TextureUsage::SAMPLED
                | wgpu::TextureUsage::COPY_SRC
                | wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        );
        // Probe cube needs depth buffer as we are rendering the scene to it.
        probe_cube.with_depth(&device);
        let irradiance_target = RenderTarget::new(
            &device,
            irradiance_resoultion as f32,
            irradiance_resoultion as f32,
            6,
            1,
            wgpu_format,
            wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        );
        let specular_target = RenderTarget::new(
            &device,
            specular_resoultion as f32,
            specular_resoultion as f32,
            6,
            9,
            wgpu_format,
            wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        );

        // Create bind group
        let mut resource_manager = resources.get_mut::<GPUResourceManager>().unwrap();
        let bind_group_layout = resource_manager
            .get_bind_group_layout("probe_material_layout")
            .unwrap();

        let bind_group = BindGroup::new(
            3,
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Probe"),
                layout: &bind_group_layout,
                bindings: &[
                    wgpu::Binding {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(
                            &irradiance_target.texture_view,
                        ),
                    },
                    wgpu::Binding {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&specular_target.texture_view),
                    },
                    wgpu::Binding {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(&brdf_texture.texture_view),
                    },
                ],
            }),
        );
        resource_manager.add_single_bind_group("probe_material", bind_group);

        Self {
            id,
            position,
            format,
            has_rendered: false,
            irradiance_resoultion,
            irradiance_target,
            probe_cube: Arc::new(probe_cube),
            quality,
            sample_count,
            sample_offset,
            samples_per_frame,
            samples_remaining: 0,
            scale,
            specular_resoultion,
            specular_target,
            brdf_texture,
        }
    }

    // Render's scene to the cube
    // This is considered a very "HEAVY" operation, and shouldn't be treated lightly
    // TODO: If wgpu ever adds multi-view's use that instead..
    pub(crate) fn render_scene(
        &mut self,
        resources: &mut Resources,
        scene: &mut crate::scene::Scene,
    ) {
        // If we already rendered don't do it again.
        if self.has_rendered {
            return;
        }

        self.samples_remaining = self.sample_count;
        self.sample_offset = 0;

        // Create new render schedule has to be different from normal as we want to not queue items up right away.
        // TODO: Have more systems support our CurrentRenderTarget.
        let mut render_schedule =
            Schedule::builder() //create_render_schedule_builder()
                .add_system(crate::graphics::systems::skybox::create())
                .flush()
                .build();

        // First we need to create new pipelines using the correct texture format
        let current_pipelines;
        {
            let mut pipeline_manager = resources.get_mut::<PipelineManager>().unwrap();
            current_pipelines = pipeline_manager.current_pipelines.clone();
            let device = resources.get::<wgpu::Device>().unwrap();
            let asset_manager = resources.get::<AssetManager>().unwrap();
            let resource_manager = resources.get::<GPUResourceManager>().unwrap();
            let skybox_pipeline = pipeline_manager.get("skybox", None).unwrap();
            let mut new_skybox_desc = skybox_pipeline.desc.clone();
            new_skybox_desc.color_state.format = self.format.into();
            let hash = new_skybox_desc.create_hash();
            pipeline_manager.add_pipeline(
                "skybox",
                &new_skybox_desc,
                vec![],
                &device,
                &asset_manager,
                &resource_manager,
            );
            pipeline_manager.set_current_pipeline_hash("skybox", hash);
        }

        {
            let camera_query = <(Write<CameraData>,)>::query();
            for (mut camera_data,) in camera_query.iter_mut(&mut scene.world) {
                if camera_data.active {
                    camera_data.active = false;
                }
            }

            // Add our special camera to the scene.
            let probe_resoultion = self.quality.get_probe_resoultion();
            let camera = CameraData::new_perspective(
                90.0,
                probe_resoultion as f32,
                probe_resoultion as f32,
                0.01,
                1000.0,
            );
            scene.world.insert((), vec![(camera,)]);

            // Order of faces: X+ X- Y+ Y- Z+ Z-
            // Render scene to each face..
            for i in 0..6 {
                // TODO: cache views?
                let view = self
                    .probe_cube
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor {
                        format: self.format.into(),
                        dimension: wgpu::TextureViewDimension::D2,
                        aspect: wgpu::TextureAspect::default(),
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: i,
                        array_layer_count: 1,
                    });
                // Insert the cube as the current render target.
                resources.insert(CurrentRenderTarget(Some((self.probe_cube.clone(), view))));
                // Update camera with new view
                let camera_query = <(Write<CameraData>,)>::query();

                for (mut camera_data,) in camera_query.iter_mut(&mut scene.world) {
                    if camera_data.active {
                        Self::update_camera(self.position, &mut camera_data, i);
                    }
                }
                // Tell the render target the current face..
                render_schedule.execute(&mut scene.world, resources);
            }

            // Reset pipelines.
            {
                let mut pipeline_manager = resources.get_mut::<PipelineManager>().unwrap();
                pipeline_manager.current_pipelines = current_pipelines;
            }

            // Finally we submit our queue.
            let mut queue_schedule = Schedule::builder()
                .flush()
                .add_thread_local_fn(crate::graphics::systems::render::create())
                .build();
            queue_schedule.execute(&mut scene.world, resources);
        }

        // Remove camera_enttiy
        {
            let mut command = CommandBuffer::new(&scene.world);
            let camera_query = <(Read<CameraData>,)>::query();
            for (entity, camera_data) in camera_query.iter_entities_mut(&mut scene.world) {
                if camera_data.0.active {
                    command.remove_component::<CameraData>(entity);
                }
            }
            command.write(&mut scene.world);

            // And reactiveate all cameras deactived..
            // TODO: Figure out how to tell which camera was actually activated before this..
            let camera_query = <(Write<CameraData>,)>::query();
            for (mut camera_data,) in camera_query.iter_mut(&mut scene.world) {
                camera_data.active = true;
            }
        }

        // Generate mip maps for the resulting cube map
        let probe_resoultion = self.quality.get_probe_resoultion();
        let mut probe_cube = {
            let device = resources.get::<wgpu::Device>().unwrap();
            RenderTarget::new(
                &device,
                probe_resoultion as f32,
                probe_resoultion as f32,
                6,
                1,
                self.format.into(),
                wgpu::TextureUsage::SAMPLED
                    | wgpu::TextureUsage::COPY_SRC
                    | wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            )
        };
        probe_cube.texture = crate::graphics::pipelines::mipmap::create(
            resources,
            &self.probe_cube.texture,
            self.format.into(),
            wgpu::TextureDimension::D2,
            probe_resoultion,
            probe_resoultion,
            6,
        );
        probe_cube.texture_view = probe_cube
            .texture
            .create_view(&wgpu::TextureViewDescriptor {
                format: wgpu::TextureFormat::Rgba32Float,
                dimension: wgpu::TextureViewDimension::Cube,
                aspect: wgpu::TextureAspect::default(),
                base_mip_level: 0,
                level_count: 9,
                base_array_layer: 0,
                array_layer_count: 6,
            });
        self.probe_cube = Arc::new(probe_cube);

        resources.insert(CurrentRenderTarget(None));
    }

    pub(crate) fn render_brdf(
        &mut self,
        resources: &mut Resources,
        _scene: &mut crate::scene::Scene,
    ) {
        // If we already processed brdf do nothing.
        if self.samples_remaining == 0 {
            return;
        }

        self.render_irradiance(resources);

        self.render_specular(resources);

        self.samples_remaining -= self.samples_per_frame;
        if self.samples_remaining > 0 {
            self.sample_offset += 1;
        }
    }

    fn render_irradiance(&mut self, resources: &Resources) {
        let device = resources.get::<wgpu::Device>().unwrap();
        let asset_manager = resources.get::<AssetManager>().unwrap();
        let sc_desc = resources.get::<wgpu::SwapChainDescriptor>().unwrap();
        let mut resource_manager = resources.get_mut::<GPUResourceManager>().unwrap();
        let mut render_graph = resources.get_mut::<RenderGraph>().unwrap();

        // create pipeline if we need to.
        let graph_node = &render_graph.get_safe("irradiance2");
        if graph_node.is_none() {
            let pipeline_desc =
                crate::graphics::pipelines::irradiance2::IrradiancePipelineDesc::default();
            render_graph.add(
                &asset_manager,
                &device,
                &sc_desc,
                &mut resource_manager,
                "irradiance2",
                pipeline_desc,
                vec![],
                false,
                None,
                false,
            );
        }

        let pipeline = &render_graph.get("irradiance2").pipeline;

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("irradiance"),
        });

        let global_bind_group = resource_manager
            .get_bind_group_layout("irradiance2")
            .unwrap();

        let output = RenderTarget::new(
            &device,
            self.irradiance_resoultion as f32,
            self.irradiance_resoultion as f32 * 6.0,
            1,
            1,
            self.format.into(),
            wgpu::TextureUsage::COPY_SRC | wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        );

        let uniform = ProbeUniform {
            data: Vec4::new(
                self.sample_offset as f32,
                self.samples_per_frame as f32,
                self.sample_count as f32,
                self.scale,
            ),
            data2: Vec4::new(
                self.irradiance_resoultion as f32,
                self.irradiance_resoultion as f32,
                0.0,
                0.0,
            ),
        };
        let uniform_size = std::mem::size_of::<ProbeUniform>() as wgpu::BufferAddress;
        let uniform_buf = device.create_buffer_with_data(
            bytemuck::bytes_of(&uniform),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: global_bind_group,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &uniform_buf,
                        range: 0..uniform_size,
                    },
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&self.probe_cube.texture_view),
                },
                wgpu::Binding {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(
                        &self.irradiance_target.texture_view,
                    ),
                },
                wgpu::Binding {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&self.probe_cube.sampler),
                },
            ],
            label: None,
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &output.texture_view,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Clear,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    },
                }],
                depth_stencil_attachment: None,
            });
            render_pass.set_pipeline(pipeline);
            render_pass.set_bind_group(0, &bind_group, &[]);
            render_pass.draw(0..6, 0..6);
        }

        for i in 0..6 {
            encoder.copy_texture_to_texture(
                wgpu::TextureCopyView {
                    texture: &output.texture,
                    mip_level: 0,
                    array_layer: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: self.irradiance_resoultion * i,
                        z: 0,
                    },
                },
                wgpu::TextureCopyView {
                    texture: &self.irradiance_target.texture,
                    mip_level: 0,
                    array_layer: i,
                    origin: wgpu::Origin3d::ZERO,
                },
                wgpu::Extent3d {
                    width: self.irradiance_resoultion as u32,
                    height: self.irradiance_resoultion as u32,
                    depth: 1,
                },
            );
        }

        let queue = resources.get::<wgpu::Queue>().unwrap();
        queue.submit(Some(encoder.finish()));
    }

    fn render_specular(&mut self, resources: &Resources) {
        let device = resources.get::<wgpu::Device>().unwrap();
        let resource_manager = resources.get_mut::<GPUResourceManager>().unwrap();
        let pipeline_manager = resources.get_mut::<PipelineManager>().unwrap();
        let mip_levels: u32 = 9;

        // create pipeline if we need to.
        let pipeline = pipeline_manager.get("specular", None).unwrap();

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("specular"),
        });

        let global_bind_group = resource_manager
            .get_bind_group_layout("specular_globals")
            .unwrap();

        let output = RenderTarget::new(
            &device,
            self.specular_resoultion as f32,
            self.specular_resoultion as f32 * 6.0,
            1,
            mip_levels,
            self.format.into(),
            wgpu::TextureUsage::COPY_SRC | wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        );
        let buffer_size = std::mem::size_of::<ProbeUniform>() as u64;

        let buffer = resource_manager.get_buffer("specular");

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: global_bind_group,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &buffer,
                        range: 0..buffer_size,
                    },
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&self.probe_cube.texture_view),
                },
                wgpu::Binding {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(
                        &self.specular_target.texture_view,
                    ),
                },
                wgpu::Binding {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&self.probe_cube.sampler),
                },
            ],
            label: None,
        });

        let mut roughness: f32 = 0.0;
        let roughness_delta = 1.0 / (mip_levels as f32 - 1.0);

        for mip_id in 0..mip_levels {
            let uniform = ProbeUniform {
                data: Vec4::new(
                    self.sample_offset as f32,
                    self.samples_per_frame as f32,
                    self.sample_count as f32,
                    self.scale,
                ),
                data2: Vec4::new(
                    self.specular_resoultion as f32,
                    self.specular_resoultion as f32,
                    roughness,
                    mip_id as f32,
                ),
            };
            let uniform_buf = device.create_buffer_with_data(
                bytemuck::bytes_of(&uniform),
                wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_SRC,
            );

            encoder.copy_buffer_to_buffer(&uniform_buf, 0, &buffer, 0, buffer_size);

            let new_view = output.texture.create_view(&wgpu::TextureViewDescriptor {
                format: self.format.into(),
                dimension: wgpu::TextureViewDimension::D2,
                aspect: wgpu::TextureAspect::default(),
                base_mip_level: mip_id,
                level_count: 1,
                base_array_layer: 0,
                array_layer_count: 1,
            });

            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &new_view,
                        resolve_target: None,
                        load_op: wgpu::LoadOp::Clear,
                        store_op: wgpu::StoreOp::Store,
                        clear_color: wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        },
                    }],
                    depth_stencil_attachment: None,
                });
                render_pass.set_pipeline(&pipeline.render_pipeline);
                render_pass.set_bind_group(0, &bind_group, &[]);
                render_pass.draw(0..6, 0..6);
            }

            roughness += roughness_delta;
        }

        for mip_id in 0..mip_levels {
            let res = (self.specular_resoultion / 2u32.pow(mip_id)) as f32;
            for i in 0..6 {
                encoder.copy_texture_to_texture(
                    wgpu::TextureCopyView {
                        texture: &output.texture,
                        mip_level: mip_id,
                        array_layer: 0,
                        origin: wgpu::Origin3d {
                            x: 0,
                            y: res as u32 * i,
                            z: 0,
                        },
                    },
                    wgpu::TextureCopyView {
                        texture: &self.specular_target.texture,
                        mip_level: mip_id,
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

        let queue = resources.get::<wgpu::Queue>().unwrap();
        queue.submit(Some(encoder.finish()));
    }

    fn update_camera(position: Vec3, camera: &mut CameraData, face_id: u32) {
        let mut eye = Vec3::zeros();
        let mut up = Vec3::new(0.0, -1.0, 0.0);
        match face_id {
            0 => {
                eye = Vec3::new(-1.0, 0.0, 0.0);
            } // X+
            1 => {
                eye = Vec3::new(1.0, 0.0, 0.0);
            } // X-
            2 => {
                eye = Vec3::new(0.0, -1.0, 0.0);
                up = Vec3::new(0.0, 0.0, 1.0);
            } // Y+
            3 => {
                eye = Vec3::new(0.0, 1.0, 0.0);
                up = Vec3::new(0.0, 0.0, -1.0);
            } // Y-
            4 => {
                eye = Vec3::new(0.0, 0.0, -1.0);
            } // Z+
            5 => {
                eye = Vec3::new(0.0, 0.0, 1.0);
            } // Z-
            _ => (),
        }
        camera.update_view(eye, position, up);
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ProbeUniform {
    // (ConvolutionSamplesOffset, ConvolutionSampleCount, ConvolutionMaxSamples, _)
    data: Vec4,
    // (width, height, ConvolutionRoughness, ConvolutionMip)
    data2: Vec4,
}

impl Default for ProbeUniform {
    fn default() -> Self {
        Self {
            data: Vec4::zeros(),
            data2: Vec4::zeros(),
        }
    }
}

unsafe impl Zeroable for ProbeUniform {}
unsafe impl Pod for ProbeUniform {}
