use crate::{
    graphics::pipelines::GlobalUniforms,
    scene::components::{transform::LocalUniform, CameraData, Transform},
};
use specs::{ReadStorage, System, WriteStorage};

pub struct PrepareUnlit<'a> {
    pub(crate) device: &'a wgpu::Device,
    pub(crate) encoder: &'a mut wgpu::CommandEncoder,
    pub(crate) constants_buffer: &'a wgpu::Buffer,
}

impl<'a> System<'a> for PrepareUnlit<'a> {
    type SystemData = (ReadStorage<'a, CameraData>, WriteStorage<'a, Transform>);

    fn run(&mut self, (camera_data, mut transforms): Self::SystemData) {
        use specs::Join;
        if transforms.count() == 0 {
            return;
        }
        let filtered_camera_data: Vec<&CameraData> = camera_data
            .join()
            .filter(|data: &&CameraData| data.active)
            .collect();
        let camera_data = filtered_camera_data.first();

        if camera_data.is_none() {
            return;
        }

        let camera_data = camera_data.unwrap();
        let camera_matrix = camera_data.get_matrix();

        let uniforms = GlobalUniforms {
            view_projection: camera_matrix,
        };

        let constants_buffer = self
            .device
            .create_buffer_with_data(bytemuck::bytes_of(&uniforms), wgpu::BufferUsage::COPY_SRC);

        self.encoder.copy_buffer_to_buffer(
            &constants_buffer,
            0,
            &self.constants_buffer,
            0,
            std::mem::size_of::<GlobalUniforms>() as u64,
        );

        {
            let size = std::mem::size_of::<LocalUniform>();
            let mut temp_buf_data = self.device.create_buffer_mapped(&wgpu::BufferDescriptor {
                size: (transforms.count() * size) as u64,
                usage: wgpu::BufferUsage::COPY_SRC,
                label: None,
            });

            // FIXME: Align and use `LayoutVerified`
            for (transform, slot) in (&mut transforms)
                .join()
                .zip(temp_buf_data.data().chunks_exact_mut(size))
            {
                transform.update();
                slot.copy_from_slice(bytemuck::bytes_of(&LocalUniform {
                    world: transform.matrix,
                }));
            }

            let temp_buf = temp_buf_data.finish();

            let mut i = 0;
            for transform in transforms.join() {
                self.encoder.copy_buffer_to_buffer(
                    &temp_buf,
                    (i * size) as wgpu::BufferAddress,
                    &transform.local_buffer,
                    0,
                    size as wgpu::BufferAddress,
                );
                i += 1;
            }
        }
    }
}
