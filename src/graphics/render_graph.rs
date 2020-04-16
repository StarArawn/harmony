use std::collections::HashMap;
use crate::{ AssetManager, Application };
use super::{
    pipeline::PrepareResult,
    pipelines::{ UnlitPipelineDesc, SkyboxPipelineDesc},
    Renderer,
    Pipeline,
    SimplePipeline,
    SimplePipelineDesc,
};

// TODO: handle node dependencies somehow.
pub struct RenderGraphNode {
    pub(crate) pipeline: Pipeline,
    pub(crate) simple_pipeline: Box<dyn SimplePipeline>,
    pub(crate) command_buffer: Option<wgpu::CommandBuffer>,
    pub(crate) dirty: bool,
}

pub struct RenderGraph {
    nodes: HashMap<String, RenderGraphNode>,
    order: Vec<String>,
}

impl RenderGraph {
    pub fn new(app: &mut Application) -> Self {
        let mut nodes = HashMap::new();

        // Unlit pipeline
        let mut unlit_pipeline_desc = UnlitPipelineDesc::default();
        let pipeline = unlit_pipeline_desc.pipeline(app);
        let unlit_pipeline: Box<dyn SimplePipeline> = Box::new(unlit_pipeline_desc.build(&app.renderer.device, &pipeline.bind_group_layouts));
        nodes.insert("unlit".to_string(), RenderGraphNode {
            pipeline,
            simple_pipeline: unlit_pipeline,
            command_buffer: None,
            dirty: true, // Nodes always dirty at first.
        });

        // Skybox pipeline 
        let mut skybox_pipeline_desc = SkyboxPipelineDesc::default();
        let pipeline = skybox_pipeline_desc.pipeline(app);
        let material_layout = &pipeline.bind_group_layouts[1];
        for cubemap_image in app.asset_manager.hdr_images.values_mut() {
            let bind_group = app.renderer.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &material_layout,
                bindings: &[
                    wgpu::Binding {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(cubemap_image.cubemap_view.as_ref().unwrap()),
                    },
                    wgpu::Binding {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&cubemap_image.cubemap_sampler),
                    }
                ],
                label: None,
            });
            cubemap_image.cubemap_bind_group = Some(bind_group);
        }
        let skybox_pipeline: Box<dyn SimplePipeline> = Box::new(skybox_pipeline_desc.build(&app.renderer.device, &pipeline.bind_group_layouts));
        nodes.insert("skybox".to_string(), RenderGraphNode {
            pipeline,
            simple_pipeline: skybox_pipeline,
            command_buffer: None,
            dirty: true, // Nodes always dirty at first.
        });


        RenderGraph {
            nodes,
            order: vec!["skybox".to_string(), "unlit".to_string()]
        }
    }

    pub fn get<T>(&self, key: T) -> &RenderGraphNode where T: Into<String> {
        let key = &key.into();
        self.nodes.get(key).unwrap_or_else(|| panic!(format!("Couldn't find render graph node called: {}", key)))
    }

    pub fn add<T: SimplePipeline + Sized + 'static>(&mut self, _pipeline: T) {
        // TODO: fix this code up to support custom pipelines..
    }

    pub fn remove(&mut self, _index: usize) {
        // self.pipeline.remove(index);
    }

    pub fn length(&self) -> usize {
        self.nodes.len()
    }

    pub fn render(&mut self, renderer: &mut Renderer, asset_manager: &mut AssetManager, world: &mut specs::World, frame: &wgpu::SwapChainOutput) -> Vec<wgpu::CommandBuffer>{
        let mut command_buffers = Vec::new();
        for node_name in self.order.iter() {
            let node: &mut RenderGraphNode = self.nodes.get_mut(node_name).unwrap();
            if node.simple_pipeline.prepare() == PrepareResult::Record || node.dirty {
                let command_buffer = node.simple_pipeline.render(Some(&frame.view), &renderer.device, &node.pipeline, Some(asset_manager), Some(world), Some(&renderer.forward_depth));
                command_buffers.push(command_buffer);
                //node.dirty = false;
            }
        }

        command_buffers
    }
}