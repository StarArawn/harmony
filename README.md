# harmony
A modern 3D/2D game engine that uses wgpu.

To run on metal with validation: 
RUST_BACKTRACE=1 METAL_DEVICE_WRAPPER_TYPE=1 cargo run --example hello-cube

To run on vulkan: 
RUST_BACKTRACE=1 cargo run --example hello-cube