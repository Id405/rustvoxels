use std::{cell::RefCell, rc::Rc, sync::Arc};

use crate::game::World;

use self::uniforms::Uniforms;
use futures::lock::Mutex;
use wgpu::{util::DeviceExt, Buffer, CommandEncoder, Texture, TextureView};

use crevice::std430::{AsStd430, Std430};

use super::{glsl_loader, texture_atlas::TextureAtlas, RenderContext, Vertex, VERTICES};

mod uniforms;

pub struct Raytracer {
    render_pipeline: wgpu::RenderPipeline,
    raytrace_uniforms: Uniforms,
    raytrace_uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    world_bind_group: wgpu::BindGroup,
    world: Arc<Mutex<World>>,
    atlas: Rc<RefCell<TextureAtlas>>,
}

impl Raytracer {
    pub async fn new(
        context: &RenderContext,
        world: Arc<Mutex<World>>,
        sc_desc: &wgpu::SurfaceConfiguration,
        atlas: Rc<RefCell<TextureAtlas>>,
    ) -> Self {
        let size = context.window.inner_size();

        let shaders;

        unsafe {
            shaders = glsl_loader::ShaderBundle::from_path("raytrace")
                .create_shader_module_spirv(context);
        }

        let (shader_vertex, shader_fragment) = shaders;

        let raytrace_uniforms = Uniforms::new(context, world.clone(), atlas.clone()).await;

        let raytrace_uniform_buffer =
            context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Raytracing Uniforms"),
                    contents: bytemuck::cast_slice(raytrace_uniforms.as_std430().as_bytes()),
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
                    label: Some("uniform_bind_group_layout"),
                });

        let uniform_bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &uniform_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: raytrace_uniform_buffer.as_entire_binding(),
                }],
                label: Some("uniform_bind_group"),
            });

        let world_lock = world.lock().await;

        let world_texture_view = atlas
            .borrow_mut()
            .get_view("voxelizer_attachment_world", context)
            .unwrap();

        // let world_sampler = context.device.create_sampler(&wgpu::SamplerDescriptor {
        //     address_mode_u: wgpu::AddressMode::ClampToEdge,
        //     address_mode_v: wgpu::AddressMode::ClampToEdge,
        //     address_mode_w: wgpu::AddressMode::ClampToEdge,
        //     mag_filter: wgpu::FilterMode::Nearest,
        //     min_filter: wgpu::FilterMode::Nearest,
        //     mipmap_filter: wgpu::FilterMode::Linear,
        //     ..Default::default()
        // });

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
                                view_dimension: wgpu::TextureViewDimension::D3,
                                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            },
                            count: None,
                        },
                    ],
                    label: Some("texture_bind_group_layout"),
                });

        let render_pipeline_layout =
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Raytracer Render Pipeline Layout"),
                    bind_group_layouts: &[&texture_bind_group_layout, &uniform_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let render_pipeline =
            context
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Raytracer Render Pipeline"),
                    layout: Some(&render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        buffers: &[Vertex::desc()],
                        module: &shader_vertex,
                        entry_point: "main",
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader_fragment,
                        entry_point: "main",
                        targets: &[
                            wgpu::ColorTargetState {
                                format: wgpu::TextureFormat::Rgba32Float,
                                blend: None,
                                write_mask: wgpu::ColorWrites::ALL,
                            },
                            wgpu::ColorTargetState {
                                format: wgpu::TextureFormat::Rgba32Float,
                                blend: None,
                                write_mask: wgpu::ColorWrites::ALL,
                            },
                            wgpu::ColorTargetState {
                                format: wgpu::TextureFormat::Rgba32Float,
                                blend: None,
                                write_mask: wgpu::ColorWrites::ALL,
                            },
                        ],
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: None,
                        polygon_mode: wgpu::PolygonMode::Fill,
                        conservative: false,
                        unclipped_depth: false,
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    multiview: None,
                });

        {
            let mut atlas_lock = atlas.borrow_mut();

            atlas_lock.register("raytracer_attachment_color", context);
            atlas_lock.register("raytracer_attachment_depth", context);
            atlas_lock.register("raytracer_attachment_world_position", context);
            atlas_lock.register_from_image(
                "raytracer_binding_noise",
                include_bytes!("../../../assets/textures/LDR_RGB1_0.png"),
                context,
            );
        }

        let world_bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&world_texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(
                            &atlas
                                .borrow()
                                .get_view("raytracer_binding_noise", context)
                                .unwrap(),
                        ),
                    },
                ],
                label: Some("world_bind_group"),
            });

        Self {
            render_pipeline,
            raytrace_uniforms,
            raytrace_uniform_buffer,
            uniform_bind_group,
            world_bind_group,
            world: world.clone(),
            atlas,
        }
    }

    pub async fn render(
        &mut self,
        encoder: &mut CommandEncoder,
        context: &RenderContext,
        vertex_buffer: &Buffer,
    ) {
        self.update_uniform_data(&context).await; // uniform data must be kept up to date before rendering is performed

        let atlas = self.atlas.borrow();

        let raytracer_attachment_color = &atlas
            .get_view("raytracer_attachment_color", context)
            .unwrap();
        let raytracer_attachment_depth = &atlas
            .get_view("raytracer_attachment_depth", context)
            .unwrap();
        let raytracer_attachment_world_position = &atlas
            .get_view("raytracer_attachment_world_position", context)
            .unwrap();

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Raytracer Render Pass"),
            color_attachments: &[
                wgpu::RenderPassColorAttachment {
                    view: raytracer_attachment_color,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                },
                wgpu::RenderPassColorAttachment {
                    view: raytracer_attachment_depth,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                },
                wgpu::RenderPassColorAttachment {
                    view: raytracer_attachment_world_position,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                },
            ],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(self.render_pipeline());
        render_pass.set_bind_group(1, self.uniform_bind_group(), &[]);
        render_pass.set_bind_group(0, self.world_bind_group(), &[]);
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.draw(0..VERTICES.len() as u32, 0..1);
    }

    fn render_pipeline(&self) -> &wgpu::RenderPipeline {
        &self.render_pipeline
    }

    async fn update_uniform_data(&mut self, context: &RenderContext) {
        self.raytrace_uniforms.update(context, self.world.clone(), self.atlas.clone()).await;
        context.queue.write_buffer(
            &self.raytrace_uniform_buffer,
            0,
            bytemuck::cast_slice(self.raytrace_uniforms.as_std430().as_bytes()),
        );
    }

    fn uniform_bind_group(&self) -> &wgpu::BindGroup {
        &self.uniform_bind_group
    }

    fn world_bind_group(&self) -> &wgpu::BindGroup {
        &self.world_bind_group
    }

    // TOOD; move into resize trait
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>, context: &RenderContext) {}
}
