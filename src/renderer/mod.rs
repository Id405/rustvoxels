use std::{cell::RefCell, rc::Rc, sync::Arc};

use futures::lock::Mutex;
use wgpu::{util::DeviceExt, CommandEncoder};
use winit::{dpi::PhysicalSize, window};

use crate::game::World;

pub use glsl_loader::ShaderBundle;
pub use render_context::RenderContext;

use self::texture_atlas::TextureAtlas;

mod denoiser;
mod glsl_loader;
mod gui_renderer;
mod raytracer;
mod render_context;
mod texture_atlas;
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
    denoiser: denoiser::Denoiser,
    texture_renderer: texture_renderer::TextureRenderer,
    gui: gui_renderer::Gui,
    world: Arc<Mutex<World>>,
    atlas: Rc<RefCell<TextureAtlas>>,
}

impl Renderer {
    // Creating some of the wgpu types requires async code
    pub async fn new(context: &RenderContext, world: Arc<Mutex<World>>) -> Renderer {
        let size = context.window.inner_size();

        let atlas = Rc::new(RefCell::new(TextureAtlas::new(&context)));

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

        let raytracer =
            raytracer::Raytracer::new(context, world.clone(), &surface_config, atlas.clone()).await; // the raytracer struct should hold its own swapchain in the future, or whatever the compute shader equivilant is

        let denoiser = denoiser::Denoiser::new(context, world.clone(), atlas.clone()).await;

        let gui = gui_renderer::Gui::new(context, world.clone(), &surface_config).await;

        let texture_renderer = texture_renderer::TextureRenderer::new(
            context,
            &surface_config,
            &atlas
                .borrow()
                .get_view("denoiser_attachment_color", context)
                .unwrap(),
        )
        .await;

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
            denoiser,
            atlas,
            gui,
        }
    }

    // ALl of this texture management is just terrible and annoying and needs to be restructured.
    // Renderer structs shouldnt own their textures, and neither should the renderer. There should
    // be a texture atlas struct holding Arc<Texture> (or maybe Arc<Mutex<Texture>> if neccessary?).
    pub async fn resize(
        &mut self,
        context: &RenderContext,
        new_size: winit::dpi::PhysicalSize<u32>,
    ) {
        let mut atlas = self.atlas.borrow_mut();

        self.size = new_size;
        self.surface_config.width = new_size.width;
        self.surface_config.height = new_size.height;
        context
            .surface
            .configure(&context.device, &self.surface_config);

        self.world.lock().await.player.as_mut().unwrap().camera.size = new_size.clone();

        atlas.resize(context);

        self.raytracer.resize(new_size, &context);
        self.denoiser.resize(new_size, &context);
        self.texture_renderer.resize(
            new_size,
            &atlas
                .get_view("denoiser_attachment_color", context)
                .unwrap(),
            &context,
        );
    }

    pub fn input(&mut self, event: &winit::event::WindowEvent) -> bool {
        // This function should only be used for accepting debug commands for the renderer
        false
    }

    pub fn update(&mut self) {
        // remove
    }

    pub async fn render(&mut self, context: &RenderContext) -> Result<(), wgpu::SurfaceError> {
        let frame = context.surface.get_current_texture()?;

        let mut encoder = context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        self.raytracer
            .render(&mut encoder, &context, &self.vertex_buffer)
            .await;

        self.denoiser
            .render(&mut encoder, &context, &self.vertex_buffer)
            .await;

        self.texture_renderer // This could be accomplished with just a copy command lol TODO
            .render(
                &mut encoder,
                &context,
                &self.vertex_buffer,
                &self
                    .atlas
                    .borrow()
                    .get_view("denoiser_attachment_color", context)
                    .unwrap(),
                &frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default()),
            )
            .await;

        self.gui
            .render(
                &mut encoder,
                &context,
                &frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default()),
            )
            .await;

        // submit will accept anything that implements IntoIter
        context.queue.submit(std::iter::once(encoder.finish()));

        frame.present();

        self.world
            .lock()
            .await
            .player
            .as_mut()
            .unwrap()
            .camera
            .frame_count += 1;

        Ok(())
    }

    pub fn size(&self) -> PhysicalSize<u32> {
        self.size
    }
}
