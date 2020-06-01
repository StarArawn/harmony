use legion::prelude::Resources;

use crate::{
    graphics::{
        material::Image,
        pipeline_manager::{PipelineDesc, PipelineManager},
        renderer::DEPTH_FORMAT,
        resources::{GPUResourceManager, RenderTarget},
    },
    AssetManager,
};
use std::sync::Arc;

pub fn create(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    asset_manager: &AssetManager,
    pipeline_manager: &mut PipelineManager,
    resource_manager: &mut GPUResourceManager,
    image: &Arc<Image>,
    size: f32,
) -> RenderTarget {
    let mut pipeline = pipeline_manager.get("cubemap", None);

    let pipeline = if pipeline.is_none() {
        let mut cubemap_pipeline = PipelineDesc::default();
        cubemap_pipeline.shader = "hdr_to_cubemap.shader".to_string();
        cubemap_pipeline.depth_state = None;

        let cubemap_material_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                bindings: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::SampledTexture {
                            component_type: wgpu::TextureComponentType::Float,
                            multisampled: false,
                            dimension: wgpu::TextureViewDimension::Cube,
                        },
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler { comparison: false },
                    },
                ],
                label: None,
            });
        resource_manager.add_bind_group_layout("equirectangular_globals", cubemap_material_layout);

        cubemap_pipeline.layouts = vec!["equirectangular_globals".to_string()];
        cubemap_pipeline.cull_mode = wgpu::CullMode::None;
        cubemap_pipeline.front_face = wgpu::FrontFace::Cw;
        cubemap_pipeline.color_state.format = wgpu::TextureFormat::Rgba32Float;
        cubemap_pipeline
            .vertex_state
            .set_index_format(wgpu::IndexFormat::Uint16);

        pipeline_manager.add_pipeline(
            "cubemap",
            &cubemap_pipeline,
            vec![],
            device,
            asset_manager,
            resource_manager,
        );

        pipeline_manager.get("cubemap", None).unwrap()
    } else {
        pipeline.unwrap()
    };

    let cube_map_target = RenderTarget::new(
        &device,
        size,
        size * 6.0,
        1,
        1,
        wgpu::TextureFormat::Rgba32Float,
        wgpu::TextureUsage::COPY_SRC | wgpu::TextureUsage::OUTPUT_ATTACHMENT,
    );

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("cubemap"),
    });

    let global_bind_group = resource_manager
        .get_bind_group_layout("equirectangular_globals")
        .unwrap();

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: global_bind_group,
        bindings: &[
            // wgpu::Binding {
            //     binding: 0,
            //     resource: wgpu::BindingResource::TextureView(&image.view),
            // },
            // wgpu::Binding {
            //     binding: 1,
            //     resource: wgpu::BindingResource::Sampler(&image.sampler),
            // },
        ],
        label: None,
    });

    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &cube_map_target.texture_view,
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

    let cube_map = RenderTarget::new(
        &device,
        size,
        size,
        6,
        1,
        wgpu::TextureFormat::Rgba32Float,
        wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
    );

    for i in 0..6 {
        encoder.copy_texture_to_texture(
            wgpu::TextureCopyView {
                texture: &cube_map_target.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: 0,
                    y: size as u32 * i,
                    z: 0,
                },
            },
            wgpu::TextureCopyView {
                texture: &cube_map.texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x: 0, y: 0, z: i },
            },
            wgpu::Extent3d {
                width: size as u32,
                height: size as u32,
                depth: 1,
            },
        );
    }

    // Push to all command buffers to the queue
    queue.submit(Some(encoder.finish()));

    cube_map
}
