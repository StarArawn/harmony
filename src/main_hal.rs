#[cfg(feature = "dx12")]
use gfx_backend_dx12 as back;
#[cfg(feature = "metal")]
use gfx_backend_metal as back;
#[cfg(feature = "vulkan")]
use gfx_backend_vulkan as back;

use gfx_hal::Instance;

use winit::{ 
    dpi::LogicalSize,
    event::{ Event, WindowEvent },
    event_loop::{ ControlFlow },
};

#[cfg(any(
    feature = "vulkan",
    feature = "dx11",
    feature = "dx12",
    feature = "metal",
    feature = "gl",
    feature = "wgl"
))]
use gfx_hal::window::Suboptimal;

mod graphics_hal;
use graphics_hal::Renderer;

mod winit_state;
use winit_state::WinitState;

struct WindowSize {
    width: u32,
    height: u32,
}

const WINDOW_SIZE: WindowSize = WindowSize {
    width: 1024,
    height: 768,
};

fn main() {
    let winit_state = WinitState::new("Harmony", LogicalSize::new(WINDOW_SIZE.width, WINDOW_SIZE.height)).unwrap();

    let (instance, mut adapters, surface) = {
        let instance =
            back::Instance::create("Harmony", 1).expect("Failed to create an instance!");
        let surface = unsafe {
            instance
                .create_surface(&winit_state.window)
                .expect("Failed to create a surface!")
        };
        let adapters = instance.enumerate_adapters();
        // Return `window` so it is not dropped: dropping it invalidates `surface`.
        (Some(instance), adapters, surface)
    };
    #[cfg(feature = "gl")]
    let (_window, instance, mut adapters, surface) = {
        #[cfg(not(target_arch = "wasm32"))]
        let (window, surface) = {
            let builder =
                back::config_context(back::glutin::ContextBuilder::new(), ColorFormat::SELF, None)
                    .with_vsync(true);
            let windowed_context = builder.build_windowed(wb, &event_loop).unwrap();
            let (context, window) = unsafe {
                windowed_context
                    .make_current()
                    .expect("Unable to make context current")
                    .split()
            };
            let surface = back::Surface::from_context(context);
            (window, surface)
        };
        #[cfg(target_arch = "wasm32")]
        let (window, surface) = {
            let window = wb.build(&event_loop).unwrap();
            web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .body()
                .unwrap()
                .append_child(&winit::platform::web::WindowExtWebSys::canvas(&window));
            let surface = back::Surface::from_raw_handle(&window);
            (window, surface)
        };

        let adapters = surface.enumerate_adapters();
        (window, None, adapters, surface)
    };

    for adapter in &adapters {
        println!("{:?}", adapter.info);
    }

    let adapter = adapters.remove(0);

    let mut renderer = Renderer::new(
        instance,
        surface,
        adapter,
        WINDOW_SIZE.width,
        WINDOW_SIZE.height,
    );

    winit_state.events_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { event, .. } => match event {
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
                #[cfg(all(feature = "gl", not(target_arch = "wasm32")))]
                {
                    let context = renderer.surface.context();
                    context.resize(dims);
                }
                renderer.dimensions = gfx_hal::window::Extent2D {
                    width: dims.width,
                    height: dims.height,
                };
                renderer.recreate_swapchain();
            },
            _ => {}
        },
        Event::MainEventsCleared => {
            gui.update();

            // and request a redraw
            window.request_redraw();
        }
        Event::RedrawEventsCleared => {
            // if let Err(e) = do_the_render(&mut hal_state) {
            //     *control_flow = ControlFlow::Exit;
            // }
            renderer.render();
        },
        _ => (),
      });
}