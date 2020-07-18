use bytemuck::{Pod, Zeroable};
use nalgebra_glm::{Mat4, Vec2};
use crate::{graphics::{resources::GPUResourceManager, pipeline_manager::PipelineManager}, core::{Frustum, GpuFrustum}, AssetManager};
use std::sync::Arc;
use super::{cull::LightCulling, frustum_creation::FrustumCreation};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct ClusterUniforms {
    froxel_count: [u32; 4],
}

pub const FAR_PLANE_DISTANCE: f32 = 33.0 /* blocks */ * 64.0 /* m */;
pub const FROXELS_X: u32 = 16;
pub const FROXELS_Y: u32 = 16;
pub const FROXELS_Z: u32 = 32;
pub const FROXEL_COUNT: u32 = FROXELS_X * FROXELS_Y * FROXELS_Z;

const FRUSTUM_COUNT: u32 = FROXELS_X * FROXELS_Y;
const FRUSTUM_BUFFER_SIZE: wgpu::BufferAddress = (FRUSTUM_COUNT * std::mem::size_of::<GpuFrustum>() as u32) as wgpu::BufferAddress;

const MAX_LIGHTS_PER_FROXEL: u32 = 128;
const LIGHT_LIST_BUFFER_SIZE: wgpu::BufferAddress = (FROXEL_COUNT * MAX_LIGHTS_PER_FROXEL * std::mem::size_of::<u32>() as u32) as wgpu::BufferAddress;

unsafe impl Zeroable for ClusterUniforms { }
unsafe impl Pod for ClusterUniforms { }

pub struct Clustering {
    light_list_buffer: wgpu::Buffer,
    frustum_buffer: wgpu::Buffer,
    frustum_creation: FrustumCreation,
    light_culling: LightCulling,
}

impl Clustering {
    pub fn new(device: Arc<wgpu::Device>, gpu_resource_manager: Arc<GPUResourceManager>, pipeline_manager: &mut PipelineManager, asset_manager: &AssetManager) -> Self {
        let frustum_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            usage: wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::STORAGE,
            size: FRUSTUM_BUFFER_SIZE,
            mapped_at_creation: false,
            label: Some("frustum buffer"),
        });

        let frustum_creation = FrustumCreation::new(
            asset_manager,
            gpu_resource_manager.clone(),
            pipeline_manager,
            device.clone(),
            &frustum_buffer,
            Vec2::new(FROXELS_X as f32, FROXELS_Y as f32),
        );

        let light_list_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            usage: wgpu::BufferUsage::STORAGE,
            size: LIGHT_LIST_BUFFER_SIZE,
            mapped_at_creation: false,
            label: Some("light list buffer"),
        });

        let light_culling = LightCulling::new(
            device,
            gpu_resource_manager.clone(),
            pipeline_manager,
            asset_manager,
            &frustum_buffer,
            &light_list_buffer
        );

        Self {
            light_list_buffer,
            frustum_creation,
            frustum_buffer,
            light_culling,
        }
    }

    pub fn resize(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        device: Arc<wgpu::Device>,
        frustum: Frustum,
        i_proj: Mat4,
    ) {
        self.frustum_creation.resize(encoder, device, frustum, i_proj);
    }

    pub fn compute(&mut self, encoder: &mut wgpu::CommandEncoder, pipeline_manager: &PipelineManager) {
        let mut pass = encoder.begin_compute_pass();
        self.frustum_creation.compute(pipeline_manager, &mut pass);

        self.light_culling.compute(pipeline_manager, &mut pass);
    }
}