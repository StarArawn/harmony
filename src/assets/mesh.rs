use super::{
    file_manager::AssetHandle,
    material::{PBRMaterial, PBRMaterialRon},
    material_manager::MaterialManager,
};
use bytemuck::{Pod, Zeroable};
use nalgebra_glm::{Vec2, Vec3, Vec4};
use std::{
    collections::HashMap,
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::Arc,
};
use crate::core::BoundingSphere;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MeshVertexData {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
    pub tangent: Vec4,
}

impl Default for MeshVertexData {
    fn default() -> Self {
        Self {
            position: Vec3::zeros(),
            normal: Vec3::zeros(),
            uv: Vec2::zeros(),
            tangent: Vec4::zeros(),
        }
    }
}

// We implement these traits so our vertex struct can be converted into bytes.
unsafe impl Zeroable for MeshVertexData {}
unsafe impl Pod for MeshVertexData {}

pub struct SubMesh {
    pub vertices: Vec<MeshVertexData>,
    indices: Vec<u32>,
    pub(crate) index_count: usize,
    mode: wgpu::PrimitiveTopology,
    pub(crate) vertex_buffer: Option<Arc<wgpu::Buffer>>,
    pub(crate) index_buffer: Arc<wgpu::Buffer>,
    pub bounding_sphere: BoundingSphere,
}

impl std::fmt::Debug for SubMesh {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SubMesh")
            .field("vertices", &self.vertices)
            .field("indices", &self.indices)
            .field("index_count", &self.index_count)
            .field("mode", &self.mode)
            .finish()
    }
}

#[derive(Debug)]
pub struct Mesh {
    pub name: String,
    pub meshes: HashMap<Arc<AssetHandle<PBRMaterial>>, SubMesh>,
    pub bounding_sphere: BoundingSphere,
}

#[derive(Debug)]
pub struct Gltf {
    pub meshes: Vec<Mesh>,
    pub bounding_sphere: BoundingSphere,
}

