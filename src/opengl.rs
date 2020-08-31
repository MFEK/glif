//! Necessary OpenGL boiler plate. See https://github.com/ctrlcctrlv/imgui-skia-example
//! Some of this code is from https://github.com/jazzfool/reclutch/
//! MIT licensed. I changed it quite a bit:
//! See also: https://github.com/ctrlcctrlv/imgui-skia-example

use glium::backend::Facade;
use glium::texture::{MipmapsOption, SrgbFormat, SrgbTexture2d};
use glium::Display as GlDisplay;
use glium::GlObject;
use glium::{IndexBuffer, Program as GlProgram, VertexBuffer};

pub mod imgui;
pub mod skia; // Jazzfool's example makes a 3D cube, while I use Dear Imgui.

#[derive(Copy, Clone)]
pub struct TextureVertex {
    position: [f32; 3],
    tex_coord: [f32; 2],
}

implement_vertex!(TextureVertex, position, tex_coord);

const fn texture_vertex(pos: [i8; 2], tex: [i8; 2]) -> TextureVertex {
    TextureVertex {
        position: [pos[0] as _, pos[1] as _, 0.0],
        tex_coord: [tex[0] as _, tex[1] as _],
    }
}

const QUAD_VERTICES: [TextureVertex; 4] = [
    texture_vertex([-1, -1], [0, 0]),
    texture_vertex([-1, 1], [0, 1]),
    texture_vertex([1, 1], [1, 1]),
    texture_vertex([1, -1], [1, 0]),
];

const QUAD_INDICES: [u32; 6] = [0, 1, 2, 0, 2, 3];

pub fn create_texture(gl_display: &GlDisplay, window_size: (u32, u32)) -> SrgbTexture2d {
    SrgbTexture2d::empty_with_format(
        gl_display,
        SrgbFormat::U8U8U8U8,
        MipmapsOption::NoMipmap,
        window_size.0,
        window_size.1,
    )
    .expect("Failed to create SrgbTexture2d")
}

pub fn quad_vertex_buffer(gl_display: &GlDisplay) -> VertexBuffer<TextureVertex> {
    glium::VertexBuffer::new(gl_display, &QUAD_VERTICES)
        .expect("Failed to initialize quad vertex buffer")
}

pub fn quad_indices(gl_display: &GlDisplay) -> IndexBuffer<u32> {
    glium::IndexBuffer::new(
        gl_display,
        glium::index::PrimitiveType::TrianglesList,
        &QUAD_INDICES,
    )
    .expect("Failed to initialize quad index buffer")
}

const QUAD_VERTEX_SHADER_SRC: &str = r#"
    #version 140

    in vec3 position;
    in vec2 tex_coord;

    out vec2 frag_tex_coord;

    void main() {
        frag_tex_coord = tex_coord;
        gl_Position = vec4(position, 1.0);
    }
"#;

const QUAD_FRAGMENT_SHADER_SRC: &str = r#"
    #version 150

    in vec2 frag_tex_coord;
    out vec4 color;

    uniform sampler2D tex;

    void main() {
        color = texture(tex, frag_tex_coord);
    }
"#;

pub fn quad_program(gl_display: &GlDisplay) -> GlProgram {
    GlProgram::from_source(
        gl_display,
        QUAD_VERTEX_SHADER_SRC,
        QUAD_FRAGMENT_SHADER_SRC,
        None,
    )
    .expect("Failed to compile OpenGL quad shader")
}
