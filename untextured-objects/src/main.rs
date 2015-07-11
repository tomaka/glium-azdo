#[macro_use]
extern crate glium;
extern crate cgmath;

use glium::{DisplayBuild, Surface};
use glium::glutin;
use cgmath::{EuclideanVector, Vector, Point};

#[derive(Copy, Clone)]
struct Vertex {
    position: (f32, f32, f32),
    color: (f32, f32, f32),
}
implement_vertex!(Vertex, position, color);

#[derive(Copy, Clone)]
struct DrawId {
    draw_id: u32,
}
implement_vertex!(DrawId, draw_id);

struct Transform {
    transforms: [[[f32; 4]; 4]],
}
implement_buffer_content!(Transform);
implement_uniform_block!(Transform, transforms);

const OBJECTS_X: usize = 64;
const OBJECTS_Y: usize = 64;
const OBJECTS_Z: usize = 64;
const OBJECTS_COUNT: usize = OBJECTS_X * OBJECTS_Y * OBJECTS_Z;

fn main() {
    let display = glutin::WindowBuilder::new()
                    .with_depth_buffer(24)
                    .build_glium()
                    .unwrap();

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

    let drawids_buffer = glium::VertexBuffer::new(&display,
        (0 .. OBJECTS_COUNT).map(|c| DrawId { draw_id: c as u32 }).collect::<Vec<_>>()
    );

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

    let program = program!(&display,
        410 => {
            vertex: include_str!("../cubes_gl_buffer_storage_vs.glsl"),
            fragment: include_str!("../cubes_gl_buffer_storage_fs.glsl"),
        }
    ).unwrap();

    let mut commands = glium::index::DrawCommandsIndicesBuffer::
                                  empty_dynamic_if_supported(&display, OBJECTS_COUNT * 3).unwrap();
    let mut transform_buffer = glium::uniforms::UniformBuffer::<Transform>::empty_unsized_if_supported(&display,
                                                                  16 * 4 * OBJECTS_COUNT).unwrap();

    let mut iteration = 0;
    let mut buf_num = 0;

    loop {
        // writing the commands
        {
            let commands = commands.slice_mut(buf_num * OBJECTS_COUNT .. (buf_num + 1) * OBJECTS_COUNT).unwrap();
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

        // writing the transforms
        {
            let angle = iteration as f32 * 0.01;
            iteration += 1;
            if angle > 2.0 * 3.141592 {
                iteration = 0;
            }

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

                        mat[0][0] = 2.0 * (x as isize - OBJECTS_X as isize) as f32;
                        mat[3][1] = 2.0 * (y as isize - OBJECTS_Y as isize) as f32;
                        mat[3][2] = 2.0 * (z as isize - OBJECTS_Z as isize) as f32;

                        transforms.transforms[index] = mat;
                        index += 1;
                    }
                }
            }
        }

        let commands = commands.slice_mut(buf_num * OBJECTS_COUNT .. (buf_num + 1) * OBJECTS_COUNT).unwrap();
        let indices = glium::index::IndicesSource::MultidrawElement {
            commands: commands.as_slice_any(),
            indices: index_buffer.as_slice_any(),
            data_type: index_buffer.get_indices_type(),
            primitives: index_buffer.get_primitives_type(),
        };

        let (w, h) = display.get_framebuffer_dimensions();
        let persp = cgmath::perspective(cgmath::deg(45.0), w as f32 / h as f32, 0.1, 10000.0);
        let dir = cgmath::Vector3::new(-0.5, -1.0, 1.0).normalize();
        let at = cgmath::Vector3::new(0.0, 0.0, 0.0);
        let eye = at - dir.mul_s(250.0);
        let view = cgmath::Matrix4::look_at(&cgmath::Point3::from_vec(&eye),
                                            &cgmath::Point3::from_vec(&at),
                                            &cgmath::Vector3::new(0.0, 0.0, 1.0));
        let pos = cgmath::Matrix4::from_translation(&eye.mul_s(-1.0));
        let matrix = persp * view * pos;

        let params = glium::DrawParameters {
            backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockWise,
            depth_test: glium::draw_parameters::DepthTest::IfLess,
            depth_write: true,
            .. Default::default()
        };

        let mut target = display.draw();
        target.clear_color_and_depth((0.0, 0.01, 0.0, 1.0), 1.0);
        target.draw((&vertex_buffer, drawids_buffer.per_instance_if_supported().unwrap()),
                    indices, &program, &uniform! { CB0: &transform_buffer, ViewProjection: matrix },
                    &params).unwrap();
        target.finish().unwrap();

        buf_num += 1;
        if buf_num >= 3 { buf_num = 0; }

        for event in display.poll_events() {
            match event {
                glutin::Event::Closed => return,
                _ => ()
            }
        }
    }
}
