use assetmanage_rs::*;
use std::{error::Error, path::PathBuf, sync::Arc, collections::HashMap};
use super::ImageManager;
use crate::graphics::material::{NewMaterial, MaterialRon};
pub(crate) struct MaterialManager {
    base_path: PathBuf,
    
    ron_manager: Manager<MaterialRon,MemoryLoader>,
    image_manager: ImageManager,
    material_cache: HashMap<PathBuf,Arc<NewMaterial>>,
}
impl MaterialManager{
    pub(crate) fn new<T: Into<PathBuf>>(base_path: T, image_manager: ImageManager) -> Self {
        let base_path = base_path.into();
        let mut builder = assetmanage_rs::Builder::new();
        let ron_manager = builder.create_manager::<MaterialRon>(());
        let loader = builder.finish_loader(());
        async_std::task::spawn(loader.run());
        Self {
            base_path,
            ron_manager,
            image_manager,
            material_cache: HashMap::new(),
        }
    }

    pub fn insert<T: Into<PathBuf>>(&mut self, abs_path: T) -> Result<(), Box<dyn Error>> {
        self.ron_manager.insert(abs_path.into(), ());
        Ok(())
    }
    pub fn get<T: Into<PathBuf>>(&mut self, abs_path: T) -> Option<Arc<NewMaterial>> {
        let abs_path = abs_path.into();
        match self.material_cache.get(&abs_path){
            Some(m) => Some(m.clone()),
            None => {
                if let Some(mat_ron) = self.ron_manager.get(&abs_path){
                    Some(self.try_construct_mat(mat_ron))
                } else {
                    match self.ron_manager.status(&abs_path){
                        Some(s) => match s{
                            LoadStatus::NotLoaded => {
                                if self.ron_manager.load(&abs_path,()).is_err(){};
                                None
                            }
                            LoadStatus::Loading => {None}
                            LoadStatus::Loaded => {None}
                        }
                        None => {None}
                    }
                    todo!()
                }
            }
        }
    }
    pub fn status<T: Into<PathBuf>>(&self, abs_path: T) -> Option<assetmanage_rs::LoadStatus> {
        self.image_manager.status(&abs_path.into())
    }
    fn try_construct_mat(&self, mat: Arc<MaterialRon>) -> Arc<NewMaterial>{
        //construct mat and add to materialcache
        todo!()
    }
}