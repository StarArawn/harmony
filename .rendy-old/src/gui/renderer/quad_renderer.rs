use rendy::{
    command::{Families, QueueId, RenderPassEncoder},
    factory::{Config, Factory, ImageState},
    graph::{
        present::PresentNode, render::*, Graph, GraphBuilder, GraphContext, NodeBuffer, NodeImage,
    },
    hal::{self, device::Device as _},
    init::winit::{
        event::{Event, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
    },
    init::AnyWindowedRendy,
    memory::Dynamic,
    mesh::PosTex,
    resource::{Buffer, BufferInfo, DescriptorSet, DescriptorSetLayout, Escape, Handle},
    shader::{ShaderKind, SourceLanguage, SourceShaderInfo, SpirvShader},
    texture::{image::ImageTextureConfig, Texture},
};

use crate::gui::renderer::vertex_formats::QuadVertex;
use crate::assets::AssetManager;

#[derive(Debug, Default)]
pub struct QuadPipelineDesc;

#[derive(Debug)]
pub struct QuadPipeline<B: hal::Backend> {
    vbuf: Escape<Buffer<B>>,
    index_buf: Escape<Buffer<B>>,
    instance_buf: Escape<Buffer<B>>,
}

const QUAD_INDICES: [u16; 6] = [0, 1, 2, 0, 2, 3];


impl<B> SimpleGraphicsPipelineDesc<B, AssetManager> for QuadPipelineDesc
where
    B: hal::Backend,
{
    type Pipeline = QuadPipeline<B>;

    fn depth_stencil(&self) -> Option<hal::pso::DepthStencilDesc> {
        None
    }

    fn load_shader_set(&self, factory: &mut Factory<B>, asset_manager: &AssetManager) ->  rendy::shader::ShaderSet<B>  {
        let shader = asset_manager.get_shader(String::from("gui_quad.shader"));
        shader.builder.build(factory,  Default::default()).unwrap()
    }

    fn vertices(
        &self,
        asset_manager: &AssetManager,
    ) -> Vec<(
        Vec<hal::pso::Element<hal::format::Format>>,
        hal::pso::ElemStride,
        hal::pso::VertexInputRate,
    )> {
        let shader = asset_manager.get_shader(String::from("gui_quad.shader"));
        let gfx_vertices = shader.reflection
            .attributes_range(..)
            .unwrap()
            .gfx_vertex_input_desc(hal::pso::VertexInputRate::Vertex);
        println!("Vertex format:");
        println!("{:?}", gfx_vertices);
        
        vec![gfx_vertices]
    }

    fn layout(&self, asset_manager: &AssetManager) -> Layout {
        let shader = asset_manager.get_shader(String::from("gui_quad.shader"));
        let layout = shader.reflection.layout().unwrap();
        println!("Pipline Layout:");
        println!("{:?}", layout);
        
        layout
    }

    fn build<'b>(
        self,
        _ctx: &GraphContext<B>,
        factory: &mut Factory<B>,
        queue: QueueId,
        asset_manager: &AssetManager,
        buffers: Vec<NodeBuffer>,
        images: Vec<NodeImage>,
        set_layouts: &[Handle<DescriptorSetLayout<B>>],
    ) -> Result<QuadPipeline<B>, hal::pso::CreationError> {
        assert!(buffers.is_empty());
        assert!(images.is_empty());
        assert_eq!(set_layouts.len(), 1);

        let shader = asset_manager.get_shader(String::from("gui_quad.shader"));

        let mut vbuf = factory
            .create_buffer(
                BufferInfo {
                    size: 4,
                    usage: hal::buffer::Usage::VERTEX,
                },
                Dynamic,
            )
            .unwrap();
        let mut index_buf = factory
            .create_buffer(
                BufferInfo {
                    size: 6,
                    usage: hal::buffer::Usage::VERTEX,
                },
                Dynamic,
            )
            .unwrap();

        let mut instance_buf = factory
            .create_buffer(
                BufferInfo {
                    size: 100,
                    usage: hal::buffer::Usage::VERTEX,
                },
                Dynamic,
            )
            .unwrap();

        unsafe {
            let quad_verts: [QuadVertex; 4] = [
                QuadVertex {
                    position: [0.0, 0.0].into(),
                },
                QuadVertex {
                    position: [1.0, 0.0].into(),
                },
                QuadVertex {
                    position: [1.0, 1.0].into(),
                },
                QuadVertex {
                    position: [0.0, 1.0].into(),
                },
            ];
            // Fresh buffer.
            factory
                .upload_visible_buffer(
                    &mut vbuf,
                    0,
                    &quad_verts,
                )
                .unwrap();

            factory.upload_visible_buffer(
                &mut index_buf,
                0,
                &QUAD_INDICES,
            ).unwrap();
        }

        Ok(QuadPipeline {
            vbuf,
            index_buf,
            instance_buf,
        })
    }
}

impl<B> SimpleGraphicsPipeline<B, AssetManager> for QuadPipeline<B>
where
    B: hal::Backend,
{
    type Desc = QuadPipelineDesc;

    fn prepare(
        &mut self,
        _factory: &Factory<B>,
        _queue: QueueId,
        _set_layouts: &[Handle<DescriptorSetLayout<B>>],
        _index: usize,
        _aux: &AssetManager,
    ) -> PrepareResult {
        PrepareResult::DrawReuse
    }

    fn draw(
        &mut self,
        layout: &B::PipelineLayout,
        mut encoder: RenderPassEncoder<'_, B>,
        _index: usize,
        _aux: &AssetManager,
    ) {
        unsafe {

        }
    }

    fn dispose(self, _factory: &mut Factory<B>, _aux: &AssetManager) {

    }
}
