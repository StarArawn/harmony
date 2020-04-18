# Harmony
A modern 3D/2D game engine that uses wgpu that is designed to work out of the box with minimal effort. It uses specs for handling game/rendering data.

## Features
- [x] A custom asset manager that handles loading data for you.
- [x] Loads GLTF meshes
- [x] (png, jpg, hdr) image support.
- [x] Per-material type forward rendering.
- [x] Unlit materials.
- [x] Scene's managed by specs world.
- [x] Entity Transforms
- [x] Perspective Camera
- [x] Render Graph
- [x] Skybox rendering from an .hdr texture.

## Future Features (Prioritized)
1. [ ] Custom render pipelines
2. [ ] More useful scene features
3. [ ] Frame Profile graph
4. [ ] Lighting (Directional, Point)
5. [ ] PBR shading model
6. [ ] HDR/Tonemap rendering
7. [ ] Temporal SMAA
8. [ ] SSAO
9. [ ] Shadow Mapping
10. [ ] Asset Bundling and custom assets types.

## Long Term
- [ ] Investigate using a hybrid rendering pipeline setup similar to halcyon, but without ray tracing(for now..). 
- [ ] Raytracing support?

To run on metal with validation:

`METAL_DEVICE_WRAPPER_TYPE=1 cargo run --example hello-cube`

To run on vulkan: 

`cargo run --example hello-cube`

Validation should be turned on already you only need to make sure to have the latest vulkan sdk installed. https://vulkan.lunarg.com/
