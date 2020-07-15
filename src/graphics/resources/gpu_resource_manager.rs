use std::sync::Arc;

use super::{ArcRenderPass, BindGroup};
use crate::{
    graphics::pipelines::{GlobalUniform, LightingUniform},
    scene::components::transform::LocalUniform,
};
use dashmap::DashMap;

/// Stores bind groups for consumption by pipelines.
/// Also can store buffers, but it's not required.
pub struct GPUResourceManager {
    // HashMap<Pipeline Name, Bind Group>
    bind_group_layouts: DashMap<String, Arc<wgpu::BindGroupLayout>>,
    single_bind_groups: DashMap<String, DashMap<u32, Arc<BindGroup>>>,
    multi_bind_groups: DashMap<String, DashMap<u32, DashMap<u32, Arc<BindGroup>>>>,
    multi_buffer: DashMap<String, DashMap<u32, Arc<wgpu::Buffer>>>,
    buffers: DashMap<String, Arc<wgpu::Buffer>>,

    pub global_uniform_buffer: wgpu::Buffer,
    pub global_lighting_buffer: wgpu::Buffer,
    pub global_bind_group: wgpu::BindGroup,
}

impl GPUResourceManager {
    pub fn new(device: Arc<wgpu::Device>) -> Self {
        let bind_group_layouts = DashMap::new();

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
                    wgpu::BindGroupLayoutEntry::new(
                        // CAMERA INFO
                        0,
                        wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                        wgpu::BindingType::UniformBuffer {
                            dynamic: false,
                            min_binding_size: wgpu::BufferSize::new(
                                std::mem::size_of::<GlobalUniform>() as _,
                            ),
                        },
                    ),
                    wgpu::BindGroupLayoutEntry::new(
                        // LIGHTING DATA
                        1,
                        wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                        wgpu::BindingType::UniformBuffer {
                            dynamic: false,
                            min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<
                                LightingUniform,
                            >()
                                as _),
                        },
                    ),
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

        bind_group_layouts.insert("globals".to_string(), Arc::new(global_bind_group_layout));

