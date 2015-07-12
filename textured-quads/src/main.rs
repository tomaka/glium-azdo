#[macro_use]
extern crate glium;
extern crate cgmath;
extern crate rand;

use glium::{DisplayBuild, Surface};
use glium::glutin;
use cgmath::{EuclideanVector, Vector, Point};

#[derive(Copy, Clone)]
struct Vertex {
    position: (f32, f32, f32),
    tex_coords: (f32, f32),
}
implement_vertex!(Vertex, position, tex_coords);

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

struct Textures<'a> {
    tex_address: [glium::texture::TextureHandle<'a>],
}
implement_buffer_content!(Textures<'a>);
implement_uniform_block!(Textures<'a>, tex_address);

// TODO: putting too many objects crashes the driver
const OBJECTS_X: usize = 20;
const OBJECTS_Y: usize = 20;
const OBJECTS_COUNT: usize = OBJECTS_X * OBJECTS_Y;

fn main() {
    let display = glutin::WindowBuilder::new()
                    .with_depth_buffer(24)
                    .build_glium()
                    .unwrap();

    let vertex_buffer = glium::VertexBuffer::new(&display, &[
        Vertex { position: (-0.5, -0.5, 0.0), tex_coords: (0.0,  0.0) },
        Vertex { position: ( 0.5, -0.5, 0.0), tex_coords: (0.0,  1.0) },
        Vertex { position: ( 0.5,  0.5, 0.0), tex_coords: (1.0,  0.0) },
        Vertex { position: (-0.5,  0.5, 0.0), tex_coords: (1.0,  1.0) },
    ]);

    let index_buffer = glium::index::IndexBuffer::new(&display,
        glium::index::PrimitiveType::TrianglesList,
        &[0, 1, 2, 0, 2, 3u16][..]
    );

    let program = program!(&display,
        410 => {
            vertex: include_str!("../textures_gl_bindless_multidraw_vs.glsl"),
            fragment: include_str!("../textures_gl_bindless_multidraw_fs.glsl"),
        }
    ).unwrap();

    let mut commands = glium::index::DrawCommandsIndicesBuffer::
                                  empty_dynamic_if_supported(&display, OBJECTS_COUNT * 3).unwrap();
    let mut transform_buffer = glium::uniforms::UniformBuffer::<Transform>::empty_unsized_if_supported(&display,
                                                                  16 * 4 * OBJECTS_COUNT).unwrap();

    let textures_storage = (0 .. OBJECTS_COUNT).map(|_| {
        let color1: (f32, f32, f32) = (rand::random(), rand::random(), rand::random());
        let color2: (f32, f32, f32) = (rand::random(), rand::random(), rand::random());
        let texture = glium::texture::Texture2d::new(&display, vec![vec![color1], vec![color2]]);
        texture.resident_if_supported().unwrap()
    }).collect::<Vec<_>>();

    let mut cb1 = glium::uniforms::UniformBuffer::<Textures>::empty_unsized_if_supported(&display, OBJECTS_COUNT * 8).unwrap();
    for (i, element) in cb1.map().tex_address.iter_mut().enumerate() {
        *element = glium::texture::TextureHandle::new(&textures_storage[i], &Default::default());
    }

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
                    base_instance: 0,
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
                    mat[3][2] = 0.0;

                    transforms.transforms[index] = mat;
                    index += 1;
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

        let matrix = build_camera(display.get_framebuffer_dimensions());

        let params = glium::DrawParameters {
            backface_culling: glium::draw_parameters::BackfaceCullingMode::CullCounterClockWise,
            depth_test: glium::draw_parameters::DepthTest::IfLess,
            depth_write: true,
            .. Default::default()
        };

        let mut target = display.draw();
        target.clear_color_and_depth((0.0, 0.0, 0.01, 1.0), 1.0);
        target.draw(&vertex_buffer, indices, &program,
                    &uniform! { CB0: &transform_buffer, CB1: &cb1, ViewProjection: matrix },
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

fn build_camera((w, h): (u32, u32)) -> cgmath::Matrix4<f32> {
    let persp = cgmath::perspective(cgmath::deg(45.0), w as f32 / h as f32, 0.1, 10000.0);

    let dir = cgmath::Vector3::new(0.0, 0.0, 1.0).normalize();
    let at = cgmath::Vector3::new(0.0, 0.0, 0.0);
    let eye = at - dir.mul_s(250.0);

    let view = cgmath::Matrix4::look_at(&cgmath::Point3::from_vec(&eye),
                                        &cgmath::Point3::from_vec(&at),
                                        &cgmath::Vector3::new(0.0, 1.0, 0.0));

    persp * view
}
