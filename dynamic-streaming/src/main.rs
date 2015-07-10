#[macro_use]
extern crate glium;

use glium::{DisplayBuild, Surface};
use glium::glutin;

/// This struct defines the layout of each vertex that we are going to draw.
#[derive(Copy, Clone)]
struct Vertex {
    position: (f32, f32),
}
implement_vertex!(Vertex, position);

/// Describes the layout of the uniform buffer `CB0`.
#[derive(Copy, Clone)]
struct UniformData {
    viewport: (f32, f32),
}
implement_uniform_block!(UniformData, viewport);

// you can change these constants if you want
const PARTICLE_COUNT_X: usize = 500;
const PARTICLE_COUNT_Y: usize = 320;
// but not this one
const VERTICES: usize = PARTICLE_COUNT_X * PARTICLE_COUNT_Y * 6;

fn main() {
    let display = glutin::WindowBuilder::new()
                    .build_glium()
                    .unwrap();

    // we create a vertex buffer whose size is three times the normal size, so that we can
    // upload to one part of the buffer while the other part is in use
    let mut vertex_buffer = glium::VertexBuffer::empty_dynamic(&display, VERTICES * 3);

    // storage for the `CB0` uniform buffer
    // we initialize the content with a dummy value because we are going to write to it at the
    // beginning of each frame
    let cb0 = glium::uniforms::UniformBuffer::new_if_supported(&display,
                                                    UniformData { viewport: (0.0, 0.0) }).unwrap();

    // compiling the program
    // if this panicks, then your hardware is probably not good enough
    let program = program!(&display,
        410 => {
            vertex: include_str!("../streaming_vb_gl_vs.glsl"),
            fragment: include_str!("../streaming_vb_gl_fs.glsl"),
        }
    ).unwrap();

    // looping forever with a counter
    for iteration in 0u32.. {
        // we are doing triple buffering
        // while part X of the buffer is used to draw, part X+1 is being uploaded and part X+2
        // is waiting
        let buf_num = (iteration as usize) % 3;

        // we map the buffer and write the vertex data to it
        // the value of `iteration` determines the values that are written
        upload(&mut vertex_buffer.slice_mut(buf_num * VERTICES .. (buf_num + 1) * VERTICES)
                                 .unwrap().map_write(), iteration);

        // updating the content of `CB0`
        let dimensions = display.get_framebuffer_dimensions();
        cb0.write(&UniformData {
            viewport: (2.0 / dimensions.0 as f32, -2.0 / dimensions.1 as f32)
        });

        // drawing
        //
        // contrary to the original example, we don't draw the quads one by one
        // glium's safety measures would keep track of the status of each individual quad in the
        // buffer, which would add a big overhead
        let mut target = display.draw();
        target.clear_color(0.3, 0.0, 0.3, 1.0);
        target.draw(vertex_buffer.slice(buf_num * VERTICES .. (buf_num + 1) * VERTICES).unwrap(),
                    &glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList),
                    &program, &uniform! { CB0: &cb0 }, &Default::default()).unwrap();
        target.finish().unwrap();

        // handling events so that we quit if the window has been closed by the user
        for event in display.poll_events() {
            match event {
                glutin::Event::Closed => return,
                _ => ()
            }
        }
    }
}

/// Writes the vertex data to the mapping.
fn upload(output: &mut glium::buffer::WriteMapping<[Vertex]>, iteration: u32) {
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
