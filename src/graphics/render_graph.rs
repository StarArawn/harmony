use super::{
    pipelines::{SkyboxPipelineDesc, UnlitPipelineDesc, PBRPipelineDesc},
    Pipeline, Renderer, SimplePipeline, SimplePipelineDesc,
};
use crate::{Application, AssetManager};
use std::collections::HashMap;

// TODO: handle node dependencies somehow.
pub struct RenderGraphNode {
    pub(crate) pipeline: Pipeline,
    pub(crate) simple_pipeline: Box<dyn SimplePipeline>,
}

pub struct RenderGraph {
    nodes: HashMap<String, RenderGraphNode>,
    order: Vec<String>,
    pub(crate) local_bind_group_layout: wgpu::BindGroupLayout,
}

impl RenderGraph {
    pub(crate) fn new(app: &mut Application) -> Self {
        let mut nodes = HashMap::new();

        let local_bind_group_layout =
            app.renderer.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                bindings: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                }],
                label: None,
            });

        // Unlit pipeline
        let mut unlit_pipeline_desc = UnlitPipelineDesc::default();
        let pipeline = unlit_pipeline_desc.pipeline(&app.asset_manager, &mut app.renderer, Some(&local_bind_group_layout));
        let unlit_pipeline: Box<dyn SimplePipeline> =
            Box::new(unlit_pipeline_desc.build(&app.renderer.device, &pipeline.bind_group_layouts));
        nodes.insert(
            "unlit".to_string(),
            RenderGraphNode {
                pipeline,
                simple_pipeline: unlit_pipeline,
            }
        );

        // PBR pipeline
        let mut pbr_pipeline_desc = PBRPipelineDesc::default();
        let pipeline = pbr_pipeline_desc.pipeline(&app.asset_manager, &mut app.renderer, Some(&local_bind_group_layout));
        let pbr_pipeline: Box<dyn SimplePipeline> =
            Box::new(pbr_pipeline_desc.build(&app.renderer.device, &pipeline.bind_group_layouts));
        nodes.insert(
            "pbr".to_string(),
            RenderGraphNode {
                pipeline,
                simple_pipeline: pbr_pipeline,
            }
        );

        // Skybox pipeline
        let mut skybox_pipeline_desc = SkyboxPipelineDesc::default();
        let pipeline = skybox_pipeline_desc.pipeline(&app.asset_manager, &mut app.renderer, None);
        let skybox_pipeline: Box<dyn SimplePipeline> = Box::new(
            skybox_pipeline_desc.build(&app.renderer.device, &pipeline.bind_group_layouts),
        );
        nodes.insert(
            "skybox".to_string(),
            RenderGraphNode {
                pipeline,
                simple_pipeline: skybox_pipeline, // Nodes always dirty at first.
            },
        );

        RenderGraph {
            nodes,
            order: vec!["skybox".to_string(), "unlit".to_string(), "pbr".to_string()],
            local_bind_group_layout,
        }
    }

    pub fn get<T>(&self, key: T) -> &RenderGraphNode
    where
        T: Into<String>,
    {
        let key = &key.into();
        self.nodes
            .get(key)
            .unwrap_or_else(|| panic!(format!("Couldn't find render graph node called: {}", key)))
    }

    pub fn add<T: SimplePipelineDesc + Sized + 'static, T2: Into<String>>(&mut self, asset_manager: &AssetManager, renderer: &mut Renderer, name: T2, mut pipeline_desc: T) {
        let name = name.into();
        let pipeline = pipeline_desc.pipeline(asset_manager, renderer, Some(&self.local_bind_group_layout));
        let built_pipeline: Box<dyn SimplePipeline> =
            Box::new(pipeline_desc.build(&renderer.device, &pipeline.bind_group_layouts));
        self.nodes.insert(name.clone(), RenderGraphNode {
            pipeline,
            simple_pipeline: built_pipeline
        });
        self.order.push(name.clone());
    }

    pub(crate) fn remove(&mut self, _index: usize) {
        // self.pipeline.remove(index);
        unimplemented!();
    }

    pub fn length(&self) -> usize {
        self.nodes.len()
    }

    pub(crate) fn render(
        &mut self,
        renderer: &mut Renderer,
        asset_manager: &mut AssetManager,
        world: &mut specs::World,
        frame: &wgpu::SwapChainOutput,
    ) -> Vec<wgpu::CommandBuffer> {
        let mut command_buffers = Vec::new();
        for node_name in self.order.iter() {
            let node: &mut RenderGraphNode = self.nodes.get_mut(node_name).unwrap();
            let command_buffer = node.simple_pipeline.render(
                Some(&frame.view),
                Some(&renderer.forward_depth),
                &renderer.device,
                &node.pipeline,
                Some(asset_manager),
                Some(world),
            );
            command_buffers.push(command_buffer);
        }

        command_buffers
    }
}
