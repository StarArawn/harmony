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

    pub fn insert<T: Into<PathBuf>>(&mut self, abs_path: T){
        self.ron_manager.insert(abs_path.into(), ());
    }
    pub fn load<T: Into<PathBuf>>(&mut self, abs_path: T) -> Result<(), std::io::Error> {
        self.ron_manager.load(abs_path.into(), ())
    }
    pub fn get<T: Into<PathBuf>>(&mut self, abs_path: T) -> Option<Arc<NewMaterial>> {
        let abs_path = abs_path.into();
        self.material_cache.get(&abs_path).cloned()
        //if Ron not inserted return None
        //if Ron not loaded return None
        //if Ron loading return None
        //if Ron loaded dont construct here. Will be constructed on next call to maintain. return None
    }
        
    
    pub fn maintain(&mut self){

        for mat_ron in self.ron_manager.get_loaded_once(){
            let ron = self.ron_manager.get(mat_ron).unwrap();
            //TODO: load images

            self.try_construct(mat_ron)
        }
    }
    fn try_construct(&self, mat_ron: Arc<MaterialRon>) -> Option<Arc<NewMaterial>>{
        //construct mat and add to materialcache
            match mat_ron.as_ref(){
                MaterialRon::PBRMaterial { 
                    main_texture, 
                    roughness_texture, 
                    normal_texture, 
                    roughness, 
                    metallic, 
                    color 
                } => {}
            }
            todo!()
            
        
    }
}