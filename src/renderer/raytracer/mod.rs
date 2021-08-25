use crate::game::World;

use self::uniforms::Uniforms;
use wgpu::util::DeviceExt;

use super::{glsl_loader, Vertex};

mod uniforms;

pub struct RenderState {
    // This type should go away and become part of the resources type
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
    uniforms: Uniforms,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    render_state: RenderState,
}

impl Raytracer {
    pub fn new(
        window: &winit::window::Window,
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
        world: &World,
    ) -> Self {
        let size = window.inner_size();

        let shader_bundle = glsl_loader::ShaderBundle::from_path("raytrace"); // todo: live reloading

        let shader_vertex = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            flags: wgpu::ShaderFlags::all(),
            source: shader_bundle.vertex,
        });

        let shader_fragment = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            flags: wgpu::ShaderFlags::all(),
            source: shader_bundle.fragment,
        });

        let uniforms = Uniforms::new(world);

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Raytracing Uniforms"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("uniform_bind_group_layout"),
            });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("uniform_bind_group"),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&uniform_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
                    write_mask: wgpu::ColorWrite::ALL,
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

        let state = RenderState {
            size,
            frame_count: 0,
        };

        Self {
            render_pipeline,
            uniforms,
            uniform_buffer,
            uniform_bind_group,
            render_state: state,
        }
    }

    pub fn render_pipeline(&self) -> &wgpu::RenderPipeline {
        &self.render_pipeline
    }

    pub fn update_uniform_data(&mut self, queue: &wgpu::Queue, world: &World) {
        self.uniforms.update(world, &self.render_state);
        queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.uniforms]),
        );
    }

    pub fn uniform_bind_group(&self) -> &wgpu::BindGroup {
        &self.uniform_bind_group
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.render_state.size = new_size;
    }

    #[deprecated]
    pub fn frame_complete(&mut self) {
        self.render_state.frame_count += 1;
    }
}
