use super::{resources::RenderTarget, Pipeline, Renderer, SimplePipeline, SimplePipelineDesc};
use crate::AssetManager;
use solvent::DepGraph;
use std::collections::HashMap;

// TODO: handle node dependencies somehow.
#[derive(Debug)]
pub struct RenderGraphNode {
    pub name: String,
    pub(crate) pipeline: Pipeline,
    pub(crate) simple_pipeline: Box<dyn SimplePipeline>,
    pub use_output_from_dependency: bool,
}

pub struct RenderGraph {
    nodes: HashMap<String, RenderGraphNode>,
    pub(crate) outputs: HashMap<String, Option<RenderTarget>>,
    dep_graph: DepGraph<String>,
    pub(crate) local_bind_group_layout: wgpu::BindGroupLayout,
}

impl RenderGraph {
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        let mut dep_graph = DepGraph::new();
        dep_graph.register_node("root".to_string());
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
            nodes: HashMap::new(),
            outputs: HashMap::new(),
            dep_graph,
            local_bind_group_layout,
        }
    }

    /// `input` - Optional view to render from. useful for post processing chains.
    /// 'output' - Optional view to render to. If none is set it will render to the latest frame buffer.
    pub fn add<T: SimplePipelineDesc + Sized + 'static, T2: Into<String>>(
        &mut self,
        asset_manager: &AssetManager,
        renderer: &mut Renderer,
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
            renderer,
            if include_local_bindings {
                Some(&self.local_bind_group_layout)
            } else {
                None
            },
        );
        let built_pipeline: Box<dyn SimplePipeline> =
            Box::new(pipeline_desc.build(&renderer.device, &pipeline.bind_group_layouts));
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

    pub(crate) fn render(
        &mut self,
        renderer: &mut Renderer,
        asset_manager: &mut AssetManager,
        world: &mut specs::World,
        frame: Option<&wgpu::SwapChainOutput>,
    ) -> wgpu::CommandBuffer {
        let mut encoder = renderer
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("main"),
            });

        let mut order = Vec::new();
        for (name, _) in self.nodes.iter_mut() {
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
                &mut renderer.device,
                &mut encoder,
                &node.pipeline,
                world,
            );

            let output = node.simple_pipeline.render(
                asset_manager,
                Some(&renderer.forward_depth),
                &renderer.device,
                &mut encoder,
                frame,
                input,
                output,
                &node.pipeline,
                world,
            );
            if output.is_some() {
                self.outputs.insert(name.clone(), output);
            }
        }

        encoder.finish()
    }
}
