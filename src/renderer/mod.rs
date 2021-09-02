use wgpu::util::DeviceExt;
use winit::{dpi::PhysicalSize, window};

use crate::game::World;

mod glsl_loader;
mod raytracer;

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
            step_mode: wgpu::InputStepMode::Vertex,
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

pub struct RenderContext {
    pub window: winit::window::Window,
    pub instance: wgpu::Instance,
    pub surface: wgpu::Surface,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl RenderContext {
    pub async fn new(event_loop: &winit::event_loop::EventLoop<()>) -> RenderContext {
        let window = winit::window::WindowBuilder::new()
            .with_title("rustvoxels")
            .build(&event_loop)
            .unwrap();

        window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));

        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(&window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();

        Self {
            window,
            instance,
            surface,
            adapter,
            device,
            queue,
        }
    }
}

pub struct Renderer {
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    size: winit::dpi::PhysicalSize<u32>,
    vertex_buffer: wgpu::Buffer,
    raytracer: raytracer::Raytracer,
}

impl Renderer {
    // Creating some of the wgpu types requires async code
    pub async fn new(context: &RenderContext, world: &World) -> Renderer {
        let size = context.window.inner_size();

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: context
                .adapter
                .get_swap_chain_preferred_format(&context.surface)
                .unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };

        let swap_chain = context.device.create_swap_chain(&context.surface, &sc_desc);

        let raytracer = raytracer::Raytracer::new(context, &sc_desc, &world); // the raytracer struct should hold its own swapchain in the future, or whatever the compute shader equivilant is

        let vertex_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsage::VERTEX,
            });

        Self {
            sc_desc,
            swap_chain,
            size,
            vertex_buffer,
            raytracer,
        }
    }

    pub fn resize(&mut self, context: &RenderContext, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.swap_chain = context
            .device
            .create_swap_chain(&context.surface, &self.sc_desc);

        self.raytracer.resize(new_size);
    }

    pub fn input(&mut self, event: &winit::event::WindowEvent) -> bool {
        // This function should only be used for accepting debug commands for the renderer
        false
    }

    pub fn update(&mut self) {
        // remove `todo!()`
    }

    pub fn render(
        &mut self,
        context: &RenderContext,
        world: &World,
    ) -> Result<(), wgpu::SwapChainError> {
        let frame = self.swap_chain.get_current_frame()?.output;

        let mut encoder = context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        self.raytracer.update_uniform_data(&context, world); // uniform data must be kept up to date before rendering is performed

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &frame.view,
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

            render_pass.set_pipeline(self.raytracer.render_pipeline()); // TODO: rendering structs take control of their own swap chain and are interacted with through a RenderStruct trait
            render_pass.set_bind_group(0, self.raytracer.world_bind_group(), &[]);
            render_pass.set_bind_group(1, self.raytracer.uniform_bind_group(), &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..VERTICES.len() as u32, 0..1);
        }

        // submit will accept anything that implements IntoIter
        context.queue.submit(std::iter::once(encoder.finish()));

        self.raytracer.frame_complete();

        Ok(())
    }

    pub fn size(&self) -> PhysicalSize<u32> {
        self.size
    }
}
