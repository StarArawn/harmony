pub mod renderer;
pub use renderer::Renderer;

pub mod material;

pub mod mesh;

mod render_graph;
pub use render_graph::{CommandBufferQueue, CommandQueueItem, RenderGraph};

mod pipeline;
pub use pipeline::{BindGroupWithData, SimplePipeline, SimplePipelineDesc, VertexStateBuilder};

pub mod pipelines;

pub mod resources;

pub mod systems;

pub mod pipeline_manager;