use glium::glutin;

pub fn create_display(event_loop: &glutin::event_loop::EventLoop<()>) -> glium::Display {
    let window_builder = glutin::window::WindowBuilder::new()
        .with_resizable(true)
        .with_maximized(true)
        // .with_inner_size(glutin::dpi::LogicalSize {
        //     width: 800.0,
        //     height: 600.0,
        // })
        .with_title("miditits");

    let context_builder = glutin::ContextBuilder::new()
        .with_depth_buffer(0)
        .with_srgb(true)
        .with_stencil_buffer(0)
        .with_vsync(true);

    glium::Display::new(window_builder, context_builder, event_loop).unwrap()
}
