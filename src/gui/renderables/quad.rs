use zerocopy::AsBytes;

#[repr(C)]
#[derive(Debug, AsBytes, Clone, Copy)]
pub struct Quad {
    pub position: [f32; 2],
    pub scale: [f32; 2],
    pub color: [f32; 4],
    pub border_color: [f32; 4],
    pub border_radius: f32,
    pub border_width: f32,
}

impl Quad {
    pub const MAX: usize = 100_000;
}
