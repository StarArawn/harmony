use std::{io, fs};
use crate::Application;

pub struct HDRImage {
    pub name: String,
    pub(crate) texture: wgpu::Texture,
    pub(crate) sampler: wgpu::Sampler,
    pub(crate) view: wgpu::TextureView,
    pub(crate) cubemap_texture: Option<wgpu::Texture>,
    pub(crate) cubemap_view: Option<wgpu::TextureView>,
    pub(crate) cubemap_sampler: wgpu::Sampler,
    pub(crate) cubemap_bind_group: Option<wgpu::BindGroup>,
}

unsafe fn any_as_u8_slice<T: Sized>(any: &T) -> &[u8] {
    let ptr = (any as *const T) as *const u8;
    std::slice::from_raw_parts(ptr, std::mem::size_of::<T>())
}

impl HDRImage {
    pub fn new<T>(device: &wgpu::Device, init_encoder: &mut wgpu::CommandEncoder, path: T, file_name: T) -> Self where T: Into<String> {
        let path = path.into();
        let file_name = file_name.into();
        
        // Load the image
        let decoder = image::hdr::HdrDecoder::new(io::BufReader::new(
            fs::File::open(&path).unwrap(),
        )).unwrap();
        let metadata = decoder.metadata();
        let decoded = decoder.read_image_hdr().unwrap();
        
        let (w, h) = (metadata.width, metadata.height);

        let texture_extent = wgpu::Extent3d {
            width: w,
            height: h,
            depth: 1,
        };
        
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_extent,
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
            label: None,
        });

        let image_data = decoded
            .iter()
            .flat_map(|pixel| vec![pixel[0], pixel[1], pixel[2], 1.0])
            .collect::<Vec<_>>();
        dbg!(image_data.len());
        dbg!((w * h) * (4 * 4));
        let image_bytes =
            unsafe { std::slice::from_raw_parts(image_data.as_ptr() as *const u8, image_data.len() * 4) }
                .to_vec();
        let temp_buf = device.create_buffer_with_data(&image_bytes, wgpu::BufferUsage::COPY_SRC);

        init_encoder.copy_buffer_to_texture(
            wgpu::BufferCopyView {
                buffer: &temp_buf,
                offset: 0,
                bytes_per_row: (4 * 4) * w,
                rows_per_image: 0,
            },
            wgpu::TextureCopyView {
                texture: &texture,
                mip_level: 0,
                array_layer: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            texture_extent,
        );

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: wgpu::CompareFunction::Undefined,
        });

        let view = texture.create_default_view();


        let cubemap_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: wgpu::CompareFunction::Undefined,
        });

        Self {
            name: file_name.clone(),
            texture,
            sampler,
            view,
            cubemap_texture: None,
            cubemap_view: None,
            cubemap_sampler,
            cubemap_bind_group: None,
        }
    }

    pub(crate) fn create_cube_map(app: &mut Application) {
        use crate::graphics::{SimplePipeline, SimplePipelineDesc};
        
        // We need to convert our regular texture map to a cube texture map with 6 faces.
        // Should be straight forward enough if we use equirectangular projection.
        // First we need a custom pipeline that will run in here to do the conversion.
        let mut cube_projection_pipeline_desc = crate::graphics::pipelines::equirectangular::CubeProjectionPipelineDesc::default();
        let pipeline = cube_projection_pipeline_desc.pipeline(app);

        let mut final_pipeline = cube_projection_pipeline_desc.build(&app.renderer.device, &pipeline.bind_group_layouts);

        let command_buffer = final_pipeline.render(None, &app.renderer.device, &pipeline, Some(&mut app.asset_manager), None, None);
        
        app.renderer.queue.submit(&[command_buffer]);
    }
}