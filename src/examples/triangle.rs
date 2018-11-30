extern crate notcraft_graphics as graphics;

use graphics::{Buffer, Context};

fn main() {
    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new()
        .with_title("Hello, world!")
        .with_dimensions(LogicalSize::new(1024.0, 768.0));
    let glutin_context = glutin::ContextBuilder::new().with_vsync(true);
    let window = glutin::GlWindow::new(window, glutin_context, &events_loop).unwrap();

    let ctx = Context::load(|symbol| window.get_proc_address(symbol));

    let positions = [[-0.5, -0.5, 0.0], [0.0, 0.5, 0.0], [0.5, -0.5, 0.0]];
    let colors = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

    let pos_buffer = BufferBuilder::new()
        .with_usage(UsageType::StaticDraw)
        .with_data(&positions[..])
        .build(&ctx);

    let col_buffer = BufferBuilder::new()
        .with_usage(UsageType::StaticDraw)
        .with_data(&colors[..])
        .build(&ctx);

    let surface = DefaultFramebuffer;

    let program = ProgramBuilder::new()
        .with_vertex_source("...")
        .with_fragment_source("...")
        .build()
        .unwrap();

    ctx.draw_arrays(
        PrimitiveType::Triangles,
        &program,
        cons![&pos_buffer, &col_buffer],
    );
}
