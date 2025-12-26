use std::mem::{offset_of, size_of, transmute};
use std::ffi::{c_void};
use std::ptr;
use std::os::raw::c_int;

use gl;
use gl::{LINEAR, LINEAR_MIPMAP_LINEAR, REPEAT, TEXTURE_2D, UNSIGNED_BYTE, TEXTURE_MAG_FILTER, TEXTURE_MIN_FILTER, TEXTURE_WRAP_S, TEXTURE_WRAP_T, RGB, TRIANGLES, FLOAT, STATIC_DRAW, ARRAY_BUFFER, ELEMENT_ARRAY_BUFFER, UNSIGNED_SHORT, TEXTURE_CUBE_MAP, TEXTURE_CUBE_MAP_POSITIVE_X, CLAMP_TO_EDGE, TEXTURE_WRAP_R};
use gl::types::{GLenum, GLint, GLsizei, GLsizeiptr, GLuint, GLushort};

use obj;
use obj::{TexturedVertex, Obj};
use stb_image::stb_image;
use nalgebra_glm as glm;

use crate::context::c_string;

// another wrapper structure for GLuint
#[derive(Default)]
pub struct Texture {
    pub id: GLuint
}

impl Texture {
    pub fn new(path: &str) -> Self {
        unsafe {
            // CString needs to be stored in a local variable to avoid pointer dangling
            let texture_src = c_string(path);

            let mut width: c_int = 0;
            let mut height: c_int = 0;
            let mut nr_channels: c_int = 0;

            let image_data = stb_image::stbi_load(
                // as_ptr() returns read-only pointer
                texture_src.as_ptr(),
                &mut width,
                &mut height,
                &mut nr_channels,
                0
            );

            let mut texture: GLuint = 0;

            gl::GenTextures(1, &mut texture);
            gl::BindTexture(TEXTURE_2D, texture);
            // ??? not sure about this segment below, read documentation
            gl::TexParameteri(TEXTURE_2D, TEXTURE_WRAP_S, REPEAT as GLint);
            gl::TexParameteri(TEXTURE_2D, TEXTURE_WRAP_T, REPEAT as GLint);
            gl::TexParameteri(TEXTURE_2D, TEXTURE_MIN_FILTER, LINEAR_MIPMAP_LINEAR as GLint);
            gl::TexParameteri(TEXTURE_2D, TEXTURE_MAG_FILTER, LINEAR as GLint);

            gl::TexImage2D(
                TEXTURE_2D, 0,
                RGB as GLint,
                width, height, 0,
                RGB, UNSIGNED_BYTE,
                image_data as *const c_void
            );

            gl::GenerateMipmap(TEXTURE_2D);

            stb_image::stbi_image_free(image_data as *mut c_void);

            Texture {
                id: texture
            }
        }
    }

    pub fn new_cubemap(paths: &[&str]) -> Self {
        unsafe {
            let mut texture: GLuint = 0;
            gl::GenTextures(1, &mut texture);
            gl::BindTexture(TEXTURE_CUBE_MAP, texture);

            let texture_sources = paths.iter().map(|p| c_string(p)).collect::<Vec<_>>();

            for (i, source) in texture_sources.iter().enumerate() {
                let mut width: c_int = 0;
                let mut height: c_int = 0;
                let mut nr_channels: c_int = 0;

                let image_data = stb_image::stbi_load(
                    source.as_ptr(),
                    &mut width,
                    &mut height,
                    &mut nr_channels,
                    0
                );

                gl::TexImage2D(
                    TEXTURE_CUBE_MAP_POSITIVE_X + i as GLenum,
                    0,
                    RGB as GLint,
                    width, height, 0,
                    RGB, UNSIGNED_BYTE,
                    image_data as *const c_void
                );

                stb_image::stbi_image_free(image_data as *mut c_void);
            }

            gl::TexParameteri(TEXTURE_CUBE_MAP, TEXTURE_MIN_FILTER, LINEAR as GLint);
            gl::TexParameteri(TEXTURE_CUBE_MAP, TEXTURE_MAG_FILTER, LINEAR as GLint);
            gl::TexParameteri(TEXTURE_CUBE_MAP, TEXTURE_WRAP_S, CLAMP_TO_EDGE as GLint);
            gl::TexParameteri(TEXTURE_CUBE_MAP, TEXTURE_WRAP_T, CLAMP_TO_EDGE as GLint);
            gl::TexParameteri(TEXTURE_CUBE_MAP, TEXTURE_WRAP_R, CLAMP_TO_EDGE as GLint);

            Texture {
                id: texture
            }
        }
    }
}

