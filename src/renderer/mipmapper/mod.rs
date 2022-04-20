use std::{cell::RefCell, convert::TryInto, mem::size_of, num::NonZeroU32, rc::Rc, sync::Arc};

use crevice::std430::{AsStd430, Std430};
use futures::lock::Mutex;
use glam::IVec2;
use itertools::Itertools;
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;

use crate::{
    game::{
        entity::components::model::{ModelVertex, Vertex},
        entity::components::Model,
        World,
    },
    renderer::{glsl_loader, RenderContext},
};

use self::uniforms::Uniforms;

use super::{texture_atlas::TextureAtlas, ShaderBundle};

mod uniforms;

pub struct Mipmapper {
    compute_pipeline: wgpu::ComputePipeline,
    compute_bind_group_layout: wgpu::BindGroupLayout,
    world: Arc<Mutex<World>>,
    mip_levels: u32,
    atlas: Rc<RefCell<TextureAtlas>>,
    views: Vec<wgpu::TextureView>,
}

impl Mipmapper {
    pub async fn new(
        context: &RenderContext,
        world: Arc<Mutex<World>>,
        atlas: Rc<RefCell<TextureAtlas>>,
    ) -> Self {
        let (width, height, length) = (512, 512, 512);

        let compute_bind_group_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D3,
                                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            ty: wgpu::BindingType::StorageTexture {
                                access: wgpu::StorageTextureAccess::WriteOnly,
                                format: wgpu::TextureFormat::Rgba32Float,
                                view_dimension: wgpu::TextureViewDimension::D3,
                            },
                            binding: 1,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None, // TODO optimize
                            },
                            binding: 2,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            count: None,
                        },
                    ],
                    label: Some("Mipmap Compute Shader Bind Group layout decriptor"),
                });

        let mip_levels =
            ((((width).min((height).min(length)) as f32).log2()).floor() - 0.0).max(0.0) as u32 - 1;

        let compute_pipeline_layout =
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Mipmap Compute Shader Pipeline Layout Descriptor"),
                    bind_group_layouts: &[&compute_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let compute_shader;

        unsafe {
            compute_shader = ShaderBundle::compute_from_path("mipmap")
                .create_compute_shader_module_spirv(context);
        }

        let compute_pipeline =
            context
                .device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("Compute pipeline"),
                    layout: Some(&compute_pipeline_layout),
                    module: &compute_shader,
                    entry_point: "main",
                });

        let atlas_ref = atlas.borrow_mut();

        let views = (0..mip_levels)
            .map(|mip| {
                atlas_ref
                    .get_view_descriptor(
                        "voxelizer_attachment_world",
                        {
                            &wgpu::TextureViewDescriptor {
                                label: Some("mip"),
                                format: None,
                                dimension: None,
                                aspect: wgpu::TextureAspect::All,
                                base_mip_level: mip,
                                mip_level_count: NonZeroU32::new(1),
                                base_array_layer: 0,
                                array_layer_count: None,
                            }
                        },
                        context,
                    )
                    .unwrap()
            })
            .collect::<Vec<_>>();

        Self {
            compute_pipeline,
            world,
            mip_levels,
            atlas: atlas.clone(),
            compute_bind_group_layout,
            views,
        }
    }

    pub async fn render(&mut self, context: &RenderContext) {
        let mut encoder = context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let atlas_lock = self.atlas.borrow();

        let index_buffer_slice = atlas_lock
            .get_buffer("voxelizer_binding_voxels_changed_index", context)
            .unwrap()
            .slice(..);

        // let index_buffer_future = index_buffer_slice.map_async(wgpu::MapMode::Read);

        // context.device.poll(wgpu::Maintain::Wait);

        // index_buffer_future.await.unwrap();

        // let index: i32;
        
        // {
        //     let index_data = index_buffer_slice.get_mapped_range();

        //     index = *bytemuck::cast_slice(&index_data).get(0).unwrap();
        // }

        // println!("{}", index);

        let invoke_positions_buffer_slice = atlas_lock
            .get_buffer("voxelizer_binding_voxels_changed", context)
            .unwrap()
            // .slice(0..(index as u64 * 32)); // Add buffers to texture atlas?
            .slice(..);

        let invoke_positions_buffer_future =
            invoke_positions_buffer_slice.map_async(wgpu::MapMode::Read);

        context.device.poll(wgpu::Maintain::Wait);

        invoke_positions_buffer_future.await.unwrap();

        let mut invoke_positions: Vec<_>;

        {
            let data = invoke_positions_buffer_slice.get_mapped_range();

            invoke_positions = bytemuck::cast_slice(&data)
                .to_vec()
                .into_iter()
                .map(|x: [i32; 4]| [x[0], x[1], x[2], 0])
                .collect();
        }

        atlas_lock
            .get_buffer("voxelizer_binding_voxels_changed", context)
            .unwrap()
            .unmap();

        for level in 1..self.mip_levels as usize {
            invoke_positions = invoke_positions
                .into_iter()
                .map(|x| [x[0] / 2, x[1] / 2, x[2] / 2, 0])
                .unique()
                .collect();

            let invoke_positions_buffer =
                context
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some(&format!(
                            "Mipmap Compute Shader Invoke Positions Buffer Level {}",
                            level
                        )),
                        usage: wgpu::BufferUsages::STORAGE,
                        contents: bytemuck::cast_slice(invoke_positions.as_slice()),
                    });

            let texture_view = self.views.get(level - 1).unwrap();

            let destination_texture = self.views.get(level).unwrap();

            let bind_group = context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some(&format!("Mipmap Compute Shader Bind Group level {}", level)),
                    layout: &self.compute_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&texture_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&destination_texture),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: invoke_positions_buffer.as_entire_binding(),
                        },
                    ],
                });

            let mut compute_pass =
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });

            compute_pass.set_pipeline(&self.compute_pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);
            compute_pass.dispatch(
                ((invoke_positions.len() as f32) / 1024.0).ceil() as u32,
                1u32,
                1u32,
            );
        }

        context.queue.submit(Some(encoder.finish()));
    }
}
