# Asset management
Harmony uses a custom built asset manager that you can easily extend for your own asset loading. You can find more info here:
https://github.com/floatingmountain/assetmanage-rs

Default asset loaders have been implemented for textures, shaders, and meshes. 

All assets are loaded in asynchronously on a separate thread. This means your scene/rendering is not blocked by loading. It is possible to stream in new assets as your player moves around a scene using this method. Assets that haven't loaded yet will not render until they've successfully been loaded.

## Image Assets
Image assets are loading in via a custom RON file. The contents of the file look like:
```rust
ImageInfo(
    file: "white.png", 
    format: SRGB,
)
```

- `file` is a relative path to the actual image information.
- `format` is the format of the image. This is useful for telling harmony that you would like the texture to load in as SRGB or HDR.

List of formats currently supported:

```rust
pub enum ImageFormat {
    SRGB,
    RGB,
    HDR16,
    HDR32,
}
```

### Asset bundles
By default harmony will attempt to load your assets directly from the file system, however bundling/building your assets has a couple of advantages. 

1. Shaders get compiled to SPIR-V ahead of time.
2. Textures generate pre-computed mip-maps instead of doing it during load/runtime.
3. Bundles are just standard zip files.
4. Easier to load into harmony: 
    ```rust
        asset_manager.load_bundle("/my_bundle.zip");
    ```

These features should help improve loading speeds.