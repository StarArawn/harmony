use ordered_float::OrderedFloat;
use std::collections::{hash_map::DefaultHasher, HashMap};
use std::{
    hash::{Hash, Hasher},
    sync::Arc,
};

use super::{
    renderer::FRAME_FORMAT, resources::GPUResourceManager, CommandBufferQueue, VertexStateBuilder,
};
use crate::{assets::shader::Shader, AssetManager};
use solvent::DepGraph;

/// A description of a render pipeline.
/// Note: You can call `default()` to get a base implementation.
/// You'll still need to specify the correct shader at the very least.
#[derive(Debug, Hash, Clone)]
pub struct PipelineDesc {
    pub shader: String,
    pub vertex_state: VertexStateBuilder,
    pub primitive_topology: wgpu::PrimitiveTopology,
    pub color_state: wgpu::ColorStateDescriptor,
    pub depth_state: Option<wgpu::DepthStencilStateDescriptor>,
    pub sample_count: u32,
    pub sampler_mask: u32,
    pub alpha_to_coverage_enabled: bool,
    pub layouts: Vec<String>,
    pub front_face: wgpu::FrontFace,
    pub cull_mode: wgpu::CullMode,
    pub depth_bias: i32,
    pub depth_bias_slope_scale: OrderedFloat<f32>,
    pub depth_bias_clamp: OrderedFloat<f32>,
}

impl Default for PipelineDesc {
    fn default() -> Self {
        Self {
            shader: "".to_string(),
            vertex_state: VertexStateBuilder::new(),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_state: wgpu::ColorStateDescriptor {
                format: FRAME_FORMAT,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            },
            depth_state: None,
            sample_count: 1,
            sampler_mask: !0,
            alpha_to_coverage_enabled: false,
            layouts: Vec::new(),
            front_face: wgpu::FrontFace::Cw,
            cull_mode: wgpu::CullMode::Back,
            depth_bias: 0,
            depth_bias_slope_scale: 0.0.into(),
            depth_bias_clamp: 0.0.into(),
        }
    }
}

impl PipelineDesc {
    /// Creates a hash of the pipeline.
    pub fn create_hash(&self) -> u64 {
        let mut s = DefaultHasher::new();
        self.hash(&mut s);
        s.finish()
    }

    /// Builds a Pipeline from the description.
    pub fn build(
        &self,
        asset_manager: &AssetManager,
        device: &wgpu::Device,
        gpu_resource_manager: &GPUResourceManager,
    ) -> Pipeline {
        let shader_handle = asset_manager.get_shader(self.shader.clone());
        let shader = futures::executor::block_on(shader_handle.get_async()).unwrap();
        let shader = match *shader {
            Shader::Core(ref shader) => shader,
            _ => panic!("Pipeline/shader mismatch!"),
        };
        let vertex_stage = wgpu::ProgrammableStageDescriptor {
            module: &shader.vertex,
            entry_point: "main",
        };
        let fragment_stage = Some(wgpu::ProgrammableStageDescriptor {
            module: &shader.fragment,
            entry_point: "main",
        });

        let bind_group_layouts: Vec<Arc<wgpu::BindGroupLayout>> = self
            .layouts
            .iter()
            .map(|group_name| {
                gpu_resource_manager
                    .get_bind_group_layout(group_name)
                    .unwrap()
                    .clone()
            })
            .collect();
        let rasterization_state = wgpu::RasterizationStateDescriptor {
            front_face: self.front_face,
            cull_mode: self.cull_mode,
            depth_bias: self.depth_bias,
            depth_bias_slope_scale: self.depth_bias_slope_scale.into(),
            depth_bias_clamp: self.depth_bias_clamp.into(),
        };
        let primitive_topology = self.primitive_topology;
        let color_states = self.color_state.clone();
        let depth_stencil_state = self.depth_state.clone();
        let vertex_state_builder = self.vertex_state.clone();
        let sample_count = self.sample_count;
        let sample_mask = self.sampler_mask;
        let alpha_to_coverage_enabled = self.alpha_to_coverage_enabled;

        // Once we create the layout we don't need the bind group layouts.
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &bind_group_layouts
                .iter()
                .map(|x| x.as_ref())
                .collect::<Vec<&wgpu::BindGroupLayout>>(),
        });

        // Creates our vertex descriptor.
        let vertex_buffers: Vec<wgpu::VertexBufferDescriptor<'_>> = vertex_state_builder
            .buffer_desc
            .iter()
            .map(|desc| wgpu::VertexBufferDescriptor {
                stride: desc.stride,
                step_mode: desc.step_mode,
                attributes: &desc.attributes,
            })
            .collect();

        let vertex_state = wgpu::VertexStateDescriptor {
            index_format: vertex_state_builder.index_format,
            vertex_buffers: &vertex_buffers,
        };

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &layout,
            vertex_stage,
            fragment_stage,
            primitive_topology,
            color_states: &[color_states],
            rasterization_state: Some(rasterization_state),
            depth_stencil_state,
            vertex_state,
            sample_count,
            sample_mask,
            alpha_to_coverage_enabled,
        });

        Pipeline {
            desc: self.clone(),
            render_pipeline: pipeline,
        }
    }
}

