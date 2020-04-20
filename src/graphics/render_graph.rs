use super::{
    Pipeline, Renderer, SimplePipeline, SimplePipelineDesc, RenderTarget,
};
use crate::{AssetManager};
use std::collections::HashMap;

// TODO: handle node dependencies somehow.
#[derive(Debug)]
pub struct RenderGraphNode {
    pub(crate) pipeline: Pipeline,
    pub(crate) simple_pipeline: Box<dyn SimplePipeline>,
    pub(crate) depedency: Option<String>,
    pub frame: Option<RenderTarget>,
}

pub struct RenderGraph {
    nodes: HashMap<String, RenderGraphNode>,
    order: Vec<String>,
    pub(crate) local_bind_group_layout: wgpu::BindGroupLayout,
}

impl RenderGraph {
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        let nodes = HashMap::new();

        let local_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                bindings: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                }],
                label: None,
            });

        RenderGraph {
            nodes,
            order: Vec::new(),
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

    pub fn add<T: SimplePipelineDesc + Sized + 'static, T2: Into<String>>(
        &mut self,
        asset_manager: &AssetManager,
        renderer: &mut Renderer,
        name: T2,
        mut pipeline_desc: T,
        depedency: T2,
        include_local_bindings: bool,
        frame: Option<RenderTarget> // Optional view to render to.
    ) {
        let name = name.into();
        let depedency = depedency.into();
        let pipeline = pipeline_desc.pipeline(asset_manager, renderer, if include_local_bindings { Some(&self.local_bind_group_layout) } else { None });
        let built_pipeline: Box<dyn SimplePipeline> =
            Box::new(pipeline_desc.build(&renderer.device, &pipeline.bind_group_layouts));
        self.nodes.insert(name.clone(), RenderGraphNode {
            pipeline,
            simple_pipeline: built_pipeline,
            depedency: if !depedency.is_empty() { Some(depedency) } else { None },
            frame,
        });
    }

    pub(crate) fn remove(&mut self, _index: usize) {
        // self.pipeline.remove(index);
        unimplemented!();
    }

    pub fn length(&self) -> usize {
        self.nodes.len()
    }

    pub fn build_target<T: Into<String>>(&mut self, name: T) -> RenderTarget {
        let node = self.nodes.get_mut(&name.into()).unwrap();
        node.frame.take().unwrap()
    }

    pub(crate) fn render(
        &mut self,
        renderer: &mut Renderer,
        asset_manager: &mut AssetManager,
        mut world: Option<&mut specs::World>,
        frame: &wgpu::SwapChainOutput,
    ) -> Vec<wgpu::CommandBuffer> {
        // Calculate graph order.
        for (node_name, node) in self.nodes.iter() {
            if node.depedency.is_some() && !self.order.contains(&node.depedency.as_ref().unwrap()) {
                self.order.push(node.depedency.as_ref().unwrap().clone());
            }
            if !self.order.contains(node_name) {
                self.order.push(node_name.clone());
            }
        }

        let mut command_buffers = Vec::new();
        for node_name in self.order.iter() {
            let node: &mut RenderGraphNode = self.nodes.get_mut(node_name).unwrap();
            let command_buffer = node.simple_pipeline.render(
                Some(&frame.view),
                Some(&renderer.forward_depth),
                &renderer.device,
                &node.pipeline,
                Some(asset_manager),
                &mut world,
                &node.frame,
            );
            command_buffers.push(command_buffer);
        }

        command_buffers
    }
}
