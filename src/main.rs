mod app;
mod app_state;
mod display;
mod store;

use egui_glium;
use egui_glium::EguiGlium;
use glium::glutin;

use crate::app::*;
use crate::app_state::*;
use crate::display::create_display;
use crate::store::Store;

use midi;

fn main() {
    // Midi run blocks main thread. its UI component can be brought out of lib.rs
    // or this glu code can go in there.
    {
        match midi::run() {
            Ok(_) => (),
            Err(err) => println!("Error: {}", err)
        }
    }
    let event_loop = glutin::event_loop::EventLoop::with_user_event();
    let display = create_display(&event_loop);

    let mut egui = EguiGlium::new(&display);
    let mut app = App::new();

    // Initialze App State and Store
    let mut store = Store::new(AppState::default());
    store.dispatch(AppEvent::Initialize);

    event_loop.run(move |event, _, control_flow| {
        let mut redraw = || {

            // Main App render
            let response = app.render(&store, &mut egui, &display);

            let needs_repaint = store.is_working();

            *control_flow = if false {
                glutin::event_loop::ControlFlow::Exit
            } else if needs_repaint || matches!(response, AppResponse::Redraw) {
                display.gl_window().window().request_redraw();
                glutin::event_loop::ControlFlow::Poll
            } else {
                glutin::event_loop::ControlFlow::Wait
            };

            {
                use glium::Surface as _;
                let mut target = display.draw();

                let clear_color = egui::Rgba::from_rgb(0.1, 0.3, 0.2);
                target.clear_color(
                    clear_color[0],
                    clear_color[1],
                    clear_color[2],
                    clear_color[3],
                );

                // draw things behind egui here

                egui.paint(&display, &mut target);

                // draw things on top of egui here

                target.finish().unwrap();
            }
        };

        match event {
            glutin::event::Event::RedrawEventsCleared if cfg!(windows) => redraw(),
            glutin::event::Event::RedrawRequested(_) if !cfg!(windows) => redraw(),
            glutin::event::Event::WindowEvent { event, .. } => {
                use glutin::event::WindowEvent;

                if matches!(event, WindowEvent::CloseRequested | WindowEvent::Destroyed) {
                    *control_flow = glium::glutin::event_loop::ControlFlow::Exit;
                }

                // Handle Global Keyboard Events
                match event {
                    glutin::event::WindowEvent::KeyboardInput { input, .. }
                        if input.state == glium::glutin::event::ElementState::Pressed =>
                    {
                        app.handle_key_event(&store, input)
                    }
                    _ => {}
                };

                egui.on_event(&event);

                // TODO: ask egui if the events warrants a repaint instead
                display.gl_window().window().request_redraw();
            }

            _ => (),
        }
    });
}
