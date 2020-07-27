use std::{borrow::Cow, sync::Arc};
use crate::{AssetManager, graphics::{pipeline_manager::{ComputePipelineDesc, PipelineManager}, resources::GPUResourceManager}};
use super::cluster::{FROXEL_COUNT};

pub struct LightCulling {
    gpu_resource_manager: Arc<GPUResourceManager>,
    bind_group: wgpu::BindGroup,
}

impl LightCulling {
    pub fn new(
        device: Arc<wgpu::Device>,
        gpu_resource_manager: Arc<GPUResourceManager>,
        pipeline_manager: &mut PipelineManager,
        asset_manager: &AssetManager,
    ) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: Cow::Borrowed(&[
                wgpu::BindGroupLayoutEntry::new(0, wgpu::ShaderStage::COMPUTE, wgpu::BindingType::StorageBuffer {
                    dynamic: false,
                    readonly: true,
                    min_binding_size: None,
                }),
                wgpu::BindGroupLayoutEntry::new(1, wgpu::ShaderStage::COMPUTE, wgpu::BindingType::StorageBuffer {
                    dynamic: false,
                    readonly: false,
                    min_binding_size: None,
                }),
            ]),
            label: Some(Cow::Borrowed("light culling layout")),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: Cow::Borrowed(&[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(gpu_resource_manager.frustum_buffer.slice(..)),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(gpu_resource_manager.light_list_buffer.slice(..)),
                },
            ]),
            label: Some(Cow::Borrowed("light culling bind group")),
        });

        gpu_resource_manager.add_bind_group_layout("froxel_cull_layout", bind_group_layout);

        let mut pipeline_desc = ComputePipelineDesc::new("core/shaders/clustered/light_culling.shader");
        pipeline_desc.layouts = vec!["froxel_cull_layout".to_string(), "globals".to_string()];

        pipeline_manager.add_compute_pipeline("froxel_cull", &pipeline_desc, vec!["globals"], &device, asset_manager, gpu_resource_manager.clone());

        Self {
            gpu_resource_manager,
            bind_group
        }
    }

    pub fn compute<'a>(&'a self, pipeline_manager: &'a PipelineManager, pass: &mut wgpu::ComputePass<'a>) {
        let pipeline = pipeline_manager.get_compute("froxel_cull", None).unwrap();
        pass.set_pipeline(&pipeline.compute_pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_bind_group(1, &self.gpu_resource_manager.global_bind_group, &[]);
        pass.dispatch(1, FROXEL_COUNT / 64, 1);
    }
}