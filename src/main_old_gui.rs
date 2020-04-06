#![warn(rust_2018_idioms)]
#![allow(dead_code)]
#![allow(clippy::module_inception)]
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
use gui::Gui;

struct WindowSize {
    width: u32,
    height: u32,
}

const WINDOW_SIZE: WindowSize = WindowSize {
    width: 1024,
    height: 768,
};

fn main() {
    let mut modifiers = ModifiersState::default();

    let (wb, events_loop) = WinitState::create("Harmony", LogicalSize::new(WINDOW_SIZE.width, WINDOW_SIZE.height));

    let (window, size, surface) = {
        let window = wb.build(&events_loop).unwrap();
        let size = window.inner_size();
        let surface = wgpu::Surface::create(&window);
        (window, size, surface)
    };

    let logical_size = window.inner_size().to_logical(window.scale_factor());

    let mut renderer = Renderer::new(window, size, surface);

    let asset_path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/").to_string();
    let mut asset_manager = AssetManager::new(asset_path);
    asset_manager.load(&renderer.device);

    // Gui
    let mut gui = Gui::new(&mut renderer.device, &renderer.window);

    // let console_gui = crate::gui::Console::new();
    // gui.add_scene(console_gui, logical_size);

    let theme_scene = crate::gui::ThemeScene::new();
    gui.add_scene(theme_scene, logical_size);

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

            // Map window event to iced event
            if let Some(event) = iced_winit::conversion::window_event(
                &event,
                renderer.window.scale_factor(),
                modifiers,
            ) {
                gui.events.push(event);
            }
        },
        Event::RedrawEventsCleared => {
            gui.update(logical_size);
            renderer.window.request_redraw();
        },
        Event::RedrawRequested(_) => {
            renderer.render(&mut gui, renderer.size);
        },
        _ => (),
    });
}