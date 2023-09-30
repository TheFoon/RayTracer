use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod renderer;
mod fps_counter;
mod gui_app;
use renderer::Renderer;

use wgpu;

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("GPU Ray Tracer")
        .with_inner_size(winit::dpi::LogicalSize::new(800, 600))
        .build(&event_loop)
        .unwrap();

    let mut renderer = pollster::block_on(Renderer::new(window));

    let start_time = std::time::Instant::now();
    let mut last_time = std::time::Instant::now();

    event_loop.run(move |event, _, control_flow| {
        renderer.platform.handle_event(&event);

        match event {
            Event::WindowEvent {
                ref event,
                window_id
            } if window_id == renderer.window.id() => {
                match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        renderer.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        renderer.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }
            Event::RedrawRequested(_) => {
                renderer.platform.update_time(start_time.elapsed().as_secs_f64());//TODO: maybe this can be moved to renderer.update()?
                match renderer.render() {
                    Ok(_) => {}
                    // Recreate the swap_chain if lost
                    Err(wgpu::SurfaceError::Lost) => {renderer.resize(renderer.size)}
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => {*control_flow = ControlFlow::Exit}
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Event::MainEventsCleared => {
                {
                    let delta_time = last_time.elapsed().as_secs_f32();
                    last_time = std::time::Instant::now();
                    renderer.update(delta_time);
                }
                renderer.window.request_redraw();
            }
            _ => {}
        }
    });
}

