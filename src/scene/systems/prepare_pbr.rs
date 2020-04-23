use crate::{
    graphics::pipelines::{
        DirectionalLight, GlobalUniforms, LightingUniform, PointLight, MAX_LIGHTS,
    },
    scene::components::{
        transform::LocalUniform, CameraData, DirectionalLightData, PointLightData, Transform,
    },
};
use nalgebra_glm::Vec4;
use specs::{ReadStorage, System, WriteStorage};
use std::convert::TryInto;

pub struct PreparePBR<'a> {
    pub(crate) device: &'a wgpu::Device,
    pub(crate) encoder: &'a mut wgpu::CommandEncoder,
    pub(crate) constants_buffer: &'a wgpu::Buffer,
    pub(crate) lighting_buffer: &'a wgpu::Buffer,
}

impl<'a> System<'a> for PreparePBR<'a> {
    type SystemData = (
        ReadStorage<'a, CameraData>,
        ReadStorage<'a, DirectionalLightData>,
        ReadStorage<'a, PointLightData>,
        WriteStorage<'a, Transform>,
    );

    fn run(
        &mut self,
        (camera_data, directional_lights, point_lights, mut transforms): Self::SystemData,
    ) {
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

        // Get lighting data.
        let mut directional_light_data_vec: Vec<DirectionalLight> = directional_lights
            .join()
            .map(|data| DirectionalLight {
                direction: Vec4::new(data.direction.x, data.direction.y, data.direction.z, 0.0),
                color: Vec4::new(data.color.x, data.color.y, data.color.z, 1.0),
            })
            .collect();

        let mut point_light_data_vec: Vec<PointLight> = (&point_lights, &transforms)
            .join()
            .map(|(data, transform)| PointLight {
                attenuation: Vec4::new(data.attenuation, 0.0, 0.0, 0.0),
                color: Vec4::new(data.color.x, data.color.y, data.color.z, 1.0),
                position: Vec4::new(
                    transform.position.x,
                    transform.position.y,
                    transform.position.z,
                    0.0,
                ),
            })
            .collect();

        let total_dir_lights = directional_light_data_vec.len() as u32;
        let total_point_lights = point_light_data_vec.len() as u32;

        // Fill in missing data if we don't have it.
        point_light_data_vec.resize_with(MAX_LIGHTS / 2, || PointLight::default());
        directional_light_data_vec.resize_with(MAX_LIGHTS / 2, || DirectionalLight::default());

        let light_uniform = LightingUniform {
            light_num: Vec4::new(total_dir_lights as f32, total_point_lights as f32, 0.0, 0.0),
            directional_lights: directional_light_data_vec.as_slice().try_into().unwrap(),
            point_lights: point_light_data_vec.as_slice().try_into().unwrap(),
        };

        let lighting_buffer = self.device.create_buffer_with_data(
            bytemuck::bytes_of(&light_uniform),
            wgpu::BufferUsage::COPY_SRC,
        );

        self.encoder.copy_buffer_to_buffer(
            &lighting_buffer,
            0,
            &self.lighting_buffer,
            0,
            std::mem::size_of::<LightingUniform>() as u64,
        );

        // Update transform buffers.
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
