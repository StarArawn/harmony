use std::sync::Arc;
use legion::prelude::*;

use super::RenderTarget;
use crate::scene::components::CameraData;
use nalgebra_glm::Vec3;
//use crate::graphics::systems::create_render_schedule_builder;

pub struct CurrentRenderTarget(pub Option<Arc<RenderTarget>>);
pub struct RenderTargetDepth(pub u32);

#[derive(Debug, Copy, Clone)]
pub enum ProbeFormat {
    RGBA16,
    RGBA32,
}

impl Into<wgpu::TextureFormat> for ProbeFormat {
    fn into(self) -> wgpu::TextureFormat {
        match self {
            ProbeFormat::RGBA16 => wgpu::TextureFormat::Rgba16Float,
            ProbeFormat::RGBA32 => wgpu::TextureFormat::Rgba32Float,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ProbeQuality {
    Low,
    Medium,
    High,
}

impl ProbeQuality {
    pub(crate) fn get_irradiance_resoultion(&self) -> u32 {
        match self {
            ProbeQuality::Low => 64,
            ProbeQuality::Medium => 128,
            ProbeQuality::High => 256,
        }
    }

    pub(crate) fn get_probe_resoultion(&self) -> u32 {
        match self {
            ProbeQuality::Low => 512,
            ProbeQuality::Medium => 1024,
            ProbeQuality::High => 2048,
        }
    }

    pub(crate) fn get_specular_resoultion(&self) -> u32 {
        match self {
            ProbeQuality::Low => 256,
            ProbeQuality::Medium => 512,
            ProbeQuality::High => 1024,
        }
    }

    pub(crate) fn get_sample_count(&self) -> u32 {
        match self {
            ProbeQuality::Low => 512,
            ProbeQuality::Medium => 1024,
            ProbeQuality::High => 2048,
        }
    }
}

pub struct Probe {
    pub position: Vec3,
    pub quality: ProbeQuality,
    pub format: ProbeFormat,
    sample_offset: u32,
    samples_per_frame: u32,
    sample_count: u32,
    scale: f32,
    irradiance_resoultion: u32,
    specular_resoultion: u32,
    probe_cube: Arc<RenderTarget>,
    irradiance_target: RenderTarget,
    specular_target: RenderTarget,
    reference_texture: Option<wgpu::Texture>,
    has_rendered: bool,
}

impl Probe {
    pub fn new(position: Vec3, device: &wgpu::Device, reference_texture: Option<wgpu::Texture>, quality: ProbeQuality, format: ProbeFormat) -> Self {
        let sample_offset = 0;
        let samples_per_frame = 512;
        let sample_count = 1024;
        let scale = 2.0;
        let probe_resoultion = quality.get_probe_resoultion();
        let irradiance_resoultion = quality.get_irradiance_resoultion();
        let specular_resoultion = quality.get_specular_resoultion();

        let wgpu_format: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb; //format.into();
        let probe_cube = RenderTarget::new(device, probe_resoultion as f32, probe_resoultion as f32, 6, 1, wgpu_format, wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_SRC | wgpu::TextureUsage::OUTPUT_ATTACHMENT);
        let irradiance_target = RenderTarget::new(device, irradiance_resoultion as f32, irradiance_resoultion as f32, 6, 1, wgpu_format, wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_SRC | wgpu::TextureUsage::OUTPUT_ATTACHMENT);
        let specular_target = RenderTarget::new(device, specular_resoultion as f32, specular_resoultion as f32, 6, 1, wgpu_format, wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_SRC | wgpu::TextureUsage::OUTPUT_ATTACHMENT);

        Self {
            position,
            format,
            has_rendered: false,
            irradiance_resoultion,
            irradiance_target,
            probe_cube: Arc::new(probe_cube),
            quality,
            reference_texture,
            sample_count,
            sample_offset,
            samples_per_frame,
            scale,
            specular_resoultion,
            specular_target,
        }
    }

    // Render's scene to the cube
    // This is considered a very "HEAVY" operation, and shouldn't be treated lightly
    // TODO: If wgpu ever adds multi-view's use that instead..
    pub(crate) fn render(&mut self, resources: &mut Resources, scene: &mut crate::scene::Scene) {
        // If we already rendered don't do it again.
        //if self.has_rendered { return; }

        // Insert the cube as the current render target.
        resources.insert(CurrentRenderTarget(Some(self.probe_cube.clone())));

        // Create new render schedule has to be different from normal as we want to not queue items up right away.
        // TODO: Have more systems support our CurrentRenderTarget.
        let mut render_schedule = Schedule::builder() //create_render_schedule_builder()
            .add_system(crate::graphics::systems::skybox::create())
            .flush()
            .build();
        
        {
            let mut width = 0.0;
            let mut height = 0.0;
            let camera_query = <(Write<CameraData>, )>::query();
            for (mut camera_data, ) in camera_query.iter_mut(&mut scene.world) {
                if camera_data.active {
                    width = camera_data.width;
                    height = camera_data.height;
                    camera_data.active = false;
                }
            }
                
            // Add our special camera to the scene.
            let camera = CameraData::new_perspective(90.0, width, height, 0.01, 1000.0);
            scene.world.insert((), vec![(camera, )]);
            
            // Order of faces: X+ X- Y+ Y- Z+ Z-
            // Render scene to each face..
            for i in 0..6 {
                // Update camera with new view
                let camera_query = <(Write<CameraData>, )>::query();

                for (mut camera_data, ) in camera_query.iter_mut(&mut scene.world) {
                    if camera_data.active {
                        Self::update_camera(self.position, &mut camera_data, i);
                    }
                }
                // Tell the render target the current face..
                resources.insert(RenderTargetDepth(i));
                render_schedule.execute(&mut scene.world, resources);
            }

            // Finally we submit our queue.
            let mut queue_schedule = Schedule::builder()
                .flush()
                .add_thread_local_fn(crate::graphics::systems::render::create())
                .build();
            queue_schedule.execute(&mut scene.world, resources);
        }

        // Remove camera_enttiy
        {
            let mut command = CommandBuffer::new(&scene.world);
            let camera_query = <(Read<CameraData>, )>::query();
            for (entity, camera_data) in camera_query.iter_entities_mut(&mut scene.world) {
                if camera_data.0.active {
                    command.remove_component::<CameraData>(entity);
                }
            }
            command.write(&mut scene.world);

            // And reactiveate all cameras deactived.. 
            // TODO: Figure out how to tell which camera was actually activated before this..
            let camera_query = <(Write<CameraData>, )>::query();
            for (mut camera_data, ) in camera_query.iter_mut(&mut scene.world) {
                camera_data.active = true;
            }
        }

        resources.insert(CurrentRenderTarget(None));

        self.has_rendered = true;
    }

    fn update_camera(position: Vec3, camera: &mut CameraData, face_id: u32) {
        match face_id {
            0 => { camera.pitch = 0.0; camera.yaw = 90.0; }, // X+
            1 => { camera.pitch = 0.0; camera.yaw = -90.0; }, // X-
            2 => { camera.pitch = -90.0; camera.yaw = 180.0; }, // Y+
            3 => { camera.pitch = 90.0; camera.yaw = 180.0; }, // Y-
            4 => { camera.pitch = 0.0; camera.yaw = 180.0; }, // Z+
            5 => { camera.pitch = 0.0; camera.yaw = 0.0; }, // Z-
            _ => (),
        }
        // Because pitch and yaw are in radians..
        camera.pitch = camera.pitch.to_radians();
        camera.yaw = camera.yaw.to_radians();
        let eye = position
        + nalgebra::Vector3::new(
                camera.yaw.sin() * camera.pitch.cos(),
                camera.pitch.sin(),
                camera.yaw.cos() * camera.pitch.cos(),
            );
            camera.position = eye;
            camera.update_view(eye, position, Vec3::new(0.0, 1.0, 0.0));
    }

}