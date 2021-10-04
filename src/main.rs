#![deny(unused_must_use)] // unused futures show as errors

use std::sync::Arc;

use futures::lock::Mutex;
use game::{GameLogic, World};
use renderer::RenderContext;
use winit::{event::*, event_loop::ControlFlow};

mod game;
mod renderer;

fn main() {
    env_logger::init();
    let event_loop = winit::event_loop::EventLoop::new();

    let context = futures::executor::block_on(RenderContext::new(&event_loop));

    let world = Arc::new(Mutex::new(World::new(&context)));

    let mut game_logic = futures::executor::block_on(GameLogic::new(world.clone()));

    let mut renderer =
        futures::executor::block_on(renderer::Renderer::new(&context, world.clone()));

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == context.window.id() => {
                if !renderer.input(event) {
                    // UPDATED!
                    match event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::KeyboardInput { input, .. } => game_logic.input_event(input),
                        WindowEvent::Resized(physical_size) => {
                            renderer.resize(&context, *physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            renderer.resize(&context, **new_inner_size);
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(_) => {
                renderer.update();
                match futures::executor::block_on(renderer.render(&context)) {
                    Ok(_) => {}
                    // Recreate the swap_chain if lost
                    Err(wgpu::SurfaceError::Lost) => renderer.resize(&context, renderer.size()),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                };
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                context.window.request_redraw();
            }
            _ => {}
        };
    });
}
