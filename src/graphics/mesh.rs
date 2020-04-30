use super::material::{PBRMaterial};
use crate::graphics::material::Material;
use bytemuck::{Pod, Zeroable};
use nalgebra_glm::{Vec2, Vec3, Vec4};
use std::ffi::OsStr;
use std::path::Path;
use log::warn;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MeshVertexData {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
    pub tangent: Vec4,
}

// We implement these traits so our vertex struct can be converted into bytes.
unsafe impl Zeroable for MeshVertexData {}
unsafe impl Pod for MeshVertexData {}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MeshTangentLine {
    pub pos: Vec3,   
    pub color: Vec3,
}

unsafe impl Zeroable for MeshTangentLine {}
unsafe impl Pod for MeshTangentLine {}

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

#[derive(Debug)]
pub struct SubMesh {
    pub vertices: Vec<MeshVertexData>,
    pub tangent_lines: Vec<MeshTangentLine>,
    indices: Vec<u32>,
    pub(crate) index_count: usize,
    mode: wgpu::PrimitiveTopology,
    material_id: Option<usize>,
    pub(crate) vertex_buffer: wgpu::Buffer,
    pub(crate) tangent_line_buffer: wgpu::Buffer,
    pub(crate) index_buffer: wgpu::Buffer,

    // Material index is stored here.
    pub(crate) material_index: u32,
}

fn vertex(sub_mesh: &SubMesh, face: usize, vert: usize) -> &MeshVertexData{
    &sub_mesh.vertices[sub_mesh.indices[(face * 3) + vert] as usize]
}

