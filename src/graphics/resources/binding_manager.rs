
use std::{collections::HashMap};

use super::{BindGroup};

/// Stores bind groups for consumption by pipelines.
#[derive(Default)]
pub struct BindingManager {
    // HashMap<Pipeline Name, Bind Group>
    single_bind_groups: HashMap<String, HashMap<u32, BindGroup>>,
    multi_bind_groups: HashMap<String, HashMap<u32, HashMap<u32, BindGroup>>>,
}

impl BindingManager {
    pub fn new() -> Self {
        Self {
            single_bind_groups: HashMap::new(),
            multi_bind_groups: HashMap::new(),
        }
    }

    pub fn add_single_resource<T: Into<String>>(&mut self, render_node: T, bind_group: BindGroup) {
        let render_node = render_node.into();
        let bind_group_index = bind_group.index;
        if self.single_bind_groups.contains_key(&render_node) {
            let bind_groups = self.single_bind_groups.get_mut(&render_node).unwrap();
            bind_groups.insert(bind_group_index, bind_group);
        } else {
            let mut hash_map = HashMap::new();
            hash_map.insert(bind_group_index, bind_group);
            self.single_bind_groups.insert(render_node.clone(), hash_map);
        }
    }

    pub fn add_multi_resource<T: Into<String>>(&mut self, render_node: T, bind_group: BindGroup, item_index: u32) {
        let render_node = render_node.into();
        let bind_group_index = bind_group.index;
        if !self.multi_bind_groups.contains_key(&render_node) {
            let mut bindings_hash_map = HashMap::new();
            let mut hashmap_bind_group = HashMap::new();
            hashmap_bind_group.insert(item_index, bind_group);
            bindings_hash_map.insert(bind_group_index, hashmap_bind_group);
            self.multi_bind_groups.insert(render_node, bindings_hash_map);
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

    pub fn get_multi_bind_group<T: Into<String>>(&self, pipeline_name: T, binding_index: u32, item_index: u32) -> &BindGroup {
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

    pub fn get_bind_group<T: Into<String>>(&self, pipeline_name: T, binding_index: u32) -> &BindGroup {
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

    pub fn set_bind_group<'a, T: Into<String>>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, pipeline_name: T, binding_index: u32) {
        let bind_group = self.get_bind_group(pipeline_name, binding_index);
        render_pass.set_bind_group(bind_group.index, &bind_group.group, &[]);
    }

    pub fn set_multi_bind_group<'a, T: Into<String>>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, pipeline_name: T, binding_index: u32, item_index: u32) {
        let bind_group = self.get_multi_bind_group(pipeline_name, binding_index, item_index);
        render_pass.set_bind_group(bind_group.index, &bind_group.group, &[]);
    }

    // pub fn get_multi_bind_group<T: Into<String>>(&self, pipeline_name: T, binding_index: u32, item_index: u32) -> &BindGroup {
    //     let pipeline_name = pipeline_name.into();
    //     if !self.multi_bind_groups.contains_key(&pipeline_name) {
    //         panic!("Resource Manager: Couldn't find any bind groups!");
    //     }

    //     let item_bind_groups = self.multi_bind_groups.get(&pipeline_name).unwrap();

    //     let bind_groups: &Vec<BindGroup> = &item_bind_groups[item_index as usize];

    //     if binding_index as usize > bind_groups.len() {
    //         panic!("Resource Manager: Couldn't find any bind groups!");
    //     }

    //     &bind_groups[binding_index as usize]
    // }
}
