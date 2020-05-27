/// A bind group that consists of a wgpu::BindGroup and an index to which slot it's bound to.
pub struct BindGroup {
    pub index: u32,
    pub group: wgpu::BindGroup,
}

impl BindGroup {
    pub fn new(bind_slot: u32, group: wgpu::BindGroup) -> Self {
        Self {
            index: bind_slot,
            group,
        }
    }
}
