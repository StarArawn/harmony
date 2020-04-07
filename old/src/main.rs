#![warn(rust_2018_idioms)]
#![allow(dead_code)]
#![allow(clippy::module_inception)]
#![allow(clippy::too_many_arguments)]
use wgpu;

use winit::{ 
    dpi::LogicalSize,
    event::{ Event, ModifiersState, WindowEvent },
    event_loop::{ ControlFlow },
};

mod winit_state;
use winit_state::WinitState;

mod graphics;
use graphics::Renderer;

mod assets;
use assets::AssetManager;

mod gui;

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
            gui_renderer.draw(
                &mut renderer.device,
                &mut renderer.queue,
                &output.view,
                crate::gui::renderables::Renderable::Group {
                    renderables: 
                        vec![
                            crate::gui::renderables::Renderable::Quad {
                                background: crate::gui::core::Background::Color(Color::from_rgb8(
                                    0x36, 0x39, 0x3F,
                                )),
                                border_radius: 0,
                                border_width: 0,
                                border_color: [0.7, 0.7, 0.7].into(),
                                bounds: crate::gui::core::Rectangle {
                                    x: 0.0,
                                    y: 0.0,
                                    width: 1024.0,
                                    height: 768.0,
                                }
                            },
                            crate::gui::renderables::Renderable::Quad {
                                background: crate::gui::core::Background::from(ACTIVE),
                                border_radius: 2,
                                border_width: 1,
                                border_color: [0.7, 0.7, 0.7].into(),
                                bounds: crate::gui::core::Rectangle {
                                    x: 50.0,
                                    y: 50.0,
                                    width: 256.0,
                                    height: 256.0,
                                }
                            },
                        ]
                },
                renderer.window.scale_factor() as f32,
            );
        },
        _ => (),
    });
}