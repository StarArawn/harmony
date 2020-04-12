pub mod default;

mod component;
pub use component::Component;


mod animation;
mod window;
mod padding;
mod text;
mod log;

pub use animation::{ AnimationBuilder, Animation};
pub use window::{ Window, WindowBuilder };
pub use padding::{ Padding, PaddingBuilder };
pub use text::Text;
pub use crate::gui::components::log::Log;