use rendy::{
    factory::{ Config },
    init::winit::{
        self,
        event_loop::{ EventLoop },
        window::{ WindowBuilder },
    },
};

#[cfg(any(feature = "dx12", feature = "metal", feature = "vulkan"))]
fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Warn)
        .filter_module("harmony", log::LevelFilter::Info)
        .init();

    let config: Config = Default::default();

    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("rendy-pbr")
        .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 960.0));

    let rendy = rendy::init::AnyWindowedRendy::init_auto(&config, window, &event_loop).unwrap();
    rendy::with_any_windowed_rendy!((rendy)
        (factory, families, surface, window) => {
            // TODO: Do stuff here.
        }
    )
}

#[cfg(not(any(feature = "dx12", feature = "metal", feature = "vulkan")))]
fn main() {
    panic!("Specify feature: { dx12, metal, vulkan }");
}