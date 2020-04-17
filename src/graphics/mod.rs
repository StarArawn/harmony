mod renderer;
pub use renderer::Renderer;

pub mod material;

pub mod mesh;

mod render_graph;
pub use render_graph::RenderGraph;

mod pipeline;
pub use pipeline::{BindGroupWithData, Pipeline, SimplePipeline, SimplePipelineDesc};

pub mod pipelines;
