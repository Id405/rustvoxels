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


pub struct Clear {
    compute_pipeline: wgpu::ComputePipeline,
    compute_bind_group_layout: wgpu::BindGroupLayout,
    world: Arc<Mutex<World>>,
    mip_levels: u32,
    atlas: Rc<RefCell<TextureAtlas>>,
    bind_groups: Vec<wgpu::BindGroup>,
    width: i32,
    height: i32,
    length: i32,
}

impl Clear {
    pub async fn new(
        context: &RenderContext,
        world: Arc<Mutex<World>>,
        atlas: Rc<RefCell<TextureAtlas>>,
    ) -> Self {
        let (width, height, length) = (128, 128, 128);

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

        let bind_groups = (1..mip_levels)
            .map(|level| {
                let texture_view = views.get(level as usize - 1).unwrap();

                let destination_texture = views.get(level as usize).unwrap();

                context
                    .device
                    .create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some(&format!("Mipmap Compute Shader Bind Group level {}", level)),
                        layout: &compute_bind_group_layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::TextureView(&texture_view),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::TextureView(&destination_texture),
                            },
                        ],
                    })
            })
            .collect::<Vec<_>>();

        Self {
            compute_pipeline,
            world,
            mip_levels,
            atlas: atlas.clone(),
            compute_bind_group_layout,
            bind_groups,
            width,
            height,
            length,
        }
    }

    pub async fn render(&mut self, context: &RenderContext) {
        let mut encoder = context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let mut compute_pass =
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });

            for level in 1..self.mip_levels as usize {
                let div_factor = (2i32).pow(level as u32);
                let (width, height, length) = (
                    self.width / div_factor,
                    self.height / div_factor,
                    self.length / div_factor,
                );

                compute_pass.set_pipeline(&self.compute_pipeline);
                compute_pass.set_bind_group(0, &self.bind_groups.get(level - 1).unwrap(), &[]);
                compute_pass.dispatch(
                    ((width as u32)).max(1),
                    ((length as u32)).max(1),
                    ((height as u32)).max(1),
                );
            }
        }

        context.queue.submit(Some(encoder.finish()));
    }
}
