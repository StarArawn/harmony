use std::collections::HashMap;

use super::BindGroup;
use crate::graphics::pipelines::{GlobalUniform, LightingUniform};

/// Stores bind groups for consumption by pipelines.
/// Also can store buffers, but it's not required.
pub struct GPUResourceManager {
    // HashMap<Pipeline Name, Bind Group>
    bind_group_layouts: HashMap<String, wgpu::BindGroupLayout>,
    single_bind_groups: HashMap<String, HashMap<u32, BindGroup>>,
    multi_bind_groups: HashMap<String, HashMap<u32, HashMap<u32, BindGroup>>>,
    multi_buffer: HashMap<String, HashMap<u32, wgpu::Buffer>>,

    buffers: HashMap<String, wgpu::Buffer>,

    pub global_uniform_buffer: wgpu::Buffer,
    pub global_lighting_buffer: wgpu::Buffer,
    pub global_bind_group: wgpu::BindGroup,
}

impl GPUResourceManager {
    pub fn new(device: &wgpu::Device) -> Self {
        let mut bind_group_layouts = HashMap::new();

        // Create our global uniforms buffers, layouts, and bindgroups here.
        // These *can* be shared across all pipelines.

        let global_uniform_buffer = device.create_buffer_with_data(
            bytemuck::bytes_of(&GlobalUniform::default()),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );

        let global_lighting_buffer = device.create_buffer_with_data(
            bytemuck::bytes_of(&LightingUniform::default()),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );

        let global_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                bindings: &[
                    wgpu::BindGroupLayoutEntry {
                        // CAMERA INFO
                        binding: 0,
                        visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                    },
                    wgpu::BindGroupLayoutEntry {
                        // LIGHTING DATA
                        binding: 1,
                        visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                    },
                ],
                label: Some("Globals"),
            });

        let global_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &global_bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(global_uniform_buffer.slice(..)),
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(global_lighting_buffer.slice(..)),
                },
            ],
            label: Some("Globals"),
        });

        bind_group_layouts.insert("globals".to_string(), global_bind_group_layout);

        // Local bind group layout
        let local_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                bindings: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                }],
                label: Some("Locals"),
            });
        bind_group_layouts.insert("locals".to_string(), local_bind_group_layout);

        Self {
            bind_group_layouts,
            buffers: HashMap::new(),
            single_bind_groups: HashMap::new(),
            multi_bind_groups: HashMap::new(),
            multi_buffer: HashMap::new(),
            global_bind_group,
            global_lighting_buffer,
            global_uniform_buffer,
        }
    }

    /// Adds a single bind group with a given key.
    pub fn add_single_bind_group<T: Into<String>>(&mut self, key: T, bind_group: BindGroup) {
        let key = key.into();
        let bind_group_index = bind_group.index;
        if self.single_bind_groups.contains_key(&key) {
            let bind_groups = self.single_bind_groups.get_mut(&key).unwrap();
            bind_groups.insert(bind_group_index, bind_group);
        } else {
            let mut hash_map = HashMap::new();
            hash_map.insert(bind_group_index, bind_group);
            self.single_bind_groups.insert(key.clone(), hash_map);
        }
    }

    /// Adds a multi bind group with a given key and an index.
    /// Useful for transformation bind groups.
    /// Storage looks like: HashMap<key, HashMap<index, BindGroup>>
    pub fn add_multi_bind_group<T: Into<String>>(
        &mut self,
        key: T,
        bind_group: BindGroup,
        item_index: u32,
    ) {
        let key = key.into();
        let bind_group_index = bind_group.index;
        if !self.multi_bind_groups.contains_key(&key) {
            let mut bindings_hash_map = HashMap::new();
            let mut hashmap_bind_group = HashMap::new();
            hashmap_bind_group.insert(item_index, bind_group);
            bindings_hash_map.insert(bind_group_index, hashmap_bind_group);
            self.multi_bind_groups.insert(key, bindings_hash_map);
        } else {
            let bindings_hash_map = self.multi_bind_groups.get_mut(&key).unwrap();
            let mut hashmap_bind_group = bindings_hash_map.get_mut(&bind_group_index);
            if hashmap_bind_group.is_some() {
                let hashmap_bind_group = hashmap_bind_group.as_mut().unwrap();
                hashmap_bind_group.insert(item_index, bind_group);
            } else {
                let mut hashmap_bind_group = HashMap::new();
                hashmap_bind_group.insert(item_index, bind_group);
                bindings_hash_map.insert(bind_group_index, hashmap_bind_group);
            }
        }
    }

    /// Same as the multi bind group but for buffers instead.
    pub fn add_multi_buffer<T: Into<String>>(
        &mut self,
        key: T,
        buffer: wgpu::Buffer,
        item_index: u32,
    ) {
        let key = key.into();
        if self.multi_buffer.contains_key(&key) {
            let item_hash_map = self.multi_buffer.get_mut(&key).unwrap();
            item_hash_map.insert(item_index, buffer);
        } else {
            let mut hash_map = HashMap::new();
            hash_map.insert(item_index, buffer);
            self.multi_buffer.insert(key, hash_map);
        }
    }

    /// Let's you retrieve a multi-buffer.
    pub fn get_multi_buffer<T: Into<String>>(&self, key: T, item_index: u32) -> &wgpu::Buffer {
        self.multi_buffer
            .get(&key.into())
            .unwrap()
            .get(&item_index)
            .unwrap()
    }

    /// Let's you retrieve a multi-bind group.
    /// binding_index is associated with an index set inside of the BindGroup.
    pub fn get_multi_bind_group<T: Into<String>>(
        &self,
        key: T,
        binding_index: u32,
        item_index: u32,
    ) -> &BindGroup {
        let key = key.into();
        if !self.multi_bind_groups.contains_key(&key) {
            panic!("Resource Manager: Couldn't find any bind groups!");
        }
        let multi_bind_groups = self.multi_bind_groups.get(&key);
        let bind_groups = multi_bind_groups.unwrap().get(&binding_index);
        if bind_groups.is_none() {
            panic!("Resource Manager: Couldn't find any bind groups!");
        }
        let bind_group = bind_groups.unwrap().get(&item_index);
        if bind_group.is_none() {
            panic!("Resource Manager: Couldn't find any bind groups!");
        }

        bind_group.as_ref().unwrap()
    }

    /// Get's a bind group.
    /// binding_index is associated with an index set inside of the BindGroup.
    pub fn get_bind_group<T: Into<String>>(
        &self,
        key: T,
        binding_index: u32,
    ) -> Option<&BindGroup> {
        let key = key.into();
        if !self.single_bind_groups.contains_key(&key) {
            panic!("Resource Manager: Couldn't find any bind groups!");
        }
        let bind_groups = self.single_bind_groups.get(&key).unwrap();
        let bind_group = bind_groups.get(&binding_index);
        if bind_group.is_none() {
            panic!("Resource Manager: Couldn't find any bind groups!");
        }

        bind_group
    }

    /// Sets a bind group.
    pub fn set_bind_group<'a, T: Into<String>>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        key: T,
        binding_index: u32,
    ) {
        let bind_group = self.get_bind_group(key, binding_index).unwrap();
        render_pass.set_bind_group(bind_group.index, &bind_group.group, &[]);
    }

    /// Sets a multi-bind group.
    pub fn set_multi_bind_group<'a, T: Into<String>>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        key: T,
        binding_index: u32,
        item_index: u32,
    ) {
        let bind_group = self.get_multi_bind_group(key, binding_index, item_index);
        render_pass.set_bind_group(bind_group.index, &bind_group.group, &[]);
    }

    /// Let's you add bind group layouts.
    pub fn add_bind_group_layout<T: Into<String>>(
        &mut self,
        name: T,
        bind_group_layout: wgpu::BindGroupLayout,
    ) {
        let name = name.into();
        if self.bind_group_layouts.contains_key(&name) {
            panic!(
                "Bind group layout already exists use `get_bind_group_layout` or a different key."
            );
        }
        self.bind_group_layouts.insert(name, bind_group_layout);
    }

    /// Gets a bind group layout based on name.
    pub fn get_bind_group_layout<T: Into<String>>(
        &self,
        name: T,
    ) -> Option<&wgpu::BindGroupLayout> {
        self.bind_group_layouts.get(&name.into())
    }

    /// Add a single buffer.
    pub fn add_buffer<T: Into<String>>(&mut self, name: T, buffer: wgpu::Buffer) {
        let name = name.into();
        if self.bind_group_layouts.contains_key(&name) {
            panic!("Buffer already exists use `get_buffer` or use a different key.");
        }
        self.buffers.insert(name, buffer);
    }

    /// Gets a single buffer.
    pub fn get_buffer<T: Into<String>>(&self, name: T) -> &wgpu::Buffer {
        self.buffers.get(&name.into()).unwrap()
    }
}
