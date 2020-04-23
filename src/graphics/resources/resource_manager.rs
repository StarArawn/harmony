
use std::collections::HashMap;

use crate::{graphics::render_graph::RenderGraphNode, AssetManager};
use super::{BindGroup, BoundResource};

/// Stores bind groups for consumption by pipelines.
struct ResourceManager {
    // HashMap<Pipeline Name, Bind Group>
    single_bind_groups: HashMap<String, Vec<BindGroup>>,
    multi_bind_groups: HashMap<String, Vec<Vec<BindGroup>>>,
}

impl ResourceManager {
    pub fn new() -> Self {
        Self {
            single_bind_groups: HashMap::new(),
            multi_bind_groups: HashMap::new(),
        }
    }

    pub fn add_single_resource(&mut self, asset_manager: &AssetManager, device: &wgpu::Device, render_node: &RenderGraphNode, resource: &dyn BoundResource) {
        let bind_group = resource.create_bind_group(asset_manager, device, &render_node.pipeline.bind_group_layouts);
        let bind_group_index = bind_group.index as usize;
        if self.single_bind_groups.contains_key(&render_node.name) {
            let bind_groups = self.single_bind_groups.get_mut(&render_node.name).unwrap();
            if bind_group_index > bind_groups.len() {
                panic!("Resource Manager: Invalid bind group index!");
            } else if bind_group_index == bind_groups.len() {
                bind_groups.push(bind_group);
            } else {
                bind_groups[bind_group_index] = bind_group;
            }
        } else {
            self.single_bind_groups.insert(render_node.name.clone(), vec![bind_group]);
        }
    }

    pub fn add_multi_resource(&mut self, asset_manager: &AssetManager, device: &wgpu::Device, render_node: &RenderGraphNode, resources: Vec<&dyn BoundResource>) {
        let mut bind_groups = Vec::new();
        let mut first_binding_index: u32 = 0;
        let mut i = 0;
        for resource in resources {
            let bind_group = resource.create_bind_group(asset_manager, device, &render_node.pipeline.bind_group_layouts);
            if i == 0 {
                first_binding_index = bind_group.index;
            }
            if bind_group.index != first_binding_index {
                panic!("Resource Manager: Invalid bind group index! All bind group indices much match!")
            }
            bind_groups.push(bind_group);
            i += 1;
        }
        if self.single_bind_groups.contains_key(&render_node.name) {
            let per_index_bind_groups = self.multi_bind_groups.get_mut(&render_node.name).unwrap();
            if first_binding_index as usize > per_index_bind_groups.len() {
                panic!("Resource Manager: Invalid bind group index!");
            }
            else if first_binding_index as usize == bind_groups.len() {
                per_index_bind_groups.push(bind_groups);
            } else {
                per_index_bind_groups[first_binding_index as usize] = bind_groups;
            }
        } else {
            self.multi_bind_groups.insert(render_node.name.clone(), vec![bind_groups]);
        }
    }

    pub fn get_bind_group<T: Into<String>>(&self, pipeline_name: T, binding_index: u32) -> &BindGroup {
        let pipeline_name = pipeline_name.into();
        if !self.single_bind_groups.contains_key(&pipeline_name) {
            panic!("Resource Manager: Couldn't find any bind groups!");
        }
        let bind_groups = self.single_bind_groups.get(&pipeline_name).unwrap();
        if binding_index as usize > bind_groups.len() {
            panic!("Resource Manager: Couldn't find any bind groups!");
        }
        &bind_groups[binding_index as usize]
    }

    pub fn get_multi_bind_group<T: Into<String>>(&self, pipeline_name: T, binding_index: u32, item_index: u32) -> &BindGroup {
        let pipeline_name = pipeline_name.into();
        if !self.multi_bind_groups.contains_key(&pipeline_name) {
            panic!("Resource Manager: Couldn't find any bind groups!");
        }

        let item_bind_groups = self.multi_bind_groups.get(&pipeline_name).unwrap();

        let bind_groups: &Vec<BindGroup> = &item_bind_groups[item_index as usize];

        if binding_index as usize > bind_groups.len() {
            panic!("Resource Manager: Couldn't find any bind groups!");
        }

        &bind_groups[binding_index as usize]
    }
}
