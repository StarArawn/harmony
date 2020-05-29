use nalgebra_glm::Vec3;

use crate::{
    graphics::{
        resources::{GPUResourceManager},
        pipeline_manager::PipelineManager,
    },
    Application, AssetManager,
};

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
    pub is_processed: bool,
    pub(crate) texture: Option<String>,
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
        Self {
            size,
            color_texture: None,
            color_view: None,
            cubemap_sampler: None,
            cubemap_bind_group: None,
            pbr_bind_group: None,
            clear_color: Vec3::zeros(),
            skybox_type: SkyboxType::HdrCubemap,
            texture: Some(texture.into()),
            is_processed: false,
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
            texture: None,
            is_processed: true,
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
            texture: None,
            is_processed: true,
        }
    }

    pub fn update_cubemap(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        asset_manager: &AssetManager,
        //image_asset_manager: &ImageAssetManager,
        pipeline_manager: &mut PipelineManager,
        resource_manager: &mut GPUResourceManager,
    ) {
//        if self.is_processed { return; }
//
//        let texture = image_asset_manager.get(self.texture.clone().unwrap());
//
//        if texture.is_some() {
//            log::info!("Creating skybox!");
//            self.is_processed = true;
//            let texture = texture.as_ref().unwrap();
//            assert!(texture.format == wgpu::TextureFormat::Rgba32Float);
//
//            let color = crate::graphics::pipelines::equirectangular2::create(
//                device,
//                queue,
//                asset_manager,
//                pipeline_manager,
//                resource_manager,
//                texture,
//                self.size,
//            );
//
//            let color_view = color.texture.create_view(&wgpu::TextureViewDescriptor {
//                label: None,
//                format: wgpu::TextureFormat::Rgba32Float,
//                dimension: wgpu::TextureViewDimension::Cube,
//                aspect: wgpu::TextureAspect::default(),
//                base_mip_level: 0,
//                level_count: 1,
//                base_array_layer: 0,
//                array_layer_count: 6,
//            });
//
//            let cubemap_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
//                label: None,
//                address_mode_u: wgpu::AddressMode::ClampToEdge,
//                address_mode_v: wgpu::AddressMode::ClampToEdge,
//                address_mode_w: wgpu::AddressMode::ClampToEdge,
//                mag_filter: wgpu::FilterMode::Nearest,
//                min_filter: wgpu::FilterMode::Nearest,
//                mipmap_filter: wgpu::FilterMode::Nearest,
//                lod_min_clamp: -100.0,
//                lod_max_clamp: 100.0,
//                compare: wgpu::CompareFunction::Undefined,
//            });
//
//            self.color_view = Some(color_view);
//            self.cubemap_sampler = Some(cubemap_sampler);
//
//            let bind_group_layout = resource_manager.get_bind_group_layout("skybox_material").unwrap();
//            self.create_bind_group2(device, bind_group_layout);
//        }
    }

    pub(crate) fn create_realtime_bind_group(
        &mut self,
        device: &wgpu::Device,
        asset_manager: &AssetManager,
        material_layout: &wgpu::BindGroupLayout,
    ) {
        let rayleigh_texture = asset_manager.get_image("rayleigh.hdr");
        let mie_texture = asset_manager.get_image("mie.hdr");

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
            compare: wgpu::CompareFunction::Undefined,
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
        material_layout: &wgpu::BindGroupLayout,
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