#[derive(Debug, Hash, Clone)]
pub struct ComputePipelineDesc {
    pub shader: String,
    pub layouts: Vec<String>,
}

impl ComputePipelineDesc {
    pub fn new<T: Into<String>>(shader: T) -> Self {
        Self {
            shader: shader.into(),
            layouts: vec![],
        }
    }

    /// Creates a hash of the pipeline.
    pub fn create_hash(&self) -> u64 {
        let mut s = DefaultHasher::new();
        self.hash(&mut s);
        s.finish()
    }

    /// Builds a Pipeline from the description.
    pub fn build(
        &self,
        asset_manager: &AssetManager,
        device: &wgpu::Device,
        gpu_resource_manager: &GPUResourceManager,
    ) -> ComputePipeline {
        let shader_handle = asset_manager.get_shader(self.shader.clone());
        let shader = futures::executor::block_on(shader_handle.get_async()).unwrap();
        let shader = match *shader {
            Shader::Compute(ref shader) => shader,
            _ => panic!("Pipeline/shader mismatch!"),
        };

        let compute_stage = wgpu::ProgrammableStageDescriptor {
            module: &shader.compute,
            entry_point: "main",
        };

        let bind_group_layouts: Vec<Arc<wgpu::BindGroupLayout>> = self
            .layouts
            .iter()
            .map(|group_name| {
                gpu_resource_manager
                    .get_bind_group_layout(group_name)
                    .unwrap()
                    .clone()
            })
            .collect();

        // Once we create the layout we don't need the bind group layouts.
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &bind_group_layouts
                .iter()
                .map(|x| x.as_ref())
                .collect::<Vec<&wgpu::BindGroupLayout>>(),
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            layout: &layout,
            compute_stage,
        });

        ComputePipeline {
            desc: self.clone(),
            compute_pipeline,
        }
    }
}

/// An actual Render Pipeline that should be stored in the manager.
/// Also contains a description of the pipeline.
pub struct Pipeline {
    pub desc: PipelineDesc,
    pub render_pipeline: wgpu::RenderPipeline,
}

pub struct ComputePipeline {
    pub desc: ComputePipelineDesc,
    pub compute_pipeline: wgpu::ComputePipeline,
}

/// The type of pipeline.
pub enum PipelineType {
    Pipeline(Pipeline),
    ComputePipeline(ComputePipeline),
    /// Node is used for things that run on the GPU without a pipeline. Such as globals.
    Node,
    // TODO: Add group type.
}

/// This is essentially a render graph with additional features.
/// It can also manage duplicate pipelines.
pub struct PipelineManager {
    pipelines: HashMap<String, HashMap<u64, PipelineType>>,
    pub(crate) current_pipelines: HashMap<String, u64>,
    dep_graph: DepGraph<String>,
    order: Vec<String>,
}

