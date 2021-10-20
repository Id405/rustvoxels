use std::{rc::Rc, sync::Arc};

use futures::lock::Mutex;
use wgpu::{util::DeviceExt, CommandEncoder};
use winit::{dpi::PhysicalSize, window};

use crate::game::World;

pub use common_uniforms::CameraUniform;
pub use render_context::RenderContext;

mod common_uniforms;
mod glsl_loader;
mod raytracer;
mod render_context;
mod texture_renderer;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
}

// main.rs
impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-1.0, -1.0, 0.0],
    },
    Vertex {
        position: [-1.0, 1.0, 0.0],
    },
    Vertex {
        position: [1.0, -1.0, 0.0],
    },
    Vertex {
        position: [-1.0, 1.0, 0.0],
    },
    Vertex {
        position: [1.0, 1.0, 0.0],
    },
    Vertex {
        position: [1.0, -1.0, 0.0],
    },
];

pub struct Renderer {
    surface_config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    vertex_buffer: wgpu::Buffer,
    raytracer: raytracer::Raytracer,
    texture_renderer: texture_renderer::TextureRenderer,
    world: Arc<Mutex<World>>,
}

impl Renderer {
    // Creating some of the wgpu types requires async code
    pub async fn new(context: &RenderContext, world: Arc<Mutex<World>>) -> Renderer {
        let size = context.window.inner_size();

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: context
                .surface
                .get_preferred_format(&context.adapter)
                .unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };

        context.surface.configure(&context.device, &surface_config);

        // let swap_chain = context.device.create_swap_chain(&context.surface, &sc_desc);

        let raytracer = raytracer::Raytracer::new(context, world.clone(), &surface_config).await; // the raytracer struct should hold its own swapchain in the future, or whatever the compute shader equivilant is

        let texture_renderer = texture_renderer::TextureRenderer::new(context, &surface_config, raytracer.render_texture_view()).await;

        let vertex_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            });

        Self {
            surface_config,
            size,
            vertex_buffer,
            raytracer,
            world,
            texture_renderer,
        }
    }

    pub fn resize(&mut self, context: &RenderContext, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.surface_config.width = new_size.width;
        self.surface_config.height = new_size.height;
        context
            .surface
            .configure(&context.device, &self.surface_config);

        self.raytracer.resize(new_size, &context);
        self.texture_renderer.resize(new_size, self.raytracer.render_texture_view(), &context);
    }

    pub fn input(&mut self, event: &winit::event::WindowEvent) -> bool {
        // This function should only be used for accepting debug commands for the renderer
        false
    }

    pub fn update(&mut self) {
        // remove `todo!()`
    }

    pub async fn render(&mut self, context: &RenderContext) -> Result<(), wgpu::SurfaceError> {
        let frame = context.surface.get_current_frame()?.output;

        let mut encoder = context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        self.raytracer
            .render(&mut encoder, &context, &self.vertex_buffer)
            .await;

        self.texture_renderer
            .render(
                &mut encoder,
                &context,
                &self.vertex_buffer,
                &frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default()),
            )
            .await;

        // submit will accept anything that implements IntoIter
        context.queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }

    fn render_texture(&self, context: &RenderContext, encoder: &CommandEncoder) {}

    pub fn size(&self) -> PhysicalSize<u32> {
        self.size
    }
}