fn vertex_mut(sub_mesh: &mut SubMesh, face: usize, vert: usize) -> &mut MeshVertexData{
    &mut sub_mesh.vertices[sub_mesh.indices[(face * 3) + vert] as usize]
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

pub struct Mesh {
    pub sub_meshes: Vec<SubMesh>,
}

impl Mesh {
    /// Imports glTF 2.0
    pub fn new<T>(
        device: &wgpu::Device,
        path: T,
        material_start_index: u32,
    ) -> (Mesh, Vec<Material>)
    where
        T: Into<String>,
    {
        let mut materials = Vec::new();
        let cloned_path = path.into().clone();
        let (document, data, _) =
            gltf::import(cloned_path).expect("Loaded the gltf file successfully!");
        let get_buffer_data = |buffer: gltf::Buffer<'_>| data.get(buffer.index()).map(|x| &*x.0);

        // let mut meshes = Vec::new();
        let meshes = document.meshes().collect::<Vec<gltf::Mesh<'_>>>();
        if meshes.len() > 1 {
            log::warn!("Currently we only support 1 mesh per gltf object. If you have more than one it will not be rendered.");
        }
        // For now we only support 1 mesh.
        let gltf_mesh: &gltf::Mesh<'_> = meshes.first().unwrap();

        let mut sub_meshes = Vec::new();
        let primitives = gltf_mesh.primitives();

        let images: Vec<gltf::Image<'_>> = document.images().collect();

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

            // Load tangents if we have them.
            if let Some(tangents) = reader.read_tangents() {
                for (i, tangent) in tangents.enumerate() {
                    vertices[i].tangent = Vec4::new(tangent[0], tangent[1], tangent[2], tangent[3]);
                }
            } else {
                // TODO: Calculate tangents if we don't have them.
                //warn!("Don't have tangents for mesh.");
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
            let mut normal_texture= None;
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


            let main_texture = Self::get_texture_url(&main_info, &images);
            let roughness_texture = Self::get_texture_url(&roughness_info, &images);
            
            let material_index = material_start_index + materials.len() as u32;
            let material = PBRMaterial::new(
                main_texture.unwrap_or("white.png".to_string()),
                normal_texture.unwrap_or("white.png".to_string()),
                roughness_texture.unwrap_or("white.png".to_string()),
                color,
                material_index,
            );
            materials.push(Material::PBR(material));

            // mesh.calculate_tangents();

            let primitive_topology = Self::get_primitive_mode(primitive.mode());

            let tangents: Vec<(Vec3, Vec3)> = vertices.iter().map(|data| (data.tangent.xyz() * data.tangent.w, data.position)).collect();
            let mut tangent_lines = Vec::new();
            // for (tangent, position) in tangents.iter() {
            //     let position: Vec3 = position.clone();// * 50.0;
            //     let tangent: Vec3 = tangent.clone();
            //     let scale: f32 = 0.1;
            //     let vec3_tangent: Vec3 =  Vec3::new(position.x + (tangent.x * scale), position.y + (tangent.y * scale), position.z + (tangent.z * scale));
            //     tangent_lines.push(MeshTangentLine { pos: position.clone(), color: 0.5 * (tangent + Vec3::new(1.0, 1.0, 1.0)) });
            //     tangent_lines.push(MeshTangentLine { pos: vec3_tangent, color: 0.5 * (tangent + Vec3::new(1.0, 1.0, 1.0)) });
            // }
            tangent_lines.push(MeshTangentLine { pos: Vec3::new(0.0, 0.0, 0.0), color: Vec3::new(0.0, 0.0, 1.0) });
            tangent_lines.push(MeshTangentLine { pos: Vec3::new(0.0, 0.0, 5.0), color: Vec3::new(0.0, 0.0, 1.0) });
            tangent_lines.push(MeshTangentLine { pos: Vec3::new(0.0, 0.0, 0.0), color: Vec3::new(0.0, 1.0, 0.0) });
            tangent_lines.push(MeshTangentLine { pos: Vec3::new(0.0, 5.0, 0.0), color: Vec3::new(0.0, 1.0, 0.0) });
            tangent_lines.push(MeshTangentLine { pos: Vec3::new(0.0, 0.0, 0.0), color: Vec3::new(1.0, 0.0, 0.0) });
            tangent_lines.push(MeshTangentLine { pos: Vec3::new(5.0, 0.0, 0.0), color: Vec3::new(1.0, 0.0, 0.0) });
            let tangent_line_buffer = device.create_buffer_with_data(
                &bytemuck::cast_slice(&tangent_lines),
                wgpu::BufferUsage::VERTEX,
            );

            let vertex_buffer = device.create_buffer_with_data(
                &bytemuck::cast_slice(&vertices),
                wgpu::BufferUsage::VERTEX,
            );
            let index_buffer = device
                .create_buffer_with_data(&bytemuck::cast_slice(&indices), wgpu::BufferUsage::INDEX);
            let index_count = indices.len();

            let mut sub_mesh = SubMesh {
                vertices,
                tangent_lines,
                indices,
                index_count,
                mode: primitive_topology,
                material_id: primitive.material().index(),
                vertex_buffer,
                tangent_line_buffer,
                index_buffer,
                material_index,
            };
            
            mikktspace::generate_tangents(&mut sub_mesh);

            sub_meshes.push(sub_mesh);
        }
        
        (Mesh { sub_meshes }, materials)
    }

    fn get_primitive_mode(mode: gltf::mesh::Mode) -> wgpu::PrimitiveTopology {
        match mode {
            gltf::mesh::Mode::Points => wgpu::PrimitiveTopology::PointList,
            gltf::mesh::Mode::Lines => wgpu::PrimitiveTopology::LineList,
            gltf::mesh::Mode::LineStrip => wgpu::PrimitiveTopology::LineStrip,
            gltf::mesh::Mode::Triangles => wgpu::PrimitiveTopology::TriangleList,
            gltf::mesh::Mode::TriangleStrip => wgpu::PrimitiveTopology::TriangleStrip,
            _ => panic!(format!("Error loading mesht topology isn't supported!")),
        }
    }

    fn get_texture_url(info: &Option<gltf::texture::Info<'_>>, images: &Vec<gltf::Image<'_>>) -> Option<String> {
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
                                .file_name()
                                .and_then(OsStr::to_str)
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