impl Gltf {
    pub async fn from_gltf(
        device: Arc<wgpu::Device>,
        material_manager: Arc<MaterialManager<PBRMaterialRon>>,
        path: PathBuf,
    ) -> Gltf {
        let data: Vec<u8> = async_std::fs::read(path.clone()).await.unwrap();

        let document = gltf::Gltf::from_slice(&data).unwrap();

        let files = document
            .buffers()
            .filter_map(|buffer| match buffer.source() {
                gltf::buffer::Source::Bin => None,
                gltf::buffer::Source::Uri(uri) => Some(uri.to_string()),
            })
            .collect::<Vec<_>>();

        let mut buffer_data = Vec::new();
        for file in files {
            let buffer_path = path.clone().parent().unwrap().join(file);
            let file = async_std::fs::read(buffer_path).await.unwrap();
            buffer_data.push(gltf::buffer::Data(file));
        }

        let get_buffer_data =
            |buffer: gltf::Buffer<'_>| buffer_data.get(buffer.index()).map(|x| &*x.0);

        let gltf_meshes = document.meshes().collect::<Vec<gltf::Mesh<'_>>>();

        let mut meshes = Vec::new();

        for gltf_mesh in gltf_meshes {
            let name = gltf_mesh.name().unwrap_or("mesh").to_string();
            let primitives = gltf_mesh.primitives();

            let images: Vec<gltf::Image<'_>> = document.images().collect();

            let mut mesh = Mesh {
                name,
                meshes: HashMap::new(),
                bounding_sphere: BoundingSphere::new(),
            };

            for primitive in primitives {
                let reader = primitive.reader(get_buffer_data);
                let positions: Vec<_> = reader
                    .read_positions()
                    .map(|iter| iter.collect())
                    .ok_or(format!("mesh primitive is missing positions"))
                    .unwrap();

                let mut vertices: Vec<MeshVertexData> = positions
                    .iter()
                    .map(|pos| MeshVertexData {
                        position: Vec3::from(pos.clone()),
                        ..MeshVertexData::default()
                    })
                    .collect();

                if let Some(normals) = reader.read_normals() {
                    for (i, normal) in normals.enumerate() {
                        vertices[i].normal = Vec3::from(normal.clone());
                    }
                }
                if let Some(uvs) = reader.read_tex_coords(0) {
                    for (i, uv) in uvs.into_f32().enumerate() {
                        vertices[i].uv = Vec2::from(uv.clone());
                    }
                }

                let mut had_tangents = false;
                // Load tangents if we have them.
                if let Some(tangents) = reader.read_tangents() {
                    for (i, tangent) in tangents.enumerate() {
                        vertices[i].tangent =
                            Vec4::new(tangent[0], tangent[1], tangent[2], tangent[3]);
                    }
                    had_tangents = true;
                }

                let indices: Vec<u32> = if let Some(index_enum) = reader.read_indices() {
                    index_enum.into_u32().collect()
                } else {
                    panic!("model doesn't have indices");
                };

                let gltf_material: gltf::Material<'_> = primitive.material();
                let pbr = gltf_material.pbr_metallic_roughness();

                let color_factor = pbr.base_color_factor();
                let color = Vec4::new(
                    color_factor[0],
                    color_factor[1],
                    color_factor[2],
                    color_factor[3],
                );

                let main_info = pbr.base_color_texture();
                let mut normal_texture = None;
                let normals_texture = gltf_material.normal_texture();
                if normals_texture.is_some() {
                    let normal_source = normals_texture.unwrap().texture().source().source();
                    match normal_source {
                        gltf::image::Source::Uri { uri, .. } => {
                            let texture_file_name = Some(
                                Path::new(&uri)
                                    .file_name()
                                    .and_then(OsStr::to_str)
                                    .unwrap()
                                    .to_string(),
                            );
                            if texture_file_name.is_some() {
                                normal_texture = Some(texture_file_name.unwrap());
                            }
                        }
                        _ => (),
                    }
                }
                let roughness_info = pbr.metallic_roughness_texture();
                let roughness = pbr.roughness_factor();
                let metallic = pbr.metallic_factor();

                let main_texture = Self::get_texture_url(&main_info, &images);
                let roughness_texture = Self::get_texture_url(&roughness_info, &images);

                let has_pbr_texture = roughness_texture.is_some();

                let material = PBRMaterialRon {
                    main_texture: main_texture.unwrap_or("core/white.png".to_string()),
                    normal_texture: normal_texture.unwrap_or("core/empty_normal.png".to_string()),
                    roughness_texture: roughness_texture.unwrap_or("core/pbr_flat.png".to_string()),
                    roughness,
                    metallic,
                    roughness_override: if has_pbr_texture { 0.0 } else { 1.0 },
                    metallic_override: if has_pbr_texture { 0.0 } else { 1.0 },
                    color,
                };
                let material_handle = material_manager.insert(material, path.clone());
                
                let primitive_topology = Self::get_primitive_mode(primitive.mode());

                let index_buffer = Arc::new(device.create_buffer_with_data(
                    &bytemuck::cast_slice(&indices),
                    wgpu::BufferUsage::INDEX,
                ));
                let index_count = indices.len();

                let bounding_sphere = BoundingSphere::from_points(vertices.iter().map(|x| x.position).collect());

                let mut sub_mesh = SubMesh {
                    vertices,
                    indices,
                    index_count,
                    mode: primitive_topology,
                    vertex_buffer: None,
                    index_buffer,
                    bounding_sphere,
                };

                if !had_tangents {
                    log::info!("No tangents found generating tangents instead!",);
                    mikktspace::generate_tangents(&mut sub_mesh);
                }

                let vertex_buffer = device.create_buffer_with_data(
                    &bytemuck::cast_slice(&sub_mesh.vertices),
                    wgpu::BufferUsage::VERTEX,
                );
                sub_mesh.vertex_buffer = Some(Arc::new(vertex_buffer));

                mesh.meshes.insert(material_handle, sub_mesh);
            }

            mesh.bounding_sphere = BoundingSphere::from_bounding_spheres(mesh.meshes.values().map(|x| &x.bounding_sphere).collect());

            meshes.push(mesh);
        }

        let bounding_sphere = BoundingSphere::from_bounding_spheres(meshes.iter().map(|x| &x.bounding_sphere).collect());

        Gltf { meshes, bounding_sphere }
    }

    fn get_primitive_mode(mode: gltf::mesh::Mode) -> wgpu::PrimitiveTopology {
        match mode {
            gltf::mesh::Mode::Points => wgpu::PrimitiveTopology::PointList,
            gltf::mesh::Mode::Lines => wgpu::PrimitiveTopology::LineList,
            gltf::mesh::Mode::LineStrip => wgpu::PrimitiveTopology::LineStrip,
            gltf::mesh::Mode::Triangles => wgpu::PrimitiveTopology::TriangleList,
            gltf::mesh::Mode::TriangleStrip => wgpu::PrimitiveTopology::TriangleStrip,
            _ => panic!("Error loading mesh topology isn't supported!"),
        }
    }

