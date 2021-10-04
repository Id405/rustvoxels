use std::sync::Arc;

use crate::game::World;

use self::uniforms::Uniforms;
use futures::lock::Mutex;
use wgpu::util::DeviceExt;

use super::{glsl_loader, CameraUniform, RenderContext, Vertex};

mod uniforms;

pub struct RenderState {
    size: winit::dpi::PhysicalSize<u32>,
    frame_count: u32,
}

impl RenderState {
    pub fn size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.size
    }
}

pub struct Raytracer {
    render_pipeline: wgpu::RenderPipeline,
    raytrace_uniforms: Uniforms,
    camera_uniforms: CameraUniform,
    raytrace_uniform_buffer: wgpu::Buffer,
    camera_uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    world_bind_group: wgpu::BindGroup,
    render_state: RenderState,
    world: Arc<Mutex<World>>,
}

impl Raytracer {
    pub async fn new(
        context: &RenderContext,
        world: Arc<Mutex<World>>,
        sc_desc: &wgpu::SurfaceConfiguration,
    ) -> Self {
        let size = context.window.inner_size();

        let render_state = RenderState {
            size,
            frame_count: 0,
        };

        let shader_bundle = glsl_loader::ShaderBundle::from_path("raytrace"); // todo: live reloading

        let shader_vertex;
        let shader_fragment;

        unsafe {
            shader_vertex =
                context
                    .device
                    .create_shader_module_spirv(&wgpu::ShaderModuleDescriptorSpirV {
                        label: Some("raytrace_vertex"),
                        source: shader_bundle.vertex,
                    });

            shader_fragment =
                context
                    .device
                    .create_shader_module_spirv(&wgpu::ShaderModuleDescriptorSpirV {
                        label: Some("raytrace_fragment"),
                        source: shader_bundle.fragment,
                    });
        }

        // shader_vertex = context.device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        //     label: Some("raytrace_vertex"),
        //     source: ShaderSource::SpirV(shader_bundle.vertex)
        // });

        // shader_fragment= context.device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        //     label: Some("raytrace_fragment"),
        //     source: ShaderSource::SpirV(shader_bundle.fragment)
        // });

        let raytrace_uniforms = Uniforms::new(world.clone(), &render_state).await;
        let camera_uniforms = CameraUniform::new(world.clone(), &context).await;

        let raytrace_uniform_buffer =
            context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Raytracing Uniforms"),
                    contents: bytemuck::cast_slice(&[raytrace_uniforms]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        let camera_uniform_buffer =
            context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Camera Uniforms"),
                    contents: bytemuck::cast_slice(&[camera_uniforms]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        let uniform_bind_group_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                    label: Some("uniform_bind_group_layout"),
                });

        let uniform_bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &uniform_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: raytrace_uniform_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: camera_uniform_buffer.as_entire_binding(),
                    },
                ],
                label: Some("uniform_bind_group"),
            });

        let world_lock = world.lock().await;
        let scene = world_lock
            .voxel_grid
            .as_ref()
            .expect("ERROR: expected resource not present");

        let world_texture = scene.as_texture();

        let world_texture_view = world_texture.create_view(&wgpu::TextureViewDescriptor::default());
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
                        // wgpu::BindGroupLayoutEntry {
                        //     binding: 1,
                        //     visibility: wgpu::ShaderStages::FRAGMENT,
                        //     ty: wgpu::BindingType::Sampler {
                        //         // This is only for TextureSampleType::Depth
                        //         comparison: false,
                        //         // This should be true if the sample_type of the texture is:
                        //         //     TextureSampleType::Float { filterable: true }
                        //         // Otherwise you'll get an error.
                        //         filtering: true,
                        //     },
                        //     count: None,
                        // },
                    ],
                    label: Some("texture_bind_group_layout"),
                });

        let world_bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&world_texture_view),
                    },
                    // wgpu::BindGroupEntry {
                    //     binding: 1,
                    //     resource: wgpu::BindingResource::Sampler(&world_sampler),
                    // },
                ],
                label: Some("world_bind_group"),
            });

        let render_pipeline_layout =
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[&texture_bind_group_layout, &uniform_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let render_pipeline =
            context
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Render Pipeline"),
                    layout: Some(&render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        buffers: &[Vertex::desc()],
                        module: &shader_vertex,
                        entry_point: "main",
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
            raytrace_uniforms,
            raytrace_uniform_buffer,
            uniform_bind_group,
            render_state,
            world_bind_group,
            camera_uniforms,
            camera_uniform_buffer,
            world: world.clone(),
        }
    }

    pub fn render_pipeline(&self) -> &wgpu::RenderPipeline {
        &self.render_pipeline
    }

    pub async fn update_uniform_data(&mut self, context: &RenderContext) {
        self.raytrace_uniforms
            .update(self.world.clone(), &self.render_state)
            .await;
        self.camera_uniforms
            .update(self.world.clone(), context)
            .await;
        context.queue.write_buffer(
            &self.raytrace_uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.raytrace_uniforms]),
        );
        context.queue.write_buffer(
            &self.camera_uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniforms]),
        );
    }

    pub fn uniform_bind_group(&self) -> &wgpu::BindGroup {
        &self.uniform_bind_group
    }

    pub fn world_bind_group(&self) -> &wgpu::BindGroup {
        &self.world_bind_group
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.render_state.size = new_size;
    }

    #[deprecated]
    pub fn frame_complete(&mut self) {
        self.render_state.frame_count += 1;
    }
}
