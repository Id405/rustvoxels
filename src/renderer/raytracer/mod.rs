use std::sync::Arc;

use crate::game::World;

use self::uniforms::Uniforms;
use futures::lock::Mutex;
use wgpu::{util::DeviceExt, Buffer, CommandEncoder, Texture, TextureView};

use crevice::std430::{AsStd430, Std430};

use super::{glsl_loader, RenderContext, Vertex, VERTICES};

mod uniforms;

pub struct Raytracer {
    render_pipeline: wgpu::RenderPipeline,
    raytrace_uniforms: Uniforms,
    raytrace_uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    world_bind_group: wgpu::BindGroup,
    render_texture: Texture,
    render_texture_view: TextureView,
    world: Arc<Mutex<World>>,
}

impl Raytracer {
    // TODO; renderable trait
    pub async fn new(
        context: &RenderContext,
        world: Arc<Mutex<World>>,
        sc_desc: &wgpu::SurfaceConfiguration,
    ) -> Self {
        let size = context.window.inner_size();

        let shaders;

        unsafe {
            shaders = glsl_loader::ShaderBundle::from_path("raytrace")
                .create_shader_module_spirv(context);
        }

        let (shader_vertex, shader_fragment) = shaders;

        let raytrace_uniforms = Uniforms::new(world.clone()).await;

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
                        targets: &[wgpu::ColorTargetState {
                            format: wgpu::TextureFormat::Rgba8UnormSrgb,
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

        let size = world_lock.player.as_ref().unwrap().camera.size;

        let render_texture_size = wgpu::Extent3d {
            width: size.width,
            height: size.height,
            depth_or_array_layers: 1,
        };

        let render_texture = context.device.create_texture(&wgpu::TextureDescriptor {
            size: render_texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: Some("raytrace render attachment"),
        });

        let render_texture_view =
            render_texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            render_pipeline,
            raytrace_uniforms,
            raytrace_uniform_buffer,
            uniform_bind_group,
            world_bind_group,
            render_texture,
            render_texture_view,
            world: world.clone(),
        }
    }

    pub async fn render(
        &mut self,
        encoder: &mut CommandEncoder,
        context: &RenderContext,
        vertex_buffer: &Buffer,
    ) {
        self.update_uniform_data(&context).await; // uniform data must be kept up to date before rendering is performed

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Raytracer Render Pass"),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: self.render_texture_view(),
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
            }],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(self.render_pipeline()); // TODO: rendering structs take control of their own swap chain and are interacted with through a RenderStruct trait
        render_pass.set_bind_group(1, self.uniform_bind_group(), &[]);
        render_pass.set_bind_group(0, self.world_bind_group(), &[]);
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.draw(0..VERTICES.len() as u32, 0..1);
    }

    pub fn render_texture(&self) -> &Texture {
        &self.render_texture
    }

    pub fn render_texture_view(&self) -> &TextureView {
        &self.render_texture_view
    }

    fn render_pipeline(&self) -> &wgpu::RenderPipeline {
        &self.render_pipeline
    }

    async fn update_uniform_data(&mut self, context: &RenderContext) {
        self.raytrace_uniforms.update(self.world.clone()).await;
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

    #[deprecated] // TOOD; move into resize trait
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>, context: &RenderContext) {
        let render_texture_size = wgpu::Extent3d {
            width: new_size.width,
            height: new_size.height,
            depth_or_array_layers: 1,
        };

        let render_texture = context.device.create_texture(&wgpu::TextureDescriptor {
            size: render_texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: Some("raytrace render attachment"),
        });

        let render_texture_view =
            render_texture.create_view(&wgpu::TextureViewDescriptor::default());

        self.render_texture = render_texture;
        self.render_texture_view = render_texture_view;
    }
}