impl PipelineManager {
    /// Creates a new pipeline manager.
    pub fn new() -> Self {
        let mut dep_graph = DepGraph::new();
        dep_graph.register_node("root".to_string());
        Self {
            pipelines: HashMap::new(),
            dep_graph,
            order: Vec::new(),
            current_pipelines: HashMap::new(),
        }
    }

    /// This lets you add new pipelines. Note: You can have multiple pipelines for the same shader. It's recommended that you store
    /// PipelineDesc and pass it in when retrieving the pipeline.
    /// Note: Pipeline's are considered a fairly costly operation, try not to create a new one every frame.
    pub fn add_pipeline<T: Into<String>>(
        &mut self,
        name: T,
        pipeline_desc: &PipelineDesc,
        dependency: Vec<&str>,
        device: &wgpu::Device,
        asset_manager: &AssetManager,
        gpu_resource_manager: Arc<GPUResourceManager>, // TODO: This is an arc so just throw it in via new
    ) {
        let hash = pipeline_desc.create_hash();
        let name = name.into();

        if !self.pipelines.contains_key(&name) {
            let pipeline_hashmap = HashMap::new();
            self.pipelines.insert(name.clone(), pipeline_hashmap);

            // Save the first pipeline into our special hashmap for keeping track of that.
            self.current_pipelines.insert(name.clone(), hash);
        }

        let pipeline_hashmap = self.pipelines.get_mut(&name).unwrap();
        if pipeline_hashmap.contains_key(&hash) {
            // Already exists do nothing in this case.
            return;
        }

        let pipeline = pipeline_desc.build(&asset_manager, &device, &gpu_resource_manager);
        pipeline_hashmap.insert(hash, PipelineType::Pipeline(pipeline));

        // Add to our graph
        self.dep_graph.register_node(name.clone());

        if dependency.len() > 0 {
            let dependency = dependency
                .iter()
                .map(|name| name.to_string())
                .collect::<Vec<String>>();
            self.dep_graph
                .register_dependencies(name.clone(), dependency);
        }

        // Recalculate order.
        self.get_order();
    }

    /// This lets you add new compute pipelines. Note: You can have multiple pipelines for the same shader. It's recommended that you store
    /// PipelineDesc and pass it in when retrieving the pipeline.
    /// Note: Pipeline's are considered a fairly costly operation, try not to create a new one every frame.
    pub fn add_compute_pipeline<T: Into<String>>(
        &mut self,
        name: T,
        pipeline_desc: &ComputePipelineDesc,
        dependency: Vec<&str>,
        device: &wgpu::Device,
        asset_manager: &AssetManager,
        gpu_resource_manager: Arc<GPUResourceManager>,
    ) {
        let hash = pipeline_desc.create_hash();
        let name = name.into();

        if !self.pipelines.contains_key(&name) {
            let pipeline_hashmap = HashMap::new();
            self.pipelines.insert(name.clone(), pipeline_hashmap);

            // Save the first pipeline into our special hashmap for keeping track of that.
            self.current_pipelines.insert(name.clone(), hash);
        }

        let pipeline_hashmap = self.pipelines.get_mut(&name).unwrap();
        if pipeline_hashmap.contains_key(&hash) {
            // Already exists do nothing in this case.
            return;
        }

        let pipeline = pipeline_desc.build(&asset_manager, &device, &gpu_resource_manager);
        pipeline_hashmap.insert(hash, PipelineType::ComputePipeline(pipeline));

        // Add to our graph
        self.dep_graph.register_node(name.clone());

        if dependency.len() > 0 {
            let dependency = dependency
                .iter()
                .map(|name| name.to_string())
                .collect::<Vec<String>>();
            self.dep_graph
                .register_dependencies(name.clone(), dependency);
        }

        // Recalculate order.
        self.get_order();
    }


