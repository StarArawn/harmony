use rendy::{
    command::{Families, QueueId, RenderPassEncoder},
    factory::{Config, Factory, ImageState},
    graph::{
        present::PresentNode, render::*, Graph, GraphBuilder, GraphContext, NodeBuffer, NodeImage,
    },
    hal::{self, device::Device as _},
    init::winit::{
        self,
        event::{Event, WindowEvent, KeyboardInput, VirtualKeyCode},
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

mod graphics;

mod assets;
use assets::AssetManager;

mod gui;

#[cfg(feature = "dx12")]
pub type Backend = rendy::dx12::Backend;

#[cfg(feature = "metal")]
pub type Backend = rendy::metal::Backend;

#[cfg(feature = "vulkan")]
pub type Backend = rendy::vulkan::Backend;

#[cfg(feature = "empty")]
pub type Backend = rendy::empty::Backend;


fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Warn)
        .filter_module("harmony", log::LevelFilter::Info)
        .init();

    let asset_path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/").to_string();
    let mut asset_manager = AssetManager::new(asset_path);
    asset_manager.load();

    let config: Config = Default::default();

    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("rendy-pbr")
        .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 960.0));

    let rendy = rendy::init::AnyWindowedRendy::init_auto(&config, window, &event_loop).unwrap();
    rendy::with_any_windowed_rendy!((rendy)
        (mut factory, mut families, surface, window) => {

            let mut graph_builder = GraphBuilder::<Backend, AssetManager>::new();

            let size = window.inner_size();

            let color = graph_builder.create_image(
                hal::image::Kind::D2(size.width as u32, size.height as u32, 1, 1),
                1,
                factory.get_surface_format(&surface),
                Some(hal::command::ClearValue {
                    color: hal::command::ClearColor {
                        float32: [0.7, 0.7, 0.7, 1.0],
                    },
                }),
            );

            let pass = graph_builder.add_node(
                gui::renderer::QuadPipeline::builder()
                    .into_subpass()
                    .with_color(color)
                    .into_pass(),
            );

            graph_builder.add_node(PresentNode::builder(&factory, surface, color).with_dependency(pass));

            let mut graph = graph_builder
                .with_frames_in_flight(3)
                .build(&mut factory, &mut families, &asset_manager).unwrap();

            event_loop.run(move |event, _, control_flow| match event {
                Event::WindowEvent { event, .. } => {
                    match event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => *control_flow = winit::event_loop::ControlFlow::Exit,
                        _ => {}
                    }
                },
                Event::MainEventsCleared => {
                    factory.maintain(&mut families);
                    
                    graph.run(&mut factory, &mut families, &asset_manager);
                }
                _ => (),
            });
        }
    )
}