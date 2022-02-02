#![deny(unused_must_use)] // unused futures show as errors

use std::sync::Arc;

use futures::lock::Mutex;
use game::logic::InputEvent;
use game::{GameLogic, World};
use renderer::RenderContext;
use ui::Ui;
use winit::{event::*, event_loop::ControlFlow};

mod config;
mod game;
mod renderer;
mod ui;

fn main() {
    env_logger::init();
    let event_loop = winit::event_loop::EventLoop::new();

    let mut context = futures::executor::block_on(RenderContext::new(&event_loop));

    let world = Arc::new(Mutex::new(World::new(&context)));

    let mut game_logic = futures::executor::block_on(GameLogic::new(world.clone()));

    let mut renderer =
        futures::executor::block_on(renderer::Renderer::new(&context, world.clone()));

    let mut last_frame = std::time::Instant::now();

    let mut mouse_grab = false;

    event_loop.run(move |event, _, control_flow| {
        futures::executor::block_on(Ui::handle_event(world.clone(), &event, &context));

        match &event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if *window_id == context.window.id() => {
                if !renderer.input(event) {
                    // UPDATED!
                    match &event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::KeyboardInput { input, .. } => {
                            futures::executor::block_on(
                                game_logic.input_event(&InputEvent::Keyboard(*input)),
                            );
                            match input.state {
                                ElementState::Pressed => match input.virtual_keycode {
                                    Some(keycode) => match keycode {
                                        VirtualKeyCode::Escape => mouse_grab = !mouse_grab,
                                        _ => (),
                                    },
                                    None => (),
                                },
                                ElementState::Released => (),
                            }
                        }
                        WindowEvent::Resized(physical_size) => {
                            futures::executor::block_on(renderer.resize(&context, *physical_size));
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            futures::executor::block_on(
                                renderer.resize(&context, **new_inner_size),
                            );
                        }
                        WindowEvent::Focused(focus) => {
                            if *focus == false {
                                mouse_grab = false;
                            }
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(_) => {
                let last_render_start = std::time::Instant::now();
                renderer.update();
                match futures::executor::block_on(renderer.render(&context)) {
                    Ok(_) => {}
                    // Recreate the swap_chain if lost
                    Err(wgpu::SurfaceError::Lost) => {
                        futures::executor::block_on(renderer.resize(&context, renderer.size()))
                    }
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                };
                let render_delta = std::time::Instant::now() - last_render_start;
                context.render_time = render_delta.as_secs_f32() * 1000.0;
            }
            Event::MainEventsCleared => {
                let delta = std::time::Instant::now() - last_frame;
                futures::executor::block_on(game_logic.update(delta.as_secs_f32()));
                context.window.set_cursor_visible(!mouse_grab);
                if let Err(why) = context.window.set_cursor_grab(mouse_grab) {
                    eprintln!("{:?}", why);
                } // TODO; rework to have cursor grabbing dictated by gamelogic
                last_frame = std::time::Instant::now();
                context.frame_count += 1;
                context.frame_time = delta.as_secs_f32() * 1000.0;
                context.window.request_redraw();
            }
            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::MouseMotion { delta } => {
                    if mouse_grab {
                        futures::executor::block_on(
                            game_logic.input_event(&InputEvent::Mouse(*delta)),
                        )
                    }
                }
                DeviceEvent::Button { button, state } => {
                    futures::executor::block_on(Ui::handle_click(
                        world.clone(),
                        &(button, state),
                        &context,
                    ));
                }
                _ => (),
            },
            _ => {}
        };
    });
}
