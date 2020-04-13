use ultraviolet::{ Vec2, Vec3, Vec4 };
use bytemuck::{ Pod, Zeroable};

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct MeshVertexData {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
    pub tangent: Vec4,
}

// We implement these traits so our vertex struct can be converted into bytes.
// TODO: Go across the entire project and replace zerocopy with bytemuck.
unsafe impl Zeroable for MeshVertexData { }
unsafe impl Pod for MeshVertexData { }

#[derive(Debug)]
pub struct SubMesh {
    vertices: Vec<MeshVertexData>,
    indices: Vec<u32>,
    pub(crate) index_count: usize,
    mode: wgpu::PrimitiveTopology,
    material_id: Option<usize>,
    pub(crate) vertex_buffer: wgpu::Buffer,
    pub(crate) index_buffer: wgpu::Buffer,
}

pub struct Mesh {
    pub sub_meshes: Vec<SubMesh>,
}

impl Mesh {
    /// Imports glTF 2.0
    pub fn new<T>(device: &wgpu::Device, path: T) -> Mesh
    where
        T: Into<String>
    {
        let cloned_path = path.into().clone();
        let (document, data, _) = gltf::import(cloned_path).expect("Loaded the gltf file successfully!");
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

        for primitive in primitives {
            let reader = primitive.reader(get_buffer_data);
            let positions: Vec<_> = reader
                .read_positions()
                .map(|iter| iter.collect())
                .ok_or(format!("mesh primitive is missing positions")).unwrap();

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
                    vertices[i].tangent = Vec4::from(tangent.clone());
                }
            } else {
                // TODO: Calculate tangents if we don't have them.
            }
    
            let indices: Vec<u32> = if let Some(index_enum) = reader.read_indices()
            {
                index_enum.into_u32().collect()
            } else {
                panic!("model doesn't have indices");
            };
    
            // let mut mesh = Mesh { indices, vertices };
            // mesh.calculate_tangents();
    
            // let mesh_data = ParsedMesh {
            //     mesh,
            //     mode: primitive.mode(),
            //     material: primitive.material().index(),
            // };
            // meshes.push(mesh_data);

            let primitive_topology = Self::get_primitive_mode(primitive.mode());

            dbg!(primitive_topology);

            let vertex_buffer = device.create_buffer_with_data(&bytemuck::cast_slice(&vertices), wgpu::BufferUsage::VERTEX);
            let index_buffer = device.create_buffer_with_data(&bytemuck::cast_slice(&indices), wgpu::BufferUsage::INDEX);
            let index_count = indices.len();
            sub_meshes.push(SubMesh {
                vertices,
                indices,
                index_count,
                mode: primitive_topology,
                material_id: primitive.material().index(),
                vertex_buffer,
                index_buffer,
            });
        }
        
        // Load materials these get stored separately.
        for material in document.materials() {
            // Kinda dumb but we can see the types if we do this..
            // rust analyzer apparently can't figure out the iterators in gltf. 
            let material: gltf::Material<'_> = material;
            // TODO: Perhaps support PBR specular workflow as well?
            let pbr = material.pbr_metallic_roughness();
            log::info!("found material {:?}", material.name());
            let _color_factor = pbr.base_color_factor();
            let _color_texture = pbr.base_color_texture();
            let _metallic_factor = pbr.metallic_factor();
            let _metallic_roughness_texture = pbr.metallic_roughness_texture();
            let _roughness_factor = pbr.roughness_factor();
            let _emissive_factor = material.emissive_factor();
            let _emissive_texture = material.emissive_texture();
            let _occlusion_map = material.occlusion_texture();
            let _normal_tex = material.normal_texture();
        }

        Mesh {
            sub_meshes
        }
    }

    fn get_primitive_mode(mode: gltf::mesh::Mode) -> wgpu::PrimitiveTopology {
        match mode {
            gltf::mesh::Mode::Points => {
                wgpu::PrimitiveTopology::PointList
            },
            gltf::mesh::Mode::Lines => {
                wgpu::PrimitiveTopology::LineList
            },
            gltf::mesh::Mode::LineStrip => {
                wgpu::PrimitiveTopology::LineStrip
            },
            gltf::mesh::Mode::Triangles => {
                wgpu::PrimitiveTopology::TriangleList
            },
            gltf::mesh::Mode::TriangleStrip => {
                wgpu::PrimitiveTopology::TriangleStrip
            },
            _ => panic!(format!("Error loading mesht topology isn't supported!")),
        }
    }

}