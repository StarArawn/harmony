#![warn(rust_2018_idioms)]
#![allow(dead_code)]
#![allow(clippy::module_inception)]
#![allow(clippy::too_many_arguments)]
use wgpu;
use log;

use winit::{ 
    dpi::LogicalSize,
    event::{ Event, ModifiersState, WindowEvent },
    event_loop::{ ControlFlow },
};

use ultraviolet::vec::Vec2;

mod winit_state;
use winit_state::WinitState;

mod graphics;
use graphics::Renderer;

mod assets;
use assets::AssetManager;

mod gui;

use gui::components::Component;

struct WindowSize {
    width: u32,
    height: u32,
}

const WINDOW_SIZE: WindowSize = WindowSize {
    width: 1024,
    height: 768,
};

use crate::gui::core::Color;
const SURFACE: Color = Color::from_rgb(
    0x40 as f32 / 255.0,
    0x44 as f32 / 255.0,
    0x4B as f32 / 255.0,
);

const ACCENT: Color = Color::from_rgb(
    0x6F as f32 / 255.0,
    0xFF as f32 / 255.0,
    0xE9 as f32 / 255.0,
);

const ACTIVE: Color = Color::from_rgb(
    0x72 as f32 / 255.0,
    0x89 as f32 / 255.0,
    0xDA as f32 / 255.0,
);

const HOVERED: Color = Color::from_rgb(
    0x67 as f32 / 255.0,
    0x7B as f32 / 255.0,
    0xC4 as f32 / 255.0,
);


fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Warn)
        .filter_module("harmony", log::LevelFilter::Info)
        .init();

    let mut modifiers = ModifiersState::default();

    let (wb, events_loop) = WinitState::create("Harmony", LogicalSize::new(WINDOW_SIZE.width, WINDOW_SIZE.height));

    let (window, size, surface) = {
        let window = wb.build(&events_loop).unwrap();
        let size = window.inner_size();
        let surface = wgpu::Surface::create(&window);
        (window, size, surface)
    };

    let mut renderer = futures::executor::block_on(Renderer::new(window, size, surface));

    let asset_path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/").to_string();
    let mut asset_manager = AssetManager::new(asset_path);
    asset_manager.load(&renderer.device);
    let gui_renderer = crate::gui::Renderer::new(&asset_manager, &mut renderer.device, wgpu::TextureFormat::Bgra8UnormSrgb, LogicalSize::new(size.width, size.height));

    let mut stretch = stretch::node::Stretch::new();
    
    // let child = stretch.new_node(
    //     stretch::style::Style { size: stretch::geometry::Size { width: stretch::style::Dimension::Percent(0.5), height: stretch::style::Dimension::Auto }, justify_content: stretch::style::JustifyContent::Center, ..Default::default() },
    //     vec![],
    // ).unwrap();

    // let node = stretch.new_node(
    //     stretch::style::Style {
    //         size: stretch::geometry::Size { width: stretch::style::Dimension::Points(100.0), height: stretch::style::Dimension::Points(100.0) },
    //         ..Default::default()
    //     },
    //     vec![child],
    // ).unwrap();

    // stretch.compute_layout(node, stretch::geometry::Size::undefined()).unwrap();
    // dbg!(stretch.layout(child).unwrap());
    // dbg!(stretch.layout(node).unwrap());

    let window_darkness = 0.2;
    let mut window1 = gui::components::WindowBuilder::new();
    window1.set_background(gui::core::Background::from(Color::from_rgb(window_darkness, window_darkness, window_darkness)));
    window1.set_flex_size(stretch::style::Dimension::Percent(0.25), stretch::style::Dimension::Auto);
    window1.set_margin(50.0, 0.0, 0.0, 0.0);

    let mut window2 = gui::components::WindowBuilder::new();
    window2.set_background(gui::core::Background::from(Color::from_rgb(window_darkness, window_darkness, window_darkness)));
    window2.set_flex_size(stretch::style::Dimension::Percent(0.25), stretch::style::Dimension::Auto);
    window2.set_margin(50.0, 0.0, 0.0, 0.0);

    let mut window3 = gui::components::WindowBuilder::new();
    window3.set_background(gui::core::Background::from(Color::from_rgb(window_darkness, window_darkness, window_darkness)));
    window3.set_flex_size(stretch::style::Dimension::Percent(0.25), stretch::style::Dimension::Auto);
    window3.set_margin(50.0, 0.0, 0.0, 0.0);

    let mut window4 = gui::components::WindowBuilder::new();
    window4.set_background(gui::core::Background::from(Color::from_rgb(window_darkness, window_darkness, window_darkness)));
    window4.set_flex_size(stretch::style::Dimension::Percent(0.25), stretch::style::Dimension::Auto);
    window4.set_margin(50.0, 50.0, 0.0, 0.0);

    let mut flex_box = gui::components::Flex::new();
    flex_box.push(window1.build());
    flex_box.push(window2.build());
    flex_box.push(window3.build());
    flex_box.push(window4.build());

    events_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { event, .. } => {
            match event {
                WindowEvent::ModifiersChanged(new_modifiers) => {
                    modifiers = new_modifiers;
                }
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                winit::event::WindowEvent::KeyboardInput {
                    input:
                        winit::event::KeyboardInput {
                            virtual_keycode: Some(winit::event::VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => *control_flow = winit::event_loop::ControlFlow::Exit,
                winit::event::WindowEvent::Resized(dims) => {
                    println!("resized to {:?}", dims);
                    // renderer.dimensions = gfx_hal::window::Extent2D {
                    //     width: dims.width,
                    //     height: dims.height,
                    // };
                    // renderer.recreate_swapchain();
                },
                _ => {}
            }
        },
        Event::RedrawEventsCleared => {
            renderer.window.request_redraw();
        },
        Event::RedrawRequested(_) => {
            let output = renderer.render();
            let node = flex_box.node(&mut stretch, stretch::style::PositionType::Relative);
            gui_renderer.draw(
                &mut renderer.device,
                &mut renderer.queue,
                &output.view,
                flex_box.draw(&mut stretch, node),
                None,
                renderer.window.scale_factor() as f32,
                &mut asset_manager,
            );
        },
        _ => (),
    });
}