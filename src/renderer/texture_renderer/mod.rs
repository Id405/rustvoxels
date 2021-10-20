use glam::IVec2;
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;

use crate::renderer::glsl_loader;

use self::uniforms::Uniforms;

use super::{RenderContext, Vertex, VERTICES};

mod uniforms;

pub struct TextureRenderer {
    render_pipeline: wgpu::RenderPipeline,
    uniforms: Uniforms,
    uniforms_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    texture_bind_group: wgpu::BindGroup,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    size: PhysicalSize<u32>,
}

impl TextureRenderer {
    pub async fn new(
        context: &RenderContext,
        sc_desc: &wgpu::SurfaceConfiguration,
        render_texture_view: &wgpu::TextureView,
    ) -> Self {
        let size = context.window.inner_size();

        let shaders;

        unsafe {
            shaders = glsl_loader::ShaderBundle::from_path("texture_renderer")
                .create_shader_module_spirv(context);
        }

        let (shader_vertex, shader_fragment) = shaders;

        let uniforms = Uniforms::new(IVec2::new(size.width as i32, size.height as i32)).await;

        let uniforms_buffer =
            context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Texture Rendering Uniforms"),
                    contents: bytemuck::cast_slice(&[uniforms]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        let uniform_bind_group_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                    label: Some("Texture Renderer Uniform Bind Group Layout"),
                });

        let uniform_bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &uniform_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniforms_buffer.as_entire_binding(),
                }],
                label: Some("Texture Renderer Uniform Bind Group"),
            });

        let texture_bind_group_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                            ty: wgpu::BindingType::Sampler {
                                comparison: false,
                                filtering: true,
                            },
                            count: None,
                        },
                    ],
                    label: Some("Texture Renderer Texture Bind Group Layout"),
                });

        let render_texture_sampler = context.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let texture_bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(render_texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&render_texture_sampler),
                    },
                ],
                label: Some("Render Texture Bind Group"),
            });

        let render_pipeline_layout =
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Texture Renderer Pipeline Layout"),
                    bind_group_layouts: &[&texture_bind_group_layout, &uniform_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let render_pipeline =
            context
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Texture Renderer Render Pipeline"),
                    layout: Some(&render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader_vertex,
                        entry_point: "main",
                        buffers: &[Vertex::desc()],
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
                        cull_mode: None,
                        polygon_mode: wgpu::PolygonMode::Fill,
                        clamp_depth: false,
                        conservative: false,
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                });

        Self {
            render_pipeline,
            uniforms,
            uniforms_buffer,
            uniform_bind_group,
            texture_bind_group,
            size,
            texture_bind_group_layout,
        }
    }

    pub async fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        context: &RenderContext,
        vertex_buffer: &wgpu::Buffer,
        surface_view: &wgpu::TextureView,
    ) {
        self.uniforms
            .update(IVec2::new(self.size.width as i32, self.size.height as i32))
            .await;

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: surface_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 1.0,
                        g: 0.8,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.texture_bind_group, &[]);
        render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.draw(0..VERTICES.len() as u32, 0..1);
    }

    pub fn resize(
        &mut self,
        size: PhysicalSize<u32>,
        render_texture_view: &wgpu::TextureView,
        context: &RenderContext,
    ) {
        let render_texture_sampler = context.device.create_sampler(&wgpu::SamplerDescriptor::default());

        self.size = size;
        self.texture_bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(render_texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&render_texture_sampler),
                    },
                ],
                label: Some("Render Texture Bind Group"),
            });
    }
}
