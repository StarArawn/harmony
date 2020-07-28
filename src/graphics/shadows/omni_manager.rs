use nalgebra_glm::{Vec3, Vec2, Vec4, Mat4};
use std::{sync::Arc, borrow::Cow};
use crate::{core::BoundingSphere, scene::components, graphics::{resources::{ArcRenderPass, GPUResourceManager}, pipeline_manager::{PipelineDesc, PipelineManager}}, AssetManager, assets::mesh::MeshVertexData};
use legion::{systems::{SubWorld, SystemQuery}, prelude::*, filter::{And, EntityFilterTuple, ComponentFilter, Passthrough}};
use bytemuck::{Zeroable, Pod};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ShadowPush {
    pub matrix: Mat4,
    pub light_pos: Vec4,
}

unsafe impl Pod for ShadowPush { }
unsafe impl Zeroable for ShadowPush { }

#[derive(Clone)]
pub struct ShadowTexture {
    texture: Arc<wgpu::Texture>,
    pub(crate) view: Arc<wgpu::TextureView>,
    pub(crate) face_views: Vec<Vec<Arc<wgpu::TextureView>>>,
    pub(crate) free: u32,
    pub(crate) size: u32,
    pub(crate) tex_size: u32,
}

pub struct OmniShadowManager {
    pub(crate) quality: ShadowQuality,
    pub(crate) quad_textures: Vec<ShadowTexture>,
    pub(crate) sampler: Arc<wgpu::Sampler>,
    pub(crate) max_casters_per_frame: u32,
}

pub enum ShadowQuality {
    Low,
    Medium,
    High,
}

// Type alias
pub type ShadowCamera = components::CameraData;

