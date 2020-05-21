use crate::graphics::material::image::Image;

pub struct AssetManager {
    path: String,
    manager: assetmanage_rs::Manager<Image>,
}

impl AssetManager {
    pub fn new(path: String) -> Self {
        let mut builder = assetmanage_rs::Builder::new();
        let manager = builder.create_manager::<Image>();
        let loader = builder.finish_loader();
        async_std::task::spawn(loader.run());

        Self {
            manager,
            path,
        }
    }

    pub fn insert<T: Into<String>>(&mut self, path: T) -> usize {
        let key = self.manager.insert(std::path::PathBuf::from(format!("{}{}", self.path, path.into())));
        self.manager.load(key).unwrap_or_else(|err| { panic!("Something went wrong! {}", err) });
        key
    }
}

#[test]
fn something_works() {
    env_logger::Builder::from_default_env()
        .init();
        
    let asset_path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/").to_string();
    let mut asset_manager = AssetManager::new(asset_path);

    let asset_handle = asset_manager.insert("/core/white.image.ron");
    asset_manager.manager.maintain();

    let something = asset_manager.manager.get_blocking(asset_handle);
    assert!(something.is_some());
}