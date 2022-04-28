use std::{cell::RefCell, convert::TryInto, mem::size_of, num::NonZeroU32, rc::Rc, sync::Arc};

use crevice::std430::{AsStd430, Std430};
use futures::lock::Mutex;
use glam::IVec2;
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

use super::texture_atlas::TextureAtlas;

mod uniforms;

pub struct Voxelizer {
    render_pipeline: wgpu::RenderPipeline,
    uniforms: Uniforms,
    uniforms_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    image_write_bind_group: wgpu::BindGroup,
    depth_texture: wgpu::Texture,
    depth_texture_view: wgpu::TextureView,
    render_texture: wgpu::Texture,
    render_texture_view: wgpu::TextureView,
    size: PhysicalSize<u32>,
    world: Arc<Mutex<World>>,
    atlas: Rc<RefCell<TextureAtlas>>,
}

impl Voxelizer {
    pub async fn new(
        context: &RenderContext,
        world: Arc<Mutex<World>>,
        atlas: Rc<RefCell<TextureAtlas>>,
    ) -> Self {
        let size = context.window.inner_size();

        let shaders;

        unsafe {
            shaders = glsl_loader::ShaderBundle::from_path("voxelizer")
                .create_shader_module_spirv(context);
        }

        let (shader_vertex, shader_fragment) = shaders;

        let texture_size = wgpu::Extent3d {
            width: 128,
            height: 128,
            depth_or_array_layers: 128,
        };

        atlas.borrow_mut().register_from_descriptor(
            "voxelizer_attachment_world",
            wgpu::TextureDescriptor {
                size: texture_size,
                mip_level_count: ((((texture_size.width)
                    .min((texture_size.height).min(texture_size.depth_or_array_layers))
                    as f32)
                    .log2())
                .floor()
                    - 0.0)
                    .max(0.0) as u32
                    - 1,
                label: Some("scene_texture"),
                sample_count: 1,
                dimension: wgpu::TextureDimension::D3,
                format: wgpu::TextureFormat::Rgba32Float,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::STORAGE_BINDING,
            },
            context,
        );

        let uniforms = Uniforms::new(context, world.clone(), atlas.clone()).await;

        let uniforms_buffer =
            context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Voxelizing Uniforms"),
                    contents: bytemuck::cast_slice(uniforms.as_std430().as_bytes()),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        let material_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                });

        let uniform_bind_group_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                    label: Some("Voxelizer Uniform Bind Group Layout"),
                });

        let uniform_bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &uniform_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniforms_buffer.as_entire_binding(),
                }],
                label: Some("Voxelizer Uniform Bind Group"),
            });

        let image_write_bind_group_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Voxelizer Image Write Bind Group Layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            ty: wgpu::BindingType::StorageTexture {
                                access: wgpu::StorageTextureAccess::WriteOnly,
                                format: wgpu::TextureFormat::Rgba32Float,
                                view_dimension: wgpu::TextureViewDimension::D3,
                            },
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            count: None,
                        },
                    ],
                });

        let image_write_bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &image_write_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(
                            &atlas
                                .borrow()
                                .get_view_descriptor(
                                    "voxelizer_attachment_world",
                                    &wgpu::TextureViewDescriptor {
                                        label: Some("mip"),
                                        format: None,
                                        dimension: Some(wgpu::TextureViewDimension::D3),
                                        aspect: wgpu::TextureAspect::All,
                                        base_mip_level: 0,
                                        mip_level_count: NonZeroU32::new(1),
                                        base_array_layer: 0,
                                        array_layer_count: None,
                                    },
                                    context,
                                )
                                .unwrap(),
                        ),
                    }
                ],
                label: Some("Voxelizer Image Write Bind Group"),
            });

        let render_pipeline_layout =
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Voxelizer Pipeline Layout"),
                    bind_group_layouts: &[
                        &uniform_bind_group_layout,
                        &material_layout,
                        &image_write_bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                });

        let render_pipeline =
            context
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Voxelizer Render Pipeline"),
                    layout: Some(&render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader_vertex,
                        entry_point: "main",
                        buffers: &[ModelVertex::desc()],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader_fragment,
                        entry_point: "main",
                        targets: &[wgpu::ColorTargetState {
                            format: wgpu::TextureFormat::R32Float,
                            blend: None,
                            write_mask: wgpu::ColorWrites::ALL,
                        }],
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: None,
                        polygon_mode: wgpu::PolygonMode::Fill,
                        unclipped_depth: false,
                        conservative: false,
                    },
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: wgpu::TextureFormat::Depth24Plus,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default(),
                    }),
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    multiview: None,
                });

        let extent = wgpu::Extent3d {
            width: 128, //TODO fix hardcoded scene size
            height: 128,
            depth_or_array_layers: 1,
        };

        let depth_texture = context.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Voxelizer Depth Texture"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24Plus,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        });

        let depth_texture_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let render_texture = context.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Voxelizer Render Texture"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        });

        let render_texture_view =
            render_texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            render_pipeline,
            uniforms,
            uniforms_buffer,
            uniform_bind_group,
            depth_texture,
            depth_texture_view,
            render_texture,
            render_texture_view,
            size,
            world,
            image_write_bind_group,
            atlas: atlas.clone(),
        }
    }

    pub async fn render(&mut self, encoder: &mut wgpu::CommandEncoder, context: &RenderContext) {
        self.uniforms
            .update(context, self.world.clone(), self.atlas.clone())
            .await;

        let world = self.world.lock().await;

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Voxelizer Render Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &self.render_texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::default()),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            let components = world.get_components::<Model>();

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.set_bind_group(2, &self.image_write_bind_group, &[]);

            for model in components.clone() {
                self.uniforms
                    .update_model_matrix(model.transform.model_matrix())
                    .await;
                context.queue.write_buffer(
                    &self.uniforms_buffer,
                    0,
                    bytemuck::cast_slice(self.uniforms.as_std430().as_bytes()),
                );

                model.render(&mut render_pass, &self.uniform_bind_group);
            }
        }
    }
}
