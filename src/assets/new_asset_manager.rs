use crate::graphics::material::image::{ImageInfo, ImageBuilder};

// pub struct AssetManager {
//     path: String,
//     manager: assetmanage_rs::Manager<Image>,
// }

// impl AssetManager {
//     pub fn new(path: String) -> Self {
//         let mut builder = assetmanage_rs::Builder::new();
//         let manager = builder.create_manager::<Image>();
//         let loader = builder.finish_loader();
//         async_std::task::spawn(loader.run());

//         Self {
//             manager,
//             path,
//         }
//     }

//     pub fn insert<T: Into<String>>(&mut self, path: T) -> usize {
//         let key = self.manager.insert(std::path::PathBuf::from(format!("{}{}", self.path, path.into())));
//         self.manager.load(key).unwrap_or_else(|err| { panic!("Something went wrong! {}", err) });
//         key
//     }
// }

#[test]
fn test_image() {
    env_logger::Builder::from_default_env()
        .init();

    let asset_path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/").to_string();
    let mut builder = assetmanage_rs::Builder::new();
    let mut image_info_manager = builder.create_manager::<ImageInfo>();
    let mut image_manager = builder.create_manager::<ImageBuilder>();
    let loader = builder.finish_loader();
    async_std::task::spawn(loader.run());

    let key = image_info_manager.insert(std::path::PathBuf::from(format!("{}{}", asset_path, "core/white.image.ron")));
    image_info_manager.load(key).unwrap();

    let image_info = image_info_manager.get_blocking(key);
    dbg!(&image_info);
    assert!(image_info.is_some());

    let image_info = image_info.unwrap();

    let image_path = format!("{}/core/{}", asset_path, image_info.file);

    let key = image_manager.insert(std::path::PathBuf::from(image_path));
    image_manager.load(key).unwrap();
    
    let image_builder = image_manager.get_blocking(key);
    
    assert!(image_builder.is_some());

    let image_builder = image_builder.unwrap();

    //let image = image_builder.build(image_info.as_ref());

    // let mut image_manager = resources.get_mut::<ImageManager>();
    // image_manager.add(image);
}