#[macro_use]
extern crate glium;
extern crate cgmath;

use glium::{DisplayBuild, Surface};
use glium::glutin;
use cgmath::{EuclideanVector, Vector, Point};

/// Represents a vertex in the vertex buffer.
#[derive(Copy, Clone)]
struct Vertex {
    position: (f32, f32, f32),
    color: (f32, f32, f32),
}
implement_vertex!(Vertex, position, color);

/// We emulate `GL_ARB_shader_draw_parameters` by storing draw IDs in a buffer.
#[derive(Copy, Clone)]
struct DrawId {
    draw_id: u32,
}
implement_vertex!(DrawId, draw_id);

/// Contains the list of transforms to apply to individual cubes.
struct Transform {
    transforms: [[[f32; 4]; 4]],
}
implement_buffer_content!(Transform);
implement_uniform_block!(Transform, transforms);

// You can modify these three values.
const OBJECTS_X: usize = 64;
const OBJECTS_Y: usize = 64;
const OBJECTS_Z: usize = 64;

// But don't touch this one.
const OBJECTS_COUNT: usize = OBJECTS_X * OBJECTS_Y * OBJECTS_Z;

fn main() {
    let display = glutin::WindowBuilder::new()
                    .with_depth_buffer(24)
                    .build_glium()
                    .unwrap();

    // building a vertex buffer that contains the cube
    let vertex_buffer = glium::VertexBuffer::new(&display, &[
        Vertex { position: (-0.5,  0.5, -0.5), color: (0.0,  1.0,  0.0) },
        Vertex { position: ( 0.5,  0.5, -0.5), color: (1.0,  1.0,  0.0) },
        Vertex { position: ( 0.5,  0.5,  0.5), color: (1.0,  1.0,  1.0) },
        Vertex { position: (-0.5,  0.5,  0.5), color: (0.0,  1.0,  1.0) },
        Vertex { position: (-0.5, -0.5,  0.5), color: (0.0,  0.0,  1.0) },
        Vertex { position: ( 0.5, -0.5,  0.5), color: (1.0,  0.0,  1.0) },
        Vertex { position: ( 0.5, -0.5, -0.5), color: (1.0,  0.0,  0.0) },
        Vertex { position: (-0.5, -0.5, -0.5), color: (0.0,  0.0,  0.0) },
    ]);

    // matches `vertex_buffer`
    let index_buffer = glium::index::IndexBuffer::new(&display,
        glium::index::PrimitiveType::TrianglesList,
        &[
            0, 1, 2, 0, 2, 3,
            4, 5, 6, 4, 6, 7,
            3, 2, 5, 3, 5, 4,
            2, 1, 6, 2, 6, 5,
            1, 7, 6, 1, 0, 7,
            0, 3, 4, 0, 4, 7u16
        ][..]
    );

    // storing each number from `0` to `OBJECTS_COUNT` linearly in a buffer
    let drawids_buffer = glium::VertexBuffer::new(&display,
        (0 .. OBJECTS_COUNT).map(|c| DrawId { draw_id: c as u32 }).collect::<Vec<_>>()
    );

    // compiling the program ; if `.unwrap()` panicks, then your hardware is probably not good
    // enough
    let program = program!(&display,
        410 => {
            vertex: include_str!("../cubes_gl_buffer_storage_vs.glsl"),
            fragment: include_str!("../cubes_gl_buffer_storage_fs.glsl"),
        }
    ).unwrap();

    // creating the buffer that will hold our draw commands
    // we have one command per cube, and we mulitply by 3 to use triple buffering
    let mut commands = glium::index::DrawCommandsIndicesBuffer::
                                  empty_dynamic_if_supported(&display, OBJECTS_COUNT * 3).unwrap();

    // storing all the transforms ; one transform per cube
    let mut transform_buffer =
                  glium::uniforms::UniformBuffer::<Transform>::empty_unsized_if_supported(&display,
                                                                  16 * 4 * OBJECTS_COUNT).unwrap();

    // looping forever with a counter
    for iteration in 0u32.. {
        // we are doing triple buffering
        // while part X of the buffer is used to draw, part X+1 is being uploaded and part X+2
        // is waiting
        let buf_num = (iteration as usize) % 3;

        // writing the commands to the commands buffer
        {
            let commands = commands.slice_mut(buf_num * OBJECTS_COUNT ..
                                              (buf_num + 1) * OBJECTS_COUNT).unwrap();
            let len = commands.len();
            let mut commands = commands.map_write();

            for i in 0 .. len {
                commands.set(i, glium::index::DrawCommandIndices {
                    count: index_buffer.len() as u32,
                    instance_count: 1,
                    first_index: 0,
                    base_vertex: 0,
                    base_instance: i as u32,
                });
            }
        }

        // writing the transform matrices
        {
            let angle = iteration as f32 * 0.01;

            let mut transforms = transform_buffer.map();
            let mut index = 0;

            for x in (0 .. OBJECTS_X) {
                for y in (0 .. OBJECTS_Y) {
                    for z in (0 .. OBJECTS_Z) {
                        let s = angle.sin();
                        let c = angle.cos();

                        let mut mat = [
                            [c,  -s,   0.0, 0.0],
                            [s,   c,   0.0, 0.0],
                            [0.0, 0.0, 1.0, 0.0],
                            [0.0, 0.0, 0.0, 1.0],
                        ];

                        mat[3][0] = 2.0 * x as f32 - OBJECTS_X as f32;
                        mat[3][1] = 2.0 * y as f32 - OBJECTS_Y as f32;
                        mat[3][2] = 2.0 * z as f32 - OBJECTS_Z as f32;

                        transforms.transforms[index] = mat;
                        index += 1;
                    }
                }
            }
        }

        // building the source of indices
        // ideally you shouldn't build a `IndicesSource` directly, but a limitation in glium
        // currently makes this mandatory
        let commands = commands.slice_mut(buf_num * OBJECTS_COUNT ..
                                          (buf_num + 1) * OBJECTS_COUNT).unwrap();
        let indices = glium::index::IndicesSource::MultidrawElement {
            commands: commands.as_slice_any(),
            indices: index_buffer.as_slice_any(),
            data_type: index_buffer.get_indices_type(),
            primitives: index_buffer.get_primitives_type(),
        };

        // the matrix of the camera to use when drawing the cubes
        let camera = build_camera(display.get_framebuffer_dimensions());

        // the parameters to use when drawing
        // nothing fancy, just depth testing and backface culling
        let params = glium::DrawParameters {
            backface_culling: glium::draw_parameters::BackfaceCullingMode::CullCounterClockWise,
            depth_test: glium::draw_parameters::DepthTest::IfLess,
            depth_write: true,
            .. Default::default()
        };

        // drawing a frame
        let mut target = display.draw();
        target.clear_color_and_depth((0.0, 0.01, 0.0, 1.0), 1.0);
        target.draw((&vertex_buffer, drawids_buffer.per_instance_if_supported().unwrap()),
                    indices, &program, &uniform! { CB0: &transform_buffer, ViewProjection: camera },
                    &params).unwrap();
        target.finish().unwrap();

        // handling events so that we can quit if the window is closed by the user
        for event in display.poll_events() {
            match event {
                glutin::Event::Closed => return,
                _ => ()
            }
        }
    }
}

fn build_camera((w, h): (u32, u32)) -> cgmath::Matrix4<f32> {
    let persp = cgmath::perspective(cgmath::deg(45.0), w as f32 / h as f32, 0.1, 10000.0);

    let dir = cgmath::Vector3::new(-0.5, -1.0, 1.0).normalize();
    let at = cgmath::Vector3::new(0.0, 0.0, 0.0);
    let eye = at - dir.mul_s(250.0);

    let view = cgmath::Matrix4::look_at(&cgmath::Point3::from_vec(&eye),
                                        &cgmath::Point3::from_vec(&at),
                                        &cgmath::Vector3::new(0.0, 0.0, 1.0));

    persp * view
}
