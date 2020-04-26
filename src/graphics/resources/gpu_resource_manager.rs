use std::collections::HashMap;

use super::BindGroup;
use crate::graphics::pipelines::{LightingUniform, GlobalUniform};

/// Stores bind groups for consumption by pipelines.
pub struct GPUResourceManager {
    // HashMap<Pipeline Name, Bind Group>
    bind_group_layouts: HashMap<String, wgpu::BindGroupLayout>,
    single_bind_groups: HashMap<String, HashMap<u32, BindGroup>>,
    multi_bind_groups: HashMap<String, HashMap<u32, HashMap<u32, BindGroup>>>,

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
                        // CAMERA TRANSFORM
                        binding: 0,
                        visibility: wgpu::ShaderStage::VERTEX,
                        ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                    },
                    wgpu::BindGroupLayoutEntry {
                        // LIGHTING DATA
                        binding: 1,
                        visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                    },
                ],
                label: None,
            });

        let global_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &global_bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &global_uniform_buffer,
                        range: 0..std::mem::size_of::<GlobalUniform>() as u64,
                    },
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &global_lighting_buffer,
                        range: 0..std::mem::size_of::<LightingUniform>() as u64,
                    },
                },
            ],
            label: None,
        });

        bind_group_layouts.insert("globals".to_string(), global_bind_group_layout);

        Self {
            bind_group_layouts,
            single_bind_groups: HashMap::new(),
            multi_bind_groups: HashMap::new(),
            global_bind_group,
            global_lighting_buffer,
            global_uniform_buffer,
        }
    }

    pub fn add_single_bind_group<T: Into<String>>(&mut self, render_node: T, bind_group: BindGroup) {
        let render_node = render_node.into();
        let bind_group_index = bind_group.index;
        if self.single_bind_groups.contains_key(&render_node) {
            let bind_groups = self.single_bind_groups.get_mut(&render_node).unwrap();
            bind_groups.insert(bind_group_index, bind_group);
        } else {
            let mut hash_map = HashMap::new();
            hash_map.insert(bind_group_index, bind_group);
            self.single_bind_groups
                .insert(render_node.clone(), hash_map);
        }
    }

    pub fn add_multi_bind_group<T: Into<String>>(
        &mut self,
        render_node: T,
        bind_group: BindGroup,
        item_index: u32,
    ) {
        let render_node = render_node.into();
        let bind_group_index = bind_group.index;
        if !self.multi_bind_groups.contains_key(&render_node) {
            let mut bindings_hash_map = HashMap::new();
            let mut hashmap_bind_group = HashMap::new();
            hashmap_bind_group.insert(item_index, bind_group);
            bindings_hash_map.insert(bind_group_index, hashmap_bind_group);
            self.multi_bind_groups
                .insert(render_node, bindings_hash_map);
        } else {
            let bindings_hash_map = self.multi_bind_groups.get_mut(&render_node).unwrap();
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

    pub fn get_multi_bind_group<T: Into<String>>(
        &self,
        pipeline_name: T,
        binding_index: u32,
        item_index: u32,
    ) -> &BindGroup {
        let pipeline_name = pipeline_name.into();
        if !self.multi_bind_groups.contains_key(&pipeline_name) {
            panic!("Resource Manager: Couldn't find any bind groups!");
        }
        let multi_bind_groups = self.multi_bind_groups.get(&pipeline_name);
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

    pub fn get_bind_group<T: Into<String>>(
        &self,
        pipeline_name: T,
        binding_index: u32,
    ) -> &BindGroup {
        let pipeline_name = pipeline_name.into();
        if !self.single_bind_groups.contains_key(&pipeline_name) {
            panic!("Resource Manager: Couldn't find any bind groups!");
        }
        let bind_groups = self.single_bind_groups.get(&pipeline_name).unwrap();
        let bind_group = bind_groups.get(&binding_index);
        if bind_group.is_none() {
            panic!("Resource Manager: Couldn't find any bind groups!");
        }

        bind_group.as_ref().unwrap()
    }

    pub fn set_bind_group<'a, T: Into<String>>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        pipeline_name: T,
        binding_index: u32,
    ) {
        let bind_group = self.get_bind_group(pipeline_name, binding_index);
        render_pass.set_bind_group(bind_group.index, &bind_group.group, &[]);
    }

    pub fn set_multi_bind_group<'a, T: Into<String>>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        pipeline_name: T,
        binding_index: u32,
        item_index: u32,
    ) {
        let bind_group = self.get_multi_bind_group(pipeline_name, binding_index, item_index);
        render_pass.set_bind_group(bind_group.index, &bind_group.group, &[]);
    }

    /// Let's you add bind group layourts.
    pub fn add_bind_group_layout<T: Into<String>>(&mut self, name: T, bind_group_layout: wgpu::BindGroupLayout) {
        let name = name.into();
        if self.bind_group_layouts.contains_key(&name) {
            panic!("Bind group layout already exists use `get_bind_group_layout` or a different key.");
        }
        self.bind_group_layouts.insert(name, bind_group_layout);
    }

    pub fn get_bind_group_layout<T: Into<String>>(&self, name: T) -> &wgpu::BindGroupLayout {
        self.bind_group_layouts.get(&name.into()).unwrap()
    }
}
