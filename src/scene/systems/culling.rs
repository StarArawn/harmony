use legion::prelude::*;
use nalgebra_glm::Vec4;

use crate::{
    scene::components,
};

pub fn create() -> Box<dyn Schedulable> {
    SystemBuilder::new("culling")
        .with_query(<Read<components::CameraData>>::query())
        .with_query(<(Write<components::Transform>, Read<components::Mesh>)>::query())
        .build(
            |_, mut world, _, (camera_query, transform_mesh_query)| {
                // TODO: store and display this stat somewhere..
                let mut total = 0;
                let camera_frustum = {
                    let filtered_camera_data: Vec<_> =
                        camera_query
                            .iter(&world)
                            .filter(|camera| camera.cull)
                            .collect();
                        let camera_data: Option<&legion::borrow::Ref<'_, components::CameraData>
                    > = filtered_camera_data.first();
                    
                    if camera_data.is_none() {
                        return;
                    }
                    camera_data.unwrap().frustum.clone()
                };

                for (mut transform, mesh) in transform_mesh_query.iter_mut(&mut world) {
                    let mesh = mesh.mesh_handle.get();

                    if mesh.is_err() {
                        continue;
                    }

                    let mesh = mesh.unwrap();
                    
                    let mut bounding_sphere = mesh.bounding_sphere.clone();
                    bounding_sphere.center = (transform.matrix * Vec4::new(bounding_sphere.center.x, bounding_sphere.center.y, bounding_sphere.center.z, 1.0)).xyz();
                    transform.cull = !camera_frustum.contains_sphere(bounding_sphere);
                    if transform.cull {
                        total += 1;
                    }
                }
           })
}