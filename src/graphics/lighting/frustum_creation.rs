use std::sync::Arc;
use nalgebra_glm::{Mat4, Vec2};
use bytemuck::{Pod, Zeroable};
use crate::{AssetManager, graphics::{resources::{GPUResourceManager}, pipeline_manager::{PipelineManager, ComputePipelineDesc}}, core::{Frustum, GpuFrustum}};
use super::cluster::{FROXELS_Y, FROXELS_X};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct FroxelUniform {
    frustum: GpuFrustum,
    i_proj: Mat4,
    frustum_count: [u32; 4],
}

unsafe impl Zeroable for FroxelUniform { }
unsafe impl Pod for FroxelUniform { }

pub struct FrustumCreation {
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    gpu_resource_manager: Arc<GPUResourceManager>,
    frustum_count: Vec2,
}

impl FrustumCreation {
    pub fn new(
        asset_manager: &AssetManager,
        gpu_resource_manager: Arc<GPUResourceManager>,
        pipeline_manager: &mut PipelineManager,
        device: Arc<wgpu::Device>,
        frustum_buffer: &wgpu::Buffer,
        frustum_count: Vec2,
    ) -> Self {
        let uniform = FroxelUniform {
            frustum: Frustum::new().into(),
            frustum_count: [frustum_count.x as u32, frustum_count.y as u32, 0, 0],
            i_proj: Mat4::identity(),
        };

        let uniform_buffer = device.create_buffer_with_data(bytemuck::bytes_of(&uniform), wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST);


        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            bindings: &[
                wgpu::BindGroupLayoutEntry::new(0, wgpu::ShaderStage::COMPUTE, wgpu::BindingType::UniformBuffer {
                    dynamic: false,
                    min_binding_size: None,
                }),
                wgpu::BindGroupLayoutEntry::new(1, wgpu::ShaderStage::COMPUTE, wgpu::BindingType::StorageBuffer {
                    readonly: false,
                    dynamic: false,
                    min_binding_size: None,
                }),
            ],
            label: Some("froxel layout"),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(uniform_buffer.slice(..)),
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(frustum_buffer.slice(..)),
                },
            ],
            label: Some("froxel bindings"),
        });

        gpu_resource_manager.add_bind_group_layout("froxel_layout", bind_group_layout);

        let mut pipeline_desc = ComputePipelineDesc::new("core/shaders/clustered/froxels.shader");
        pipeline_desc.layouts = vec!["froxel_layout".to_string()];

        pipeline_manager.add_compute_pipeline("froxel_creation", &pipeline_desc, vec![], &device, asset_manager, gpu_resource_manager.clone());

        Self {
            uniform_buffer,
            bind_group,
            gpu_resource_manager,
            frustum_count,
        }
    }

    pub fn resize(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        device: Arc<wgpu::Device>,
        frustum: Frustum,
        i_proj: Mat4,
    ) {

        let uniform = FroxelUniform {
            frustum: frustum.into(),
            frustum_count: [self.frustum_count.x as u32, self.frustum_count.y as u32, 0, 0],
            i_proj,
        };

        let uniform_staging_buffer = device.create_buffer_with_data(bytemuck::bytes_of(&uniform), wgpu::BufferUsage::COPY_SRC);

        encoder.copy_buffer_to_buffer(
            &uniform_staging_buffer,
            0,
            &self.uniform_buffer,
            0,
            std::mem::size_of::<FroxelUniform>() as wgpu::BufferAddress,
        );
    }

    pub fn compute<'a>(&'a self, pipeline_manager: &'a PipelineManager, pass: &mut wgpu::ComputePass<'a>) {
        let pipeline = pipeline_manager.get_compute("froxel_creation", None).unwrap();
        pass.set_pipeline(&pipeline.compute_pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.dispatch(FROXELS_X / 8, FROXELS_Y / 8, 1);
    }
}