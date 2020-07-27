use legion::prelude::Resources;

use crate::{
    graphics::{
        pipeline_manager::{PipelineDesc, PipelineManager},
        resources::GPUResourceManager,
    },
    AssetManager,
};
use std::{borrow::Cow, sync::Arc};

// mipmaps always run pretty much right away.
pub fn create(
    resources: &Resources,
    original_texture: &wgpu::Texture,
    format: wgpu::TextureFormat,
    dimension: wgpu::TextureDimension,
    width: u32,
    height: u32,
    depth: u32,
) -> wgpu::Texture {
    let asset_manager = resources.get_mut::<AssetManager>().unwrap();
    let mut pipeline_manager = resources.get_mut::<PipelineManager>().unwrap();
    let resource_manager = resources.get::<Arc<GPUResourceManager>>().unwrap();
    let device = resources.get::<Arc<wgpu::Device>>().unwrap();

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("mipmap"),
    });

    //1 + floor(log2(max(w, h, d)))
    let mip_map_count = (width.max(height) as f32).log2().floor() as u32;

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width,
            height,
            depth,
        },
        mip_level_count: mip_map_count,
        sample_count: 1,
        dimension,
        format,
        usage: wgpu::TextureUsage::SAMPLED
            | wgpu::TextureUsage::OUTPUT_ATTACHMENT
            | wgpu::TextureUsage::COPY_DST,
    });

    let mut bind_group_layout = resource_manager.get_bind_group_layout("mipmap");

    // Create bind group layout and bind group for passing in texture to mip map shader.
    if bind_group_layout.is_none() {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(Cow::Borrowed("mipmap")),
            entries: Cow::Borrowed(&[
                wgpu::BindGroupLayoutEntry::new(
                    0,
                    wgpu::ShaderStage::FRAGMENT,
                    wgpu::BindingType::SampledTexture {
                        component_type: wgpu::TextureComponentType::Float,
                        multisampled: false,
                        dimension: wgpu::TextureViewDimension::D2,
                    },
                ),
                wgpu::BindGroupLayoutEntry::new(
                    1,
                    wgpu::ShaderStage::FRAGMENT,
                    wgpu::BindingType::Sampler { comparison: false },
                ),
            ]),
        });
        resource_manager.add_bind_group_layout("mipmap", layout);
        bind_group_layout = resource_manager.get_bind_group_layout("mipmap");
    }

    let mut pipeline = pipeline_manager.get("mipmap", None);

    if pipeline.is_none() {
        let mut mipmap_desc = PipelineDesc::default();
        mipmap_desc.shader = "core/shaders/calculations/mipmap.shader".to_string();
        mipmap_desc.color_states[0].format = format;
        mipmap_desc.cull_mode = wgpu::CullMode::None;
        mipmap_desc.layouts = vec!["mipmap".to_string()];
        pipeline_manager.add_pipeline(
            "mipmap",
            &mipmap_desc,
            vec![],
            &device,
            &asset_manager,
            resource_manager.clone(),
        );
        pipeline = pipeline_manager.get("mipmap", None);
    }

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

    for face_id in 0..depth {
        for mip_id in 0..mip_map_count {
            let view = original_texture.create_view(&wgpu::TextureViewDescriptor {
                label: None,
                format,
                dimension: wgpu::TextureViewDimension::D2,
                aspect: wgpu::TextureAspect::default(),
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: face_id,
                array_layer_count: 1,
            });

            let new_view = texture.create_view(&wgpu::TextureViewDescriptor {
                label: None,
                format,
                dimension: wgpu::TextureViewDimension::D2,
                aspect: wgpu::TextureAspect::default(),
                base_mip_level: mip_id,
                level_count: 1,
                base_array_layer: face_id,
                array_layer_count: 1,
            });

            // Create a bind group. In this case the bind group is new every time for mip maps.
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(Cow::Borrowed("mipmap")),
                layout: bind_group_layout.as_ref().unwrap(),
                entries: Cow::Borrowed(&[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ]),
            });

            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: Cow::Borrowed(&[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &new_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                        store: true,
                    },
                }]),
                depth_stencil_attachment: None,
            });
            rpass.set_pipeline(&pipeline.as_ref().unwrap().render_pipeline);
            rpass.set_bind_group(0, &bind_group, &[]);
            rpass.draw(0..4, 0..1);
        }
    }

    let queue = resources.get::<Arc<wgpu::Queue>>().unwrap();
    queue.submit(Some(encoder.finish()));

    // device.poll(wgpu::Maintain::Wait);

    texture
}