    fn get_texture_url(
        info: &Option<gltf::texture::Info<'_>>,
        images: &Vec<gltf::Image<'_>>,
    ) -> Option<String> {
        let mut file_name = None;
        if info.is_some() {
            let info = info.as_ref().unwrap();
            let tex = info.texture();

            let image: Option<&gltf::Image<'_>> = images.get(tex.index());
            if image.is_some() {
                let image = image.unwrap();
                let source = image.source();
                match source {
                    gltf::image::Source::Uri { uri, .. } => {
                        let texture_file_name = Some(
                            Path::new(&uri)
                                .to_str()    
                                .unwrap()
                                .to_string(),
                        );
                        if texture_file_name.is_some() {
                            file_name = Some(texture_file_name.unwrap());
                        }
                    }
                    _ => (),
                }
            }
        }
        file_name
    }
}

fn vertex(sub_mesh: &SubMesh, face: usize, vert: usize) -> &MeshVertexData {
    &sub_mesh.vertices[sub_mesh.indices[face * 3 + vert] as usize]
}

fn vertex_mut(sub_mesh: &mut SubMesh, face: usize, vert: usize) -> &mut MeshVertexData {
    &mut sub_mesh.vertices[sub_mesh.indices[face * 3 + vert] as usize]
}

impl mikktspace::Geometry for SubMesh {
    fn num_faces(&self) -> usize {
        self.indices.len() / 3
    }

    fn num_vertices_of_face(&self, _face: usize) -> usize {
        3
    }

    fn position(&self, face: usize, vert: usize) -> [f32; 3] {
        vertex(self, face, vert).position.into()
    }

    fn normal(&self, face: usize, vert: usize) -> [f32; 3] {
        vertex(self, face, vert).normal.into()
    }

    fn tex_coord(&self, face: usize, vert: usize) -> [f32; 2] {
        vertex(self, face, vert).uv.into()
    }

    fn set_tangent_encoded(&mut self, tangent: [f32; 4], face: usize, vert: usize) {
        vertex_mut(self, face, vert).tangent = tangent.into();
    }
}

#[cfg(test)]
mod tests {
    use super::Gltf;
    use crate::{
        assets::{material_manager::MaterialManager, texture_manager::TextureManager},
        graphics::{pipelines::pbr::create_pbr_bindgroup_layout, resources::GPUResourceManager},
    };
    use std::{path::PathBuf, sync::Arc};

    #[test]
    fn should_load_mesh() {
        futures::executor::block_on(async {
            let (_, device, queue) = async_std::task::block_on(async {
                let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
                let adapter = instance
                    .request_adapter(
                        &wgpu::RequestAdapterOptions {
                            power_preference: wgpu::PowerPreference::Default,
                            compatible_surface: None,
                        },
                    )
                    .await
                    .unwrap();

                let adapter_features = adapter.features();
                let (device, queue) = adapter
                    .request_device(
                        &wgpu::DeviceDescriptor {
                            features: adapter_features,
                            limits: wgpu::Limits::default(),
                            shader_validation: true,
                        },
                        None,
                    )
                    .await
                    .unwrap();
                let arc_device = Arc::new(device);
                let arc_queue = Arc::new(queue);
                (adapter, arc_device, arc_queue)
            });

            let texture_manager = TextureManager::new(device.clone(), queue.clone());

            let gpu_resource_manager = Arc::new(GPUResourceManager::new(device.clone()));

            let pbr_bind_group_layout = create_pbr_bindgroup_layout(device.clone());
            gpu_resource_manager
                .add_bind_group_layout("pbr_material_layout", pbr_bind_group_layout);

            let material_manager = Arc::new(MaterialManager::new(
                device.clone(),
                queue,
                Arc::new(texture_manager),
                gpu_resource_manager,
                PathBuf::from("./"),
            ));

            let _mesh = Gltf::from_gltf(
                device.clone(),
                material_manager,
                PathBuf::from("./assets/example/meshes/cube/cube.gltf"),
            )
            .await;
        });
    }
}
