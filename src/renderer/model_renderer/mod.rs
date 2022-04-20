use std::sync::Arc;

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
    renderer::glsl_loader,
};

use self::uniforms::Uniforms;

use super::{RenderContext, VERTICES};

mod uniforms;

pub struct ModelRenderer {
    render_pipeline: wgpu::RenderPipeline,
    uniforms: Uniforms,
    uniforms_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    depth_texture: wgpu::Texture,
    depth_texture_view: wgpu::TextureView,
    size: PhysicalSize<u32>,
    world: Arc<Mutex<World>>,
}

impl ModelRenderer {
    pub async fn new(
        context: &RenderContext,
        world: Arc<Mutex<World>>,
        sc_desc: &wgpu::SurfaceConfiguration,
    ) -> Self {
        let size = context.window.inner_size();

        let shaders;

        unsafe {
            shaders = glsl_loader::ShaderBundle::from_path("model_renderer")
                .create_shader_module_spirv(context);
        }

        let (shader_vertex, shader_fragment) = shaders;

        let uniforms = Uniforms::new(world.clone()).await;

        let uniforms_buffer =
            context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Model Rendering Uniforms"),
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
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                    label: Some("Model Renderer Uniform Bind Group Layout"),
                });

        let uniform_bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &uniform_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniforms_buffer.as_entire_binding(),
                }],
                label: Some("Model Renderer Uniform Bind Group"),
            });

        let render_pipeline_layout =
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Model Renderer Pipeline Layout"),
                    bind_group_layouts: &[&uniform_bind_group_layout, &material_layout],
                    push_constant_ranges: &[],
                });

        let render_pipeline =
            context
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Model Renderer Render Pipeline"),
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
                            format: sc_desc.format,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        }],
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: Some(wgpu::Face::Back),
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
            width: size.width,
            height: size.height,
            depth_or_array_layers: 1,
        };

        let desc = wgpu::TextureDescriptor {
            label: Some("Model Renderer Depth Texture"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24Plus,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        };

        let depth_texture = context.device.create_texture(&desc);

        let depth_texture_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            render_pipeline,
            uniforms,
            uniforms_buffer,
            uniform_bind_group,
            depth_texture,
            depth_texture_view,
            size,
            world,
        }
    }

    pub async fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        context: &RenderContext,
        vertex_buffer: &wgpu::Buffer,
        render_texture_view: &wgpu::TextureView,
    ) {
        self.uniforms.update(self.world.clone()).await;

        let world = self.world.lock().await;

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: render_texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
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

            // render_pass.set_pipeline(&self.render_pipeline);
            // render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);

            let components = world.get_components::<Model>();

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            // render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            // render_pass.draw(0..VERTICES.len() as u32, 0..1);

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

    pub fn resize(
        &mut self,
        size: PhysicalSize<u32>,
        render_texture_view: &wgpu::TextureView,
        context: &RenderContext,
    ) {
        let render_texture_sampler = context
            .device
            .create_sampler(&wgpu::SamplerDescriptor::default());

        let extent = wgpu::Extent3d {
            width: size.width,
            height: size.height,
            depth_or_array_layers: 1,
        };

        let desc = wgpu::TextureDescriptor {
            label: Some("Model Renderer Depth Texture"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24Plus,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        };

        self.depth_texture = context.device.create_texture(&desc);

        self.depth_texture_view = self
            .depth_texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.size = size;
    }
}
