#[macro_use]
extern crate glium;

use glium::{DisplayBuild, Surface};
use glium::glutin;

#[derive(Copy, Clone)]
struct Vertex {
    position: (f32, f32),
}

const PARTICLE_COUNT_X: usize = 500;
const PARTICLE_COUNT_Y: usize = 320;

fn main() {
    let display = glutin::WindowBuilder::new()
                    .build_glium()
                    .unwrap();


    implement_vertex!(Vertex, position);


    const VERTICES: usize = PARTICLE_COUNT_X * PARTICLE_COUNT_Y * 6;

    let mut vertex_buffer = glium::VertexBuffer::empty_dynamic(&display, VERTICES * 3);

    #[derive(Copy, Clone)]
    struct UniformData {
        viewport: (f32, f32),
    }

    implement_uniform_block!(UniformData, viewport);

    let uniforms = glium::uniforms::UniformBuffer::new_if_supported(&display,
                                                    UniformData { viewport: (0.0, 0.0) }).unwrap();

    let program = program!(&display,
        410 => {
            vertex: include_str!("../streaming_vb_gl_vs.glsl"),
            fragment: include_str!("../streaming_vb_gl_fs.glsl"),
        }
    ).unwrap();

    let mut iteration = 0;
    let mut buf_num = 0;

    loop {
        upload(&mut vertex_buffer.slice_mut(buf_num * VERTICES .. (buf_num + 1) * VERTICES)
                                 .unwrap().map_write(), iteration);
        iteration += 1;

        let mut target = display.draw();

        let dimensions = target.get_dimensions();
        uniforms.write(&[UniformData { viewport: (2.0 / dimensions.0 as f32, -2.0 / dimensions.1 as f32) }]);

        target.clear_color(0.3, 0.0, 0.3, 1.0);
        target.draw(vertex_buffer.slice(buf_num * VERTICES .. (buf_num + 1) * VERTICES).unwrap(),
                    &glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList),
                    &program, &uniform! { CB0: &uniforms }, &Default::default()).unwrap();
        target.finish().unwrap();

        buf_num += 1;
        if buf_num >= 3 { buf_num = 1; }

        for event in display.poll_events() {
            match event {
                glutin::Event::Closed => return,
                _ => ()
            }
        }
    }
}

fn upload(output: &mut glium::buffer::WriteMapping<Vertex>, iteration: u32) {
    const SPACING: f32 = 1.0;
    const W: f32 = 1.0;
    const H: f32 = 1.0;

    const MARCH_PIXELS_X: u32 = 24;
    const MARCH_PIXELS_Y: u32 = 128;

    let offset_x = (iteration % MARCH_PIXELS_X) as f32 * W;
    let offset_y = ((iteration / MARCH_PIXELS_X) % MARCH_PIXELS_Y) as f32 * H;

    let mut address = 0;

    for y_pos in (0 .. PARTICLE_COUNT_Y) {
        let y = SPACING as f32 + y_pos as f32 * (SPACING + H);

        for x_pos in (0 .. PARTICLE_COUNT_X) {
            let x = SPACING as f32 + x_pos as f32 * (SPACING + W);

            output.set(address + 0,
                       Vertex { position: (x + offset_x + 0.0, y + offset_y + 0.0) });
            output.set(address + 1,
                       Vertex { position: (x + offset_x + W, y + offset_y + 0.0) });
            output.set(address + 2,
                       Vertex { position: (x + offset_x + 0.0, y + offset_y + H) });
            output.set(address + 3,
                       Vertex { position: (x + offset_x + W, y + offset_y + 0.0) });
            output.set(address + 4,
                       Vertex { position: (x + offset_x + 0.0, y + offset_y + H) });
            output.set(address + 5,
                       Vertex { position: (x + offset_x + W, y + offset_y + H) });

            address += 6;
        }
    }
}
