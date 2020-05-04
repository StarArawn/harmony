# Harmony
A modern 3D/2D game engine that uses wgpu and is designed to work out of the box with minimal effort. It uses legion for handling game/rendering data.

Note: Currently this library is very early in development. Things are very likely going to change. 

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
- [x] Multi-threaded rendering
- [-] Multiple Enviroment probes(almost done).
- [x] Custom render pipelines
- [x] Lighting (Directional, Point)
- [x] PBR shading model

## Future Features (Prioritized)
4. [ ] HDR/Tonemap rendering
5. [ ] Temporal SMAA
6. [ ] SSAO
7. [ ] Shadow Mapping
8. [ ] Asset Bundling and custom assets types.
9. [ ] More useful scene features

## Long Term
- [ ] Investigate using a hybrid rendering pipeline setup similar to halcyon, but without ray tracing(for now..). 
- [ ] Raytracing support?

## Running

To run on metal with validation:

`METAL_DEVICE_WRAPPER_TYPE=1 cargo run --example hello-cube`

To run on vulkan: 

`cargo run --example hello-cube`

Validation should be turned on already you only need to make sure to have the latest vulkan sdk installed. https://vulkan.lunarg.com/

## Examples

- `hello-world` a simple example showcasing the ability to draw text to the screen.
- `hello-cube` a example of how to load gltf files and display them on the screen.
- `benchmark` a benchmark that renders 2500 cubes to test performance.

## shaderc-rs
We use shaderc-rs in harmony to compile GLSL into spir-v. This process works great once we have compiled shaderc-rs unfortunetly shaderc-rs uses shaderc which is written in C++. It tends to compile very slow and require certain things to compile successfully. We have an issue to eventually replace shaderc with something written in pure rust, but currently that crate does not exist. For now anyone attempting to use harmony who encounters issues compiling shaderc should take a look at the documentation found in the readme of shaderc-rs's github page which can be found here:

https://github.com/google/shaderc-rs

If more help is needed or you feel as though the issue you encountered is directly related to shaderc's usage in harmony feel free to open an issue.

## Screenshots
![Hello Cube](/screenshots/screen2.png?raw=true "Hello cube!")

## Acknowledgements:
- Termhn: https://github.com/termhn/rendy-pbr
- Floatingmountain: https://github.com/floatingmountain for helping out a ton!

### Help?
You can find me on the rust game development discord server here:
https://discord.gg/rHxrAPW
I'm more than happy to help out if I am around!
