use std::sync::Arc;

use futures::lock::Mutex;
use winit::event::ElementState;

use crate::{game::World, renderer::RenderContext, config::ConfigValue};

use imgui::*;

pub struct UiState {
    // TODO make UiState use config values
    samples: i32,
    max_steps: i32,
    reprojection_percent: f32,
    blur_strength: f32,
    move_speed: f32,
    do_lighting: bool,
    world: Arc<Mutex<World>>,
}

impl UiState {
    pub async fn new(world: Arc<Mutex<World>>) -> Self {
        let world_lock = world.lock().await;
        let config = world_lock.config.as_ref().unwrap();

        // todo make work from default config value
        UiState {
            samples: config.get_var("renderer_raytracer_samples").unwrap().as_i32(),
            max_steps: config.get_var("renderer_raytracer_max_steps").unwrap().as_i32(),
            reprojection_percent: config.get_var("renderer_denoiser_reprojection_percent").unwrap().as_f32(),
            blur_strength: config.get_var("renderer_denoiser_edge_avoiding_blur_strength").unwrap().as_f32(),
            move_speed: config.get_var("game_input_movement_speed").unwrap().as_f32(),
            do_lighting: config.get_var("renderer_raytracer_do_lighting").unwrap().as_bool(),
            world: world.clone(),
        }
    }

    pub async fn update(&self) {
        let mut world_lock = self.world.lock().await;
        let config = world_lock.config.as_mut().unwrap();

        // todo make work from default config value
        config.set_var("renderer_raytracer_samples", ConfigValue::I32(self.samples));
        config.set_var("renderer_raytracer_max_steps", ConfigValue::I32(self.max_steps));
        config.set_var("renderer_denoiser_reprojection_percent", ConfigValue::F32(self.reprojection_percent));
        config.set_var("renderer_denoiser_edge_avoiding_blur_strength", ConfigValue::F32(self.blur_strength));
        config.set_var("game_input_movement_speed", ConfigValue::F32(self.move_speed));
        config.set_var("renderer_raytracer_do_lighting", ConfigValue::Bool(self.do_lighting));
    }
}

pub struct Ui {
    pub context: imgui::Context,
    pub platform: imgui_winit_support::WinitPlatform,
}

impl Ui {
    pub fn new(context: &RenderContext) -> Self {
        let hidpi_factor = context.window.scale_factor();

        let mut imgui = imgui::Context::create();
        let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui);
        imgui.set_ini_filename(None);

        let dpi_mode = if let Ok(factor) = std::env::var("IMGUI_EXAMPLE_FORCE_DPI_FACTOR") {
            // Allow forcing of HiDPI factor for debugging purposes
            match factor.parse::<f64>() {
                Ok(f) => imgui_winit_support::HiDpiMode::Locked(f),
                Err(e) => panic!("Invalid scaling factor: {}", e),
            }
        } else {
            imgui_winit_support::HiDpiMode::Default
        };

        platform.attach_window(imgui.io_mut(), &context.window, dpi_mode);

        let font_size = (13.0 * hidpi_factor) as f32;
        imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

        imgui
            .fonts()
            .add_font(&[imgui::FontSource::DefaultFontData {
                config: Some(imgui::FontConfig {
                    oversample_h: 1,
                    pixel_snap_h: true,
                    size_pixels: font_size,
                    ..Default::default()
                }),
            }]);

        Self {
            context: imgui,
            platform,
        }
    }

    pub async fn render_frame(
        world: Arc<Mutex<World>>,
        ui_state: &mut UiState,
        encoder: &mut wgpu::CommandEncoder,
        context: &RenderContext,
        render_texture_view: &wgpu::TextureView,
        imgui_renderer: &mut imgui_wgpu::Renderer,
    ) {
        let mut world_lock = world.lock().await;
        let config = world_lock.config.as_ref().unwrap();
        let gui = world_lock.ui.as_mut().unwrap();
        let ui = gui.context.frame();
        let window = imgui::Window::new("Render Stats");

        window
            .size([500.0, 400.0], imgui::Condition::FirstUseEver)
            .build(&ui, || {
                ui.text(format!("Frame Time: {}", context.frame_time));
                ui.text(format!("Render Time: {}", context.render_time));
                ui.separator();
                ui.text("Raytracer Config");
                Slider::new("Samples", 1, 20).build(&ui, &mut ui_state.samples);
                Slider::new("Max Steps", 1, 250).build(&ui, &mut ui_state.max_steps);
                Slider::new("Reprojection Percent", 0.5f32, 1.0)
                    .build(&ui, &mut ui_state.reprojection_percent);
                Slider::new("Edge Avoiding Blur Strength", 0.0f32, 2.5)
                    .build(&ui, &mut ui_state.blur_strength);
                Slider::new("Move Speed", 0.0f32, 50.0).build(&ui, &mut ui_state.move_speed);

                if ui.radio_button_bool("disable lighting", ui_state.do_lighting) {
                    ui_state.do_lighting = !ui_state.do_lighting;
                }
            });

        gui.platform.prepare_render(&ui, &context.window);

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &render_texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });

        imgui_renderer
            .render(ui.render(), &context.queue, &context.device, &mut rpass)
            .expect("Rendering failed");
    }

    pub async fn handle_event(
        world: Arc<Mutex<World>>,
        event: &winit::event::Event<'_, ()>,
        context: &RenderContext,
    ) {
        let mut world_lock = world.lock().await;
        let ui = world_lock.ui.as_mut().unwrap();
        let platform = &mut ui.platform;

        platform.handle_event(ui.context.io_mut(), &context.window, event);
    }

    pub async fn handle_click(
        world: Arc<Mutex<World>>,
        event: &(&u32, &ElementState),
        context: &RenderContext,
    ) {
        let mut world_lock = world.lock().await;
        let ui = world_lock.ui.as_mut().unwrap();

        let index = match event.0 {
            1 => 0,
            2 => 3,
            3 => 1,
            _ => todo!(),
        };

        ui.context.io_mut().mouse_down[index] = *event.1 == ElementState::Pressed;
    }
}