impl OmniShadowManager {
    pub fn new(device: Arc<wgpu::Device>, quality: ShadowQuality) -> Self {
        let size = match quality {
            ShadowQuality::Low => Vec2::new(2048.0, 2048.0),
            ShadowQuality::Medium => Vec2::new(4096.0, 4096.0),
            ShadowQuality::High => Vec2::new(8192.0, 8192.0),
        };

        // let (quad1, quad2,  quad3, quad4) = Self::calculate_quadrants(size);

        let half_size = size / 2.0;

        // Four quad textures
        let mut quad_textures = Vec::new();
        for i in 1..5 {
            let division = 2u32.pow(i + 1);

            // This might result in more memory consumption then what we want..
            let texture_count = ((division * division) as f32 / 6.0).ceil() as u32;

            let tex_size = half_size.x as u32 / division; 

            let texture = Arc::new(device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Shadow map atlas"),
                size: wgpu::Extent3d {
                    width: tex_size,
                    height: tex_size,
                    depth: texture_count * 6, // 6 faces in a cube. 3 textures.
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            }));

            let view = Arc::new(texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some(&format!("Cube Shadow Texture: {}", i)),
                format: wgpu::TextureFormat::Depth32Float,
                dimension: wgpu::TextureViewDimension::CubeArray,
                aspect: wgpu::TextureAspect::All,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                array_layer_count: texture_count * 6,
            }));

            let mut face_views = Vec::new();

            for tex_id in 0..texture_count {
                let tex_index = tex_id * 6;

                let mut inner_face_views = Vec::new();
                for face in 0..6 {
                    let view = Arc::new(texture.create_view(&wgpu::TextureViewDescriptor {
                        label: Some(&format!("cube_shadow_texture_{:?}", face)),
                        format: wgpu::TextureFormat::Depth32Float,
                        dimension: wgpu::TextureViewDimension::D2,
                        aspect: wgpu::TextureAspect::All,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: tex_index + face,
                        array_layer_count: 1,
                    }));
                    inner_face_views.push(view);
                }
                face_views.push(inner_face_views);
            }

            quad_textures.push(ShadowTexture {
                texture,
                view,
                face_views,
                free: 0,
                size: texture_count,
                tex_size,
            });
        }

        let sampler = Arc::new(device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("cube_map_shadow_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        }));
        
        Self {
            quality,
            quad_textures,
            sampler,
            max_casters_per_frame: 5,
        }
    }

    pub fn create_pipeline(device: Arc<wgpu::Device>, asset_manager: &AssetManager, gpu_resource_manager: Arc<GPUResourceManager>, pipeline_manager: &mut PipelineManager) {
        // Create pipeline..
        let mut pipeline_desc = PipelineDesc::default();
        pipeline_desc.shader = "core/shaders/shadow.shader".to_string();
        pipeline_desc.color_states = vec![]; // Clear out color states.
        pipeline_desc.layouts = vec!["locals".to_string()];
        pipeline_desc.depth_state = Some(wgpu::DepthStencilStateDescriptor {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil_front: wgpu::StencilStateFaceDescriptor::IGNORE,
            stencil_back: wgpu::StencilStateFaceDescriptor::IGNORE,
            stencil_read_mask: 0,
            stencil_write_mask: 0,
        });
        pipeline_desc.push_constant_ranges = vec![
            wgpu::PushConstantRange {
                stages: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                range: 0..80,
            },
        ];
        pipeline_desc.depth_bias = 2;
        pipeline_desc.depth_bias_slope_scale = 2.0.into();
        pipeline_desc.depth_bias_clamp = 0.0.into();
        let vertex_size = std::mem::size_of::<MeshVertexData>();
        pipeline_desc
            .vertex_state
            .set_index_format(wgpu::IndexFormat::Uint32)
            .new_buffer_descriptor(
                vertex_size as wgpu::BufferAddress,
                wgpu::InputStepMode::Vertex,
                wgpu::vertex_attr_array![0 => Float3, 1 => Float3, 2 => Float2, 3 => Float4].to_vec(),
            );

        pipeline_manager.add_pipeline("shadow", &pipeline_desc, vec![], &device, asset_manager, gpu_resource_manager);
    }

    pub fn update(&mut self,
        sorted_point_lights: Vec<(f32, Vec3, (u32, u32))>, // (light attenuation, world position)
        pipeline_manager: &PipelineManager,
        resource_manager: Arc<GPUResourceManager>,
        encoder: &mut wgpu::CommandEncoder,
        shadow_camera: &mut ShadowCamera,
        mesh_query: &mut SystemQuery<(Read<components::Mesh>, Read<components::Transform>), EntityFilterTuple<And<(ComponentFilter<components::Mesh>, ComponentFilter<components::Transform>)>, And<(Passthrough, Passthrough)>, And<(Passthrough, Passthrough)>>>,
        world: &mut SubWorld
    ) {
        let pipeline = pipeline_manager.get("shadow", None).unwrap();
    
        self.reset_used();

        let mut total = 0;
        for (light_range, pos, texture_coords) in sorted_point_lights {
            // Step 1: reserve space this frame for the point light to render in.
            // Figures out which quad/texture to use.
            // let (quad_id, inner_texture_id) = self.get_cube_coords();
            let current_quad = &self.quad_textures[texture_coords.0 as usize];

            if total >= self.max_casters_per_frame {
                // We only render X number of shadow casters a frame.
                break;
            }

            // Find meshes within light radius.
            let mut light_bounds = BoundingSphere::new();
            light_bounds.center = pos;
            light_bounds.radius = light_range;

            let meshes = mesh_query
                .iter(world)
                .filter(|(mesh, transform)| {
                    let mesh_data = mesh.mesh_handle.get();
                    
                    if mesh_data.is_err() || transform.cull {
                        return false;
                    }
                    
                    let mesh = mesh_data.unwrap();
                    let mut bounding_sphere = mesh.bounding_sphere.clone();
                    bounding_sphere.center = (transform.matrix * Vec4::new(bounding_sphere.center.x, bounding_sphere.center.y, bounding_sphere.center.z, 1.0)).xyz();
                    return bounding_sphere.intersects_sphere(&light_bounds);
                })
                .map(|(mesh, transform)| {
                    let mesh = mesh.mesh_handle.get().unwrap();
                    // Arc<Mesh> hard transform on clone.
                    // TODO: Figure out performance impacts of cloning here..
                    (mesh.clone(), transform.clone())
                })
                .collect::<Vec<_>>();
            
            for face in 0..6 {
                // Get view for the current face.
                let face_view = &current_quad.face_views[texture_coords.1 as usize][face];

                // Start render pass.
                let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: Cow::Borrowed(&[]),
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                        attachment: &face_view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: true,
                        }),
                        stencil_ops: None,
                    }),
                });
                let arena1 = typed_arena::Arena::new();
                let arena2 = typed_arena::Arena::new();

                let mut render_pass = ArcRenderPass::new(&arena1, &arena2, render_pass);
                render_pass.set_pipeline(pipeline);

                shadow_camera.resize_range(current_quad.tex_size as f32, current_quad.tex_size as f32, 0.1, light_range);
                shadow_camera.set_cubic_camera(pos, face as u32);
                render_pass.set_push_constants(wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT, 0, bytemuck::cast_slice(&[ShadowPush {
                    matrix: shadow_camera.get_matrix(),
                    light_pos: Vec4::new(pos.x, pos.y, pos.z, 1.0),
                }]));

                // Step 2: Render shadow maps to that space.
                for (asset_mesh, transform) in meshes.iter() {
                    resource_manager.set_multi_bind_group(
                        &mut render_pass,
                        "transform",
                        0,
                        transform.index,
                    );

                    for mesh in asset_mesh.meshes.iter() {
                        for (_, sub_mesh) in mesh.meshes.iter() {
                            render_pass
                                .set_index_buffer(sub_mesh.index_buffer.clone());
                            render_pass.set_vertex_buffer(
                                0,
                                sub_mesh.vertex_buffer.as_ref().unwrap().clone(),
                            );
                            
                            render_pass.draw_indexed(
                                0..sub_mesh.index_count as u32,
                                0,
                                0..1,
                            );
                        }
                    }
                }
            }
            total += 1;
        }
    }

    // TODO: figure out a better stratgey for this..
    pub fn reset_used(&mut self) {
        for t in self.quad_textures.iter_mut() {
            t.free = 0;
        }
    }

    pub fn get_cube_coords(&mut self) -> (usize, usize) {
        let mut quad_index = 0;
        let mut inner_index = 0;
        for t in self.quad_textures.iter_mut() {
            if t.free < t.size {
                inner_index = t.free as usize;
                t.free += 1;
                break;
            } else {
                quad_index += 1;
            }
        }
        (quad_index, inner_index)
    }

    // pub fn calculate_quadrants(size: Vec2) -> (Vec<VirtualTexture>, Vec<VirtualTexture>, Vec<VirtualTexture>, Vec<VirtualTexture>) {
    //     // 4 quads in the atlas.
    //     let new_size = size / 2.0;
    //     // Each quad represents a cell in a 2x2 grid.
    //     // Quad 1 is split into a 4x4 grid.
    //     let quad1 = Self::create_nodes(new_size, Vec2::new(0.0, 0.0), 4);
    //     // Quad 2 is split into a 8x8 grid.
    //     let quad2 = Self::create_nodes(new_size, Vec2::new(new_size.x, 0.0), 8);
    //     // Quad 3 is split into a 16x16 grid.
    //     let quad3 = Self::create_nodes(new_size, Vec2::new(0.0, new_size.y), 16);
    //     // Quad 4 is split into a 32x32 grid.
    //     let quad4 = Self::create_nodes(new_size, Vec2::new(new_size.x, new_size.y), 32);

    //     (quad1, quad2, quad3, quad4)
    // }
}