        // Local bind group layout
        let local_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                bindings: &[wgpu::BindGroupLayoutEntry::new(
                    0,
                    wgpu::ShaderStage::VERTEX,
                    wgpu::BindingType::UniformBuffer {
                        dynamic: false,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<LocalUniform>() as _,
                        ),
                    },
                )],
                label: Some("Locals"),
            });
        bind_group_layouts.insert("locals".to_string(), Arc::new(local_bind_group_layout));

        Self {
            bind_group_layouts,
            buffers: DashMap::new(),
            single_bind_groups: DashMap::new(),
            multi_bind_groups: DashMap::new(),
            multi_buffer: DashMap::new(),
            global_bind_group,
            global_lighting_buffer,
            global_uniform_buffer,
        }
    }

    /// Adds a single bind group with a given key.
    pub fn add_single_bind_group<T: Into<String>>(&self, key: T, bind_group: BindGroup) {
        let key = key.into();
        let bind_group_index = bind_group.index;
        if self.single_bind_groups.contains_key(&key) {
            let bind_groups = self.single_bind_groups.get_mut(&key).unwrap();
            bind_groups.insert(bind_group_index, Arc::new(bind_group));
        } else {
            let hash_map = DashMap::new();
            hash_map.insert(bind_group_index, Arc::new(bind_group));
            self.single_bind_groups.insert(key.clone(), hash_map);
        }
    }

    /// Adds a multi bind group with a given key and an index.
    /// Useful for transformation bind groups.
    /// Storage looks like: HashMap<key, HashMap<index, BindGroup>>
    pub fn add_multi_bind_group<T: Into<String>>(
        &self,
        key: T,
        bind_group: BindGroup,
        item_index: u32,
    ) {
        let key = key.into();
        let bind_group_index = bind_group.index;
        if !self.multi_bind_groups.contains_key(&key) {
            let bindings_hash_map = DashMap::new();
            let hashmap_bind_group = DashMap::new();
            hashmap_bind_group.insert(item_index, Arc::new(bind_group));
            bindings_hash_map.insert(bind_group_index, hashmap_bind_group);
            self.multi_bind_groups.insert(key, bindings_hash_map);
        } else {
            let bindings_hash_map = self.multi_bind_groups.get_mut(&key).unwrap();
            let mut hashmap_bind_group = bindings_hash_map.get_mut(&bind_group_index);
            if hashmap_bind_group.is_some() {
                let hashmap_bind_group = hashmap_bind_group.as_mut().unwrap();
                hashmap_bind_group.insert(item_index, Arc::new(bind_group));
            } else {
                let hashmap_bind_group = DashMap::new();
                hashmap_bind_group.insert(item_index, Arc::new(bind_group));
                bindings_hash_map.insert(bind_group_index, hashmap_bind_group);
            }
        }
    }

    /// Same as the multi bind group but for buffers instead.
    pub fn add_multi_buffer<T: Into<String>>(&self, key: T, buffer: wgpu::Buffer, item_index: u32) {
        let key = key.into();
        if self.multi_buffer.contains_key(&key) {
            let item_hash_map = self.multi_buffer.get_mut(&key).unwrap();
            item_hash_map.insert(item_index, Arc::new(buffer));
        } else {
            let hash_map = DashMap::new();
            hash_map.insert(item_index, Arc::new(buffer));
            self.multi_buffer.insert(key, hash_map);
        }
    }

    /// Let's you retrieve a multi-buffer.
    pub fn get_multi_buffer<T: Into<String>>(&self, key: T, item_index: u32) -> Arc<wgpu::Buffer> {
        self.multi_buffer
            .get(&key.into())
            .unwrap()
            .get(&item_index)
            .unwrap()
            .clone()
    }

    /// Let's you retrieve a multi-bind group.
    /// binding_index is associated with an index set inside of the BindGroup.
    pub fn get_multi_bind_group<T: Into<String>>(
        &self,
        key: T,
        binding_index: u32,
        item_index: u32,
    ) -> Arc<BindGroup> {
        let key = key.into();
        if !self.multi_bind_groups.contains_key(&key) {
            panic!("Resource Manager: Couldn't find any bind groups!");
        }
        let multi_bind_groups = self.multi_bind_groups.get(&key);
        let multi_bind_groups = multi_bind_groups.unwrap();
        let bind_groups = multi_bind_groups.get(&binding_index);
        if bind_groups.is_none() {
            panic!("Resource Manager: Couldn't find any bind groups!");
        }
        let bind_groups = bind_groups.unwrap();
        let bind_group = bind_groups.get(&item_index);
        if bind_group.is_none() {
            panic!("Resource Manager: Couldn't find any bind groups!");
        }

        let bind_group = bind_group.as_ref().unwrap();
        bind_group.value().clone()
    }

    /// Get's a bind group.
    /// binding_index is associated with an index set inside of the BindGroup.
    pub fn get_bind_group<T: Into<String>>(
        &self,
        key: T,
        binding_index: u32,
    ) -> Option<Arc<BindGroup>> {
        let key = key.into();
        if !self.single_bind_groups.contains_key(&key) {
            panic!("Resource Manager: Couldn't find any bind groups!");
        }
        let bind_groups = self.single_bind_groups.get(&key).unwrap();
        let bind_group = bind_groups.get(&binding_index);
        if bind_group.is_none() {
            panic!("Resource Manager: Couldn't find any bind groups!");
        }

        match bind_group {
            Some(bind_group) => Some(bind_group.clone()),
            None => None,
        }
    }

    /// Sets a multi-bind group.
    pub fn set_multi_bind_group<'a, T: Into<String>>(
        &'a self,
        render_pass: &mut ArcRenderPass<'a>,
        key: T,
        binding_index: u32,
        item_index: u32,
    ) {
        let bind_group: Arc<BindGroup> = self.get_multi_bind_group(key, binding_index, item_index);
        render_pass.set_bind_group_internal(bind_group);
    }

    /// Let's you add bind group layouts.
    pub fn add_bind_group_layout<T: Into<String>>(
        &self,
        name: T,
        bind_group_layout: wgpu::BindGroupLayout,
    ) {
        let name = name.into();
        if self.bind_group_layouts.contains_key(&name) {
            panic!(
                "Bind group layout already exists use `get_bind_group_layout` or a different key."
            );
        }
        self.bind_group_layouts
            .insert(name, Arc::new(bind_group_layout));
    }

    /// Gets a bind group layout based on name.
    pub fn get_bind_group_layout<T: Into<String>>(
        &self,
        name: T,
    ) -> Option<Arc<wgpu::BindGroupLayout>> {
        match self.bind_group_layouts.get(&name.into()) {
            Some(layout) => Some(layout.value().clone()),
            None => None,
        }
    }

    /// Add a single buffer.
    pub fn add_buffer<T: Into<String>>(&self, name: T, buffer: wgpu::Buffer) {
        let name = name.into();
        if self.bind_group_layouts.contains_key(&name) {
            panic!("Buffer already exists use `get_buffer` or use a different key.");
        }
        self.buffers.insert(name, Arc::new(buffer));
    }

    /// Gets a single buffer.
    pub fn get_buffer<T: Into<String>>(&self, name: T) -> Arc<wgpu::Buffer> {
        self.buffers.get(&name.into()).unwrap().value().clone()
    }
}