    /// A node is an encoder you want to run at some step inside of the pipeline workflow.
    pub fn add_node<T: Into<String>>(&mut self, name: T, dependency: Vec<&str>) {
        let name = name.into();
        let mut hasher = DefaultHasher::new();
        name.hash(&mut hasher);
        let hash = hasher.finish();

        if !self.pipelines.contains_key(&name) {
            let pipeline_hashmap = HashMap::new();
            self.pipelines.insert(name.clone(), pipeline_hashmap);

            // Save the first pipeline into our special hashmap for keeping track of that.
            self.current_pipelines.insert(name.clone(), hash);
        }

        let pipeline_hashmap = self.pipelines.get_mut(&name).unwrap();
        if pipeline_hashmap.contains_key(&hash) {
            // Already exists do nothing in this case. Perhaps error?
            return;
        }

        // Add to our graph
        self.dep_graph.register_node(name.clone());

        if dependency.len() > 0 {
            let dependency = dependency
                .iter()
                .map(|name| name.to_string())
                .collect::<Vec<String>>();
            self.dep_graph
                .register_dependencies(name.clone(), dependency);
        }

        // Recalculate order.
        self.get_order();
    }

    fn get_order(&mut self) {
        let mut order = Vec::new();
        for (name, _) in self.pipelines.iter() {
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

        // UI always comes last.
        order.push("UI".to_string());

        self.order = order;
    }

    /// Let's you retrieve a reference to a pipeline from the manager.
    /// Note if you don't pass in a pipeline description it defaults to whatever the current pipeline is.
    pub fn get<T: Into<String>>(
        &self,
        name: T,
        pipeline_desc: Option<&PipelineDesc>,
    ) -> Option<&Pipeline> {
        let name = name.into();
        let pipeline_hashmap = self.pipelines.get(&name);
        if pipeline_hashmap.is_none() {
            return None;
        }

        let hash = if pipeline_desc.is_some() {
            pipeline_desc.unwrap().create_hash()
        } else {
            *self.current_pipelines.get(&name).unwrap()
        };

        let pipeline_type = pipeline_hashmap.unwrap().get(&hash);
        if pipeline_type.is_none() {
            return None;
        }
        match pipeline_type.as_ref().unwrap() {
            PipelineType::Pipeline(pipeline) => Some(pipeline),
            _ => None,
        }
    }

    /// Let's you retrieve a reference to a pipeline from the manager.
    /// Note if you don't pass in a pipeline description it defaults to whatever the current pipeline is.
    pub fn get_compute<T: Into<String>>(
        &self,
        name: T,
        pipeline_desc: Option<&PipelineDesc>,
    ) -> Option<&ComputePipeline> {
        let name = name.into();
        let pipeline_hashmap = self.pipelines.get(&name);
        if pipeline_hashmap.is_none() {
            return None;
        }

        let hash = if pipeline_desc.is_some() {
            pipeline_desc.unwrap().create_hash()
        } else {
            *self.current_pipelines.get(&name).unwrap()
        };

        let pipeline_type = pipeline_hashmap.unwrap().get(&hash);
        if pipeline_type.is_none() {
            return None;
        }
        match pipeline_type.as_ref().unwrap() {
            PipelineType::ComputePipeline(pipeline) => Some(pipeline),
            _ => None,
        }
    }

    /// Get's the hash for the current pipeline being used.
    pub fn get_current_pipeline_hash<T: Into<String>>(&self, name: T) -> u64 {
        let name = name.into();
        *self.current_pipelines.get(&name).unwrap()
    }

    /// Sets the current pipeline that is used based off a hash.
    pub fn set_current_pipeline_hash<T: Into<String>>(&mut self, name: T, hash: u64) {
        let name = name.into();
        self.current_pipelines.insert(name, hash);
    }

    /// Collects command buffers for submission.
    pub(crate) fn collect_buffers(
        &self,
        command_queue: &mut CommandBufferQueue,
    ) -> Vec<wgpu::CommandBuffer> {
        let mut command_buffers = Vec::new();
        let mut queue_items = Vec::new();
        while let Ok(command) = command_queue.pop() {
            queue_items.push(command);
        }

        for order in self.order.iter() {
            while let Some(queue_item_index) = queue_items
                .iter()
                .position(|queue_item| &queue_item.name == order)
            {
                let queue_item = queue_items.remove(queue_item_index);
                command_buffers.push(queue_item.buffer);
            }
        }

        command_buffers
    }
}