pub struct Mesh<V = TexturedVertex, I = u16> {
    #[allow(dead_code)]
    pub vertices: Vec<V>,
    pub indices: Vec<I>,
    pub vao: GLuint,
    // single mesh can possibly have more than one texture
    pub texture: Texture,
    #[allow(dead_code)] 
    vbo: GLuint,
    #[allow(dead_code)] 
    ebo: GLuint,
    #[allow(dead_code)] 
    instance_vbo: GLuint,
}

impl<V, I> Mesh<V, I> {
    pub fn new(model: Obj<V, I>, texture: Texture, instance_offsets: &[glm::Vec3]) -> Self {
        let mut vao: GLuint = 0;
        let mut vbo: GLuint = 0;
        let mut ebo: GLuint = 0;
        let mut instance_vbo: GLuint = 0;

        unsafe {
            // generating array & buffer objects
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::GenBuffers(1, &mut ebo);
            gl::GenBuffers(1, &mut instance_vbo);

            gl::BindVertexArray(vao);

            // setting up vertex buffer
            gl::BindBuffer(ARRAY_BUFFER, vbo);
            gl::BufferData(
                ARRAY_BUFFER,
                (model.vertices.len() * size_of::<TexturedVertex>()) as GLsizeiptr,
                transmute(&model.vertices[0]),
                STATIC_DRAW // this might change depending on use case of buffer object
            );

            // setting up index buffer
            gl::BindBuffer(ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(
                ELEMENT_ARRAY_BUFFER,
                (model.indices.len() * size_of::<GLushort>()) as GLsizeiptr,
                transmute(&model.indices[0]),
                STATIC_DRAW // and this one probably shouldn't change, cause who tf needs to modify indices
            );

            // specifying vertex data layout

            // Vertex Attributes (Position, Normal, TexCoords)
            let stride = size_of::<TexturedVertex>() as GLsizei;
            // Position
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 3, FLOAT, gl::FALSE, stride, 0 as *const c_void);
            // Normal
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(1, 3, FLOAT, gl::FALSE, stride, offset_of!(TexturedVertex, normal) as *const c_void);
            // Texture Coordinates
            gl::EnableVertexAttribArray(2);
            // TexCoords are Vec2 in obj::TexturedVertex apparently, not Vec3
            gl::VertexAttribPointer(2, 2, FLOAT, gl::FALSE, stride, offset_of!(TexturedVertex, texture) as *const c_void);

            // Setup instance VBO
            gl::BindBuffer(ARRAY_BUFFER, instance_vbo);
            gl::BufferData(
                ARRAY_BUFFER,
                (instance_offsets.len() * size_of::<glm::Vec3>()) as GLsizeiptr,
                transmute(&instance_offsets[0]),
                STATIC_DRAW // Or DYNAMIC_DRAW if offsets change often
            );

            // Instance Attribute (Offset) - using location 3
            gl::EnableVertexAttribArray(3);
            gl::VertexAttribPointer(
                3, // attribute location 3
                3, // size vec3
                FLOAT, // type
                gl::FALSE, // normalized?
                size_of::<glm::Vec3>() as GLsizei, // stride
                ptr::null() // offset
            );
            gl::VertexAttribDivisor(3, 1); // Tell OpenGL this is an instanced vertex attribute.

            gl::BindVertexArray(0);
            // Unbind ARRAY_BUFFER too, good practice
            gl::BindBuffer(ARRAY_BUFFER, 0);
        }

        Self {
            vertices: model.vertices,
            indices: model.indices,
            texture,
            vao,
            vbo,
            ebo,
            instance_vbo
        }
    }
}

// don't forget to generate offset vector
pub fn draw_instanced<V, I>(src: &Mesh<V, I>, instances: GLsizei) {
    unsafe {
        gl::BindTexture(TEXTURE_2D, src.texture.id);
        gl::BindVertexArray(src.vao);

        gl::DrawElementsInstanced(
            TRIANGLES,
            src.indices.len() as GLsizei,
            UNSIGNED_SHORT,
            ptr::null(),
            instances
        );

        gl::BindVertexArray(0);
    }
}

pub fn draw<V, I>(src: &Mesh) {
    unsafe {
        gl::BindTexture(TEXTURE_2D, src.texture.id);
        gl::BindVertexArray(src.vao);

        gl::DrawElements(
            TRIANGLES,
            src.indices.len() as GLsizei,
            UNSIGNED_SHORT,
            ptr::null(),
        );

        gl::BindVertexArray(0);
    }
}