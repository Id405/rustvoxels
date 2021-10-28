use std::sync::Arc;

use crevice::std430::{AsStd430, Std430};
use futures::lock::Mutex;
use glam::IVec2;
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;
use world::World;

use crate::{game::world, renderer::glsl_loader};

use self::uniforms::Uniforms;

use super::{RenderContext, Vertex, VERTICES};

mod uniforms;

pub struct Denoiser {
    world: Arc<Mutex<World>>,
    render_pipeline: wgpu::RenderPipeline,
    uniforms: Uniforms,
    uniforms_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    out_render_textures: (wgpu::Texture, wgpu::Texture),
    out_render_textures_views: (wgpu::TextureView, wgpu::TextureView),
    render_texture_view: wgpu::TextureView,
    render_texture_sampler: wgpu::Sampler,
    depth_texture_view: wgpu::TextureView,
    depth_texture_sampler: wgpu::Sampler,
    size: PhysicalSize<u32>,
    frame_count: u64,
}

impl Denoiser {
    pub async fn new(
        context: &RenderContext,
        world: Arc<Mutex<World>>,
        render_texture: &wgpu::Texture,
        depth_texture: &wgpu::Texture,
    ) -> Self {
        let size = context.window.inner_size();

        let shaders;

        unsafe {
            shaders = glsl_loader::ShaderBundle::from_path("denoiser")
                .create_shader_module_spirv(context);
        }

        let (shader_vertex, shader_fragment) = shaders;

        let uniforms = Uniforms::new(world.clone()).await;

        let uniforms_buffer =
            context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Denoising Uniforms"),
                    contents: bytemuck::cast_slice(uniforms.as_std430().as_bytes()),
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
                    label: Some("Denoising Uniform Bind Group Layout"),
                });

        let uniform_bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &uniform_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniforms_buffer.as_entire_binding(),
                }],
                label: Some("Denoising Renderer Uniform Bind Group"),
            });

        let texture_size = wgpu::Extent3d {
            width: size.width,
            height: size.height,
            depth_or_array_layers: 1,
        };

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
                                sample_type: wgpu::TextureSampleType::Float { filterable: false },
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
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 3,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler {
                                comparison: false,
                                filtering: true,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 4,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 5,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler {
                                comparison: false,
                                filtering: true,
                            },
                            count: None,
                        },
                    ],
                    label: Some("Denoising Renderer Texture Bind Group Layout"),
                });

        let render_texture_view =
            render_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let render_texture_sampler = context.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let depth_texture_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let depth_texture_sampler = context.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let render_pipeline_layout =
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Denoising Pipeline Layout"),
                    bind_group_layouts: &[&texture_bind_group_layout, &uniform_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let render_pipeline =
            context
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Denoising Render Pipeline"),
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
                            format: wgpu::TextureFormat::Rgba32Float,
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

        let out_render_textures = (
            context.device.create_texture(&wgpu::TextureDescriptor {
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba32Float,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::RENDER_ATTACHMENT,
                label: Some("raytrace render attachment"),
            }),
            context.device.create_texture(&wgpu::TextureDescriptor {
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba32Float,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::RENDER_ATTACHMENT,
                label: Some("raytrace render attachment"),
            }),
        );

        let out_render_textures_views = (
            out_render_textures
                .0
                .create_view(&wgpu::TextureViewDescriptor::default()),
            out_render_textures
                .1
                .create_view(&wgpu::TextureViewDescriptor::default()),
        );

        Self {
            world,
            render_pipeline,
            uniforms,
            uniforms_buffer,
            uniform_bind_group,
            size,
            texture_bind_group_layout,
            out_render_textures,
            out_render_textures_views,
            render_texture_view,
            render_texture_sampler,
            frame_count: 0,
            depth_texture_view,
            depth_texture_sampler,
        }
    }

    pub async fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        context: &RenderContext,
        vertex_buffer: &wgpu::Buffer,
    ) {
        self.frame_count += 1;

        self.uniforms.update(self.world.clone()).await;

        context.queue.write_buffer(
            &self.uniforms_buffer,
            0,
            bytemuck::cast_slice(self.uniforms.as_std430().as_bytes()),
        );

        let (past_render_texture_view_sample, past_render_texture_view_attachment) =
            match self.frame_count % 2 == 0 {
                true => (
                    &self.out_render_textures_views.0,
                    &self.out_render_textures_views.1,
                ),
                false => (
                    &self.out_render_textures_views.1,
                    &self.out_render_textures_views.0,
                ),
            };

        let texture_bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&self.render_texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.render_texture_sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(
                            past_render_texture_view_sample,
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::Sampler(&self.render_texture_sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::TextureView(&self.depth_texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: wgpu::BindingResource::Sampler(&self.depth_texture_sampler),
                    },
                ],
                label: Some("Denoising Texture Bind Group"),
            });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            // TODO proceed to copy the rendered texture to a new texture representing only the most recently rendered texture for texture view to use
            // Or have texture renderer update its texture every frames
            label: Some("Render Pass"),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: past_render_texture_view_attachment,
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
        render_pass.set_bind_group(0, &texture_bind_group, &[]);
        render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.draw(0..VERTICES.len() as u32, 0..1);
    }

    // TODO I really badly need a resize trait, and then textures can be registered to be resized/updated on window resize
    pub fn resize(
        &mut self,
        size: PhysicalSize<u32>,
        render_texture: &wgpu::Texture,
        depth_texture: &wgpu::Texture,
        context: &RenderContext,
    ) {
        self.render_texture_view =
            render_texture.create_view(&wgpu::TextureViewDescriptor::default());

        self.depth_texture_view =
            depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let texture_size = wgpu::Extent3d {
            width: size.width,
            height: size.height,
            depth_or_array_layers: 1,
        };

        self.out_render_textures = (
            context.device.create_texture(&wgpu::TextureDescriptor {
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba32Float,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::RENDER_ATTACHMENT,
                label: Some("raytrace render attachment"),
            }),
            context.device.create_texture(&wgpu::TextureDescriptor {
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba32Float,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::RENDER_ATTACHMENT,
                label: Some("raytrace render attachment"),
            }),
        );

        self.out_render_textures_views = (
            self.out_render_textures
                .0
                .create_view(&wgpu::TextureViewDescriptor::default()),
            self.out_render_textures
                .1
                .create_view(&wgpu::TextureViewDescriptor::default()),
        );

        self.size = size;
    }

    pub fn render_textures(&self) -> (&wgpu::Texture, &wgpu::Texture) {
        (&self.out_render_textures.0, &self.out_render_textures.1)
    }

    pub fn render_texture_view(&self) -> &wgpu::TextureView {
        // TODO; check to make sure this returns the correct, most recently rendered texture view.
        match self.frame_count % 2 == 0 {
            true => &self.out_render_textures_views.1,
            false => &self.out_render_textures_views.0,
        }
    }
}
