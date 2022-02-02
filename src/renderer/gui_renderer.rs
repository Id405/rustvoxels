use std::sync::Arc;

use futures::lock::Mutex;

use crate::{
    game::World,
    ui::{Ui, UiState},
};

use super::RenderContext;

pub struct Gui {
    imgui_renderer: imgui_wgpu::Renderer,
    world: Arc<Mutex<World>>,
    ui_state: UiState,
}

impl Gui {
    pub async fn new(
        context: &RenderContext,
        world: Arc<Mutex<World>>,
        surface_config: &wgpu::SurfaceConfiguration,
    ) -> Self {
        let imgui_renderer_config = imgui_wgpu::RendererConfig {
            texture_format: surface_config.format,
            ..Default::default()
        };

        let ui_state = UiState::new(world.clone()).await;

        Self {
            imgui_renderer: imgui_wgpu::Renderer::new(
                &mut world.lock().await.ui.as_mut().unwrap().context,
                &context.device,
                &context.queue,
                imgui_renderer_config,
            ),
            world: world.clone(),
            ui_state,
        }
    }

    pub async fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        context: &RenderContext,
        render_texture_view: &wgpu::TextureView,
    ) {
        self.ui_state.update().await;
        Ui::render_frame(
            self.world.clone(),
            &mut self.ui_state,
            encoder,
            context,
            render_texture_view,
            &mut self.imgui_renderer,
        )
        .await;
    }
}
