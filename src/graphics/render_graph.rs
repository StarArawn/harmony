use super::{resources::{BindingManager, RenderTarget}, Pipeline, SimplePipeline, SimplePipelineDesc};
use crate::AssetManager;
use solvent::DepGraph;
use std::collections::HashMap;
use legion::systems::resource::Resources;

use crossbeam::queue::ArrayQueue;

pub struct CommandQueueItem {
    pub name: String,
    pub buffer: wgpu::CommandBuffer,
}

pub type CommandBufferQueue = ArrayQueue::<CommandQueueItem>;

// TODO: handle node dependencies somehow.
#[derive(Debug)]
pub struct RenderGraphNode {
    pub name: String,
    pub(crate) pipeline: Pipeline,
    pub(crate) simple_pipeline: Box<dyn SimplePipeline>,
    pub use_output_from_dependency: bool,
}

pub struct RenderGraph {
    pub(crate) nodes: HashMap<String, RenderGraphNode>,
    pub(crate) outputs: HashMap<String, Option<RenderTarget>>,
    dep_graph: DepGraph<String>,
    pub(crate) local_bind_group_layout: wgpu::BindGroupLayout,
    pub binding_manager: BindingManager,
}

impl RenderGraph {
    pub(crate) fn new(resources: &mut Resources, create_command_queue: bool) -> Self {
        let mut dep_graph = DepGraph::new();
        dep_graph.register_node("root".to_string());
        
        let local_bind_group_layout = {
            let device = resources.get::<wgpu::Device>().unwrap();
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                bindings: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                }],
                label: None,
            })
        };

        if create_command_queue {
            let command_queue = CommandBufferQueue::new(50);
            resources.insert(command_queue);
        }

        RenderGraph {
            nodes: HashMap::new(),
            outputs: HashMap::new(),
            dep_graph,
            local_bind_group_layout,
            binding_manager: BindingManager::new(),
        }
    }

    /// `input` - Optional view to render from. useful for post processing chains.
    /// 'output' - Optional view to render to. If none is set it will render to the latest frame buffer.
    pub fn add<T: SimplePipelineDesc + Sized + 'static, T2: Into<String>>(
        &mut self,
        asset_manager: &AssetManager,
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
        name: T2,
        mut pipeline_desc: T,
        dependency: Vec<&str>,
        include_local_bindings: bool,
        output: Option<RenderTarget>,
        use_output_from_dependency: bool,
    ) {
        let name = name.into();
        let pipeline = pipeline_desc.pipeline(
            asset_manager,
            device,
            sc_desc,
            if include_local_bindings {
                Some(&self.local_bind_group_layout)
            } else {
                None
            },
        );
        let built_pipeline: Box<dyn SimplePipeline> =
            Box::new(pipeline_desc.build(&device, &pipeline.bind_group_layouts, &mut self.binding_manager));
        let node = RenderGraphNode {
            name: name.clone(),
            pipeline,
            simple_pipeline: built_pipeline,
            use_output_from_dependency,
        };
        self.nodes.insert(name.clone(), node);
        self.outputs.insert(name.clone(), output);
        self.dep_graph.register_node(name.clone());
        if dependency.len() > 0 {
            let dependency = dependency
                .iter()
                .map(|name| name.to_string())
                .collect::<Vec<String>>();
            self.dep_graph
                .register_dependencies(name.clone(), dependency);
        }
    }

    /// Allows you to take the output render target for a given node.
    pub fn pull_render_target<T>(&mut self, name: T) -> RenderTarget
    where
        T: Into<String>,
    {
        let name = name.into();
        let output = self.outputs.get_mut(&name).unwrap();
        output.take().unwrap()
    }

    /// Allows you to take the output render target for a given node.
    pub fn get<T>(&self, name: T) -> &RenderGraphNode
    where
        T: Into<String>,
    {
        self.nodes.get(&name.into()).unwrap()
    }

    fn get_order(&self) -> Vec<String> {
        let mut order = Vec::new();
        for (name, _) in self.nodes.iter() {
            let dependencies = self.dep_graph.dependencies_of(&name);
            if dependencies.is_ok() {
                for node in dependencies.unwrap() {
                    match node {
                        Ok(n) => {
                            if !order.contains(n) {
                                order.push(n.clone());
                            }
                        }
                        Err(e) => panic!("Solvent error detected: {:?}", e),
                    }
                }
            }
        }

        order
    }

    pub(crate) fn render_one_time(
        &mut self,
        device: &wgpu::Device,
        asset_manager: &mut AssetManager,
        world: &mut legion::world::World,
        frame: Option<&wgpu::SwapChainOutput>,
        forward_depth: Option<&wgpu::TextureView>,
    ) -> wgpu::CommandBuffer {
        let mut encoder = device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("main"),
            });

        let order = self.get_order();
        
        for name in order {
            let node = self.nodes.get_mut(&name).unwrap();
            let mut input = None;
            if node.use_output_from_dependency {
                let dependencies = self.dep_graph.dependencies_of(&name);
                if dependencies.is_ok() {
                    let mut dependencies = dependencies.unwrap();
                    let dependency = dependencies.next().unwrap();
                    if dependency.is_ok() {
                        let dependency = dependency.unwrap().to_string();
                        input = self.outputs.get(&dependency).unwrap().as_ref();
                    }
                }
            }
            let output = self.outputs.get(&name).unwrap().as_ref();

            node.simple_pipeline.prepare(
                asset_manager,
                device,
                &mut encoder,
                &node.pipeline,
                world,
            );

            let output = node.simple_pipeline.render(
                asset_manager,
                forward_depth,
                device,
                &mut encoder,
                frame,
                input,
                output,
                &node.pipeline,
                world,
                &mut self.binding_manager,
            );
            if output.is_some() {
                self.outputs.insert(name.clone(), output);
            }
        }

        encoder.finish()
    }

    pub fn collect_buffers(&self, command_queue: &mut CommandBufferQueue) -> Vec<wgpu::CommandBuffer> {
        let mut command_buffers = Vec::new();
        let mut queue_items = Vec::new();
        while let Ok(command) = command_queue.pop() {
            queue_items.push(command);
        }

        // TODO: probably shouldn't do this every frame.
        let ordering = self.get_order();

        for order in ordering {
            let queue_item_index = queue_items.iter().position(|queue_item| queue_item.name == order);
            if queue_item_index.is_some() {
                let queue_item = queue_items.remove(queue_item_index.unwrap());
                command_buffers.push(queue_item.buffer);
            }
        }

        command_buffers
    }
}
