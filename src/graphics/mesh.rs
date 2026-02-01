use std::ffi::c_void;
use std::mem::{offset_of, size_of};
use std::os::raw::c_int;
use std::ptr;

use gl::types::{GLenum, GLint, GLsizei, GLsizeiptr, GLuint};
use gl::{
    BindBuffer, BindTexture, BindVertexArray, BufferData, DrawElements, DrawElementsInstanced,
    EnableVertexAttribArray, GenBuffers, GenVertexArrays, VertexAttribDivisor, VertexAttribPointer,
    ARRAY_BUFFER, CLAMP_TO_EDGE, ELEMENT_ARRAY_BUFFER, FLOAT, LINEAR, LINEAR_MIPMAP_LINEAR, REPEAT,
    STATIC_DRAW, TEXTURE_2D, TEXTURE_CUBE_MAP, TEXTURE_CUBE_MAP_POSITIVE_X, TEXTURE_MAG_FILTER,
    TEXTURE_MIN_FILTER, TEXTURE_WRAP_R, TEXTURE_WRAP_S, TEXTURE_WRAP_T, TRIANGLES, UNSIGNED_BYTE,
    UNSIGNED_INT, UNSIGNED_SHORT,
};

use nalgebra_glm as glm;
use obj::{Obj, Position, TexturedVertex, Vertex};
use stb_image::stb_image;

use crate::graphics::context::c_string;

#[derive(Debug, Clone)]
pub struct Texture {
    pub id: GLuint,
    pub kind: GLenum,
}

// TODO: Think about how we can create universal new() fn instead of new_TEXTURE_TYPE() ??
// But imo that seems overcomplicated, like you need to have 'dynamic' path argument(&str or &[&str])
// While also matching texture type itself
impl Texture {
    pub fn new_texture_2d(path: &str) -> Self {
        unsafe {
            // CString needs to be stored in a local variable to avoid pointer dangling
            let texture_src = c_string(path);

            let mut width: c_int = 0;
            let mut height: c_int = 0;
            let mut nr_channels: c_int = 0;

            stb_image::stbi_set_flip_vertically_on_load(1); // Flip texture vertically

            let image_data = stb_image::stbi_load(
                // as_ptr() returns read-only pointer
                texture_src.as_ptr(),
                &mut width,
                &mut height,
                &mut nr_channels,
                0,
            );

            if image_data.is_null() {
                // Handle error - maybe return Result or panic?
                panic!("Failed to load texture: {}", path);
            }

            let mut texture: GLuint = 0;

            gl::GenTextures(1, &mut texture);
            gl::BindTexture(TEXTURE_2D, texture);
            // ??? not sure about this segment below, read documentation
            gl::TexParameteri(TEXTURE_2D, TEXTURE_WRAP_S, REPEAT as GLint);
            gl::TexParameteri(TEXTURE_2D, TEXTURE_WRAP_T, REPEAT as GLint);
            gl::TexParameteri(
                TEXTURE_2D,
                TEXTURE_MIN_FILTER,
                LINEAR_MIPMAP_LINEAR as GLint,
            );
            gl::TexParameteri(TEXTURE_2D, TEXTURE_MAG_FILTER, LINEAR as GLint);

            let format = if nr_channels == 4 { gl::RGBA } else { gl::RGB };

            gl::TexImage2D(
                TEXTURE_2D,
                0,
                format as GLint, // Use determined format
                width,
                height,
                0,
                format, // Use determined format
                UNSIGNED_BYTE,
                image_data as *const c_void,
            );

            gl::GenerateMipmap(TEXTURE_2D);

            stb_image::stbi_image_free(image_data as *mut c_void);

            Texture {
                id: texture,
                kind: TEXTURE_2D,
            }
        }
    }

    pub fn new_cubemap(paths: &[&str]) -> Self {
        unsafe {
            let mut texture: GLuint = 0;
            gl::GenTextures(1, &mut texture);
            gl::BindTexture(TEXTURE_CUBE_MAP, texture);

            let texture_sources = paths.iter().map(|p| c_string(p)).collect::<Vec<_>>();

            stb_image::stbi_set_flip_vertically_on_load(0); // Cubemaps usually don't need flipping

            for (i, source) in texture_sources.iter().enumerate() {
                let mut width: c_int = 0;
                let mut height: c_int = 0;
                let mut nr_channels: c_int = 0;

                let image_data = stb_image::stbi_load(
                    source.as_ptr(),
                    &mut width,
                    &mut height,
                    &mut nr_channels,
                    0,
                );

                if image_data.is_null() {
                    panic!("Failed to load cubemap face: {}", paths[i]);
                }

                let format = if nr_channels == 4 { gl::RGBA } else { gl::RGB };

                gl::TexImage2D(
                    TEXTURE_CUBE_MAP_POSITIVE_X + i as GLenum,
                    0,
                    format as GLint,
                    width,
                    height,
                    0,
                    format,
                    UNSIGNED_BYTE,
                    image_data as *const c_void,
                );

                stb_image::stbi_image_free(image_data as *mut c_void);
            }

            gl::TexParameteri(TEXTURE_CUBE_MAP, TEXTURE_MIN_FILTER, LINEAR as GLint);
            gl::TexParameteri(TEXTURE_CUBE_MAP, TEXTURE_MAG_FILTER, LINEAR as GLint);
            gl::TexParameteri(TEXTURE_CUBE_MAP, TEXTURE_WRAP_S, CLAMP_TO_EDGE as GLint);
            gl::TexParameteri(TEXTURE_CUBE_MAP, TEXTURE_WRAP_T, CLAMP_TO_EDGE as GLint);
            gl::TexParameteri(TEXTURE_CUBE_MAP, TEXTURE_WRAP_R, CLAMP_TO_EDGE as GLint);

            Texture {
                id: texture,
                kind: TEXTURE_CUBE_MAP,
            }
        }
    }
}

#[derive(Debug)]
pub struct Mesh<V, I> {
    pub vertices: Vec<V>,
    pub indices: Vec<I>,
    pub vao: GLuint,
    vbo: GLuint,
    ebo: GLuint,
    instance_vbo: Option<GLuint>,
}

// --- Implementations for Mesh<V, I> ---

// Generic implementation (if any shared logic)
impl<V, I> Mesh<V, I> {
    // Shared methods can go here
}

// Implementation for TexturedVertex
impl<I> Mesh<TexturedVertex, I> {
    pub fn new(model: Obj<TexturedVertex, I>, instance_offsets: Option<&[glm::Vec3]>) -> Self {
        let mut vao: GLuint = 0;
        let mut vbo: GLuint = 0;
        let mut ebo: GLuint = 0;
        let mut instance_vbo: Option<GLuint> = None;

        unsafe {
            GenVertexArrays(1, &mut vao);
            GenBuffers(1, &mut vbo);
            GenBuffers(1, &mut ebo);

            if let Some(offsets) = instance_offsets {
                if !offsets.is_empty() {
                    // Only create if not empty
                    let mut vbo_id = 0;
                    GenBuffers(1, &mut vbo_id);
                    instance_vbo = Some(vbo_id);
                }
            }

            BindVertexArray(vao);

            // VBO for TexturedVertex
            BindBuffer(ARRAY_BUFFER, vbo);
            BufferData(
                ARRAY_BUFFER,
                (model.vertices.len() * size_of::<TexturedVertex>()) as GLsizeiptr,
                model.vertices.as_ptr() as *const c_void,
                STATIC_DRAW,
            );

            // EBO
            BindBuffer(ELEMENT_ARRAY_BUFFER, ebo);
            BufferData(
                ELEMENT_ARRAY_BUFFER,
                (model.indices.len() * size_of::<I>()) as GLsizeiptr, // Use size_of::<I>()
                model.indices.as_ptr() as *const c_void,
                STATIC_DRAW,
            );

            // Vertex Attributes
            let stride = size_of::<TexturedVertex>() as GLsizei;
            // Position (Location 0)
            EnableVertexAttribArray(0);
            VertexAttribPointer(
                0,
                3,
                FLOAT,
                gl::FALSE,
                stride,
                offset_of!(TexturedVertex, position) as *const c_void,
            );
            // Normal (Location 1)
            EnableVertexAttribArray(1);
            VertexAttribPointer(
                1,
                3,
                FLOAT,
                gl::FALSE,
                stride,
                offset_of!(TexturedVertex, normal) as *const c_void,
            );
            // TexCoords (Location 2)
            EnableVertexAttribArray(2);
            VertexAttribPointer(
                2,
                2,
                FLOAT,
                gl::FALSE,
                stride,
                offset_of!(TexturedVertex, texture) as *const c_void,
            );

            // Instance VBO Setup
            if let (Some(offsets), Some(inst_vbo_id)) = (instance_offsets, instance_vbo) {
                // We already checked if offsets is empty when creating buffer
                BindBuffer(ARRAY_BUFFER, inst_vbo_id);
                BufferData(
                    ARRAY_BUFFER,
                    (offsets.len() * size_of::<glm::Vec3>()) as GLsizeiptr,
                    offsets.as_ptr() as *const c_void,
                    STATIC_DRAW,
                );
                // Instance Attribute (Offset - Location 3)
                EnableVertexAttribArray(3);
                VertexAttribPointer(
                    3,
                    3,
                    FLOAT,
                    gl::FALSE,
                    size_of::<glm::Vec3>() as GLsizei,
                    ptr::null(),
                );
                VertexAttribDivisor(3, 1); // Instanced attribute
            }

            BindVertexArray(0);
            BindBuffer(ARRAY_BUFFER, 0); // Unbind VBO after VAO configuration
                                         // EBO stays bound to VAO
        }

        Self {
            vertices: model.vertices,
            indices: model.indices,
            vao,
            vbo,
            ebo,
            instance_vbo,
        }
    }
}

// Implementation for Vertex (Position + Normal)
impl<I> Mesh<Vertex, I> {
    pub fn new(model: Obj<Vertex, I>) -> Self {
        let mut vao: GLuint = 0;
        let mut vbo: GLuint = 0;
        let mut ebo: GLuint = 0;

        unsafe {
            GenVertexArrays(1, &mut vao);
            GenBuffers(1, &mut vbo);
            GenBuffers(1, &mut ebo);

            BindVertexArray(vao);

            // VBO for Vertex
            BindBuffer(ARRAY_BUFFER, vbo);
            BufferData(
                ARRAY_BUFFER,
                (model.vertices.len() * size_of::<Vertex>()) as GLsizeiptr,
                model.vertices.as_ptr() as *const c_void,
                STATIC_DRAW,
            );

            // EBO
            BindBuffer(ELEMENT_ARRAY_BUFFER, ebo);
            BufferData(
                ELEMENT_ARRAY_BUFFER,
                (model.indices.len() * size_of::<I>()) as GLsizeiptr, // Use size_of::<I>()
                model.indices.as_ptr() as *const c_void,
                STATIC_DRAW,
            );

            // Vertex Attributes
            let stride = size_of::<Vertex>() as GLsizei;
            // Position (Location 0)
            EnableVertexAttribArray(0);
            VertexAttribPointer(
                0,
                3,
                FLOAT,
                gl::FALSE,
                stride,
                offset_of!(Vertex, position) as *const c_void,
            );
            // Normal (Location 1)
            EnableVertexAttribArray(1);
            VertexAttribPointer(
                1,
                3,
                FLOAT,
                gl::FALSE,
                stride,
                offset_of!(Vertex, normal) as *const c_void,
            );

            BindVertexArray(0);
            BindBuffer(ARRAY_BUFFER, 0);
        }

        Self {
            vertices: model.vertices,
            indices: model.indices,
            vao,
            vbo,
            ebo,
            instance_vbo: None,
        }
    }
}

impl Mesh<Position, u16> {
    pub fn new_from_source(vertices: Vec<Position>, indices: Vec<u16>) -> Mesh<Position, u16> {
        let mut vao: GLuint = 0;
        let mut vbo: GLuint = 0;
        let mut ebo: GLuint = 0;

        unsafe {
            GenVertexArrays(1, &mut vao);
            GenBuffers(1, &mut vbo);
            GenBuffers(1, &mut ebo);

            BindVertexArray(vao);

            // VBO for Position
            BindBuffer(ARRAY_BUFFER, vbo);
            BufferData(
                ARRAY_BUFFER,
                (vertices.len() * size_of::<Position>()) as GLsizeiptr,
                vertices.as_ptr() as *const c_void,
                STATIC_DRAW,
            );

            // EBO
            BindBuffer(ELEMENT_ARRAY_BUFFER, ebo);
            BufferData(
                ELEMENT_ARRAY_BUFFER,
                (indices.len() * size_of::<u16>()) as GLsizeiptr, // Use size_of::<I>()
                indices.as_ptr() as *const c_void,
                STATIC_DRAW,
            );

            // Vertex Attributes
            let stride = size_of::<Position>() as GLsizei;
            // Position (Location 0)
            EnableVertexAttribArray(0);
            VertexAttribPointer(0, 3, FLOAT, gl::FALSE, stride, 0 as *const c_void); // Offset is 0

            BindVertexArray(0);
            BindBuffer(ARRAY_BUFFER, 0);
        }

        Self {
            vertices,
            indices,
            vao,
            vbo,
            ebo,
            instance_vbo: None,
        }
    }
}

// Implementation for Position only
impl<I> Mesh<Position, I> {
    pub fn new(model: Obj<Position, I>) -> Self {
        let mut vao: GLuint = 0;
        let mut vbo: GLuint = 0;
        let mut ebo: GLuint = 0;

        unsafe {
            GenVertexArrays(1, &mut vao);
            GenBuffers(1, &mut vbo);
            GenBuffers(1, &mut ebo);

            BindVertexArray(vao);

            // VBO for Position
            BindBuffer(ARRAY_BUFFER, vbo);
            BufferData(
                ARRAY_BUFFER,
                (model.vertices.len() * size_of::<Position>()) as GLsizeiptr,
                model.vertices.as_ptr() as *const c_void,
                STATIC_DRAW,
            );

            // EBO
            BindBuffer(ELEMENT_ARRAY_BUFFER, ebo);
            BufferData(
                ELEMENT_ARRAY_BUFFER,
                (model.indices.len() * size_of::<I>()) as GLsizeiptr, // Use size_of::<I>()
                model.indices.as_ptr() as *const c_void,
                STATIC_DRAW,
            );

            // Vertex Attributes
            let stride = size_of::<Position>() as GLsizei;
            // Position (Location 0)
            EnableVertexAttribArray(0);
            VertexAttribPointer(0, 3, FLOAT, gl::FALSE, stride, 0 as *const c_void); // Offset is 0

            BindVertexArray(0);
            BindBuffer(ARRAY_BUFFER, 0);
        }

        Self {
            vertices: model.vertices,
            indices: model.indices,
            vao,
            vbo,
            ebo,
            instance_vbo: None,
        }
    }
}

// --- Helper Trait for Index Type ---
pub trait GlIndexType {
    const GL_TYPE: GLenum;
}

impl GlIndexType for u16 {
    const GL_TYPE: GLenum = UNSIGNED_SHORT;
}

impl GlIndexType for u32 {
    const GL_TYPE: GLenum = UNSIGNED_INT;
}

// --- Draw Functions ---

pub fn draw_instanced<V, I: GlIndexType>(
    src: &Mesh<V, I>,
    texture: Option<Texture>,
    instances: GLsizei,
) {
    unsafe {
        if let Some(ref texture) = texture {
            BindTexture(texture.kind, texture.id);
        } else {
            BindTexture(TEXTURE_2D, 0);
        }

        BindVertexArray(src.vao);

        DrawElementsInstanced(
            TRIANGLES,
            src.indices.len() as GLsizei,
            I::GL_TYPE, // Use trait constant
            ptr::null(),
            instances,
        );

        BindVertexArray(0);
        BindTexture(TEXTURE_2D, 0);
    }
}

pub fn draw<V, I: GlIndexType>(src: &Mesh<V, I>, texture: Option<&Texture>) {
    unsafe {
        if let Some(ref texture) = texture {
            BindTexture(texture.kind, texture.id);
        } else {
            BindTexture(TEXTURE_2D, 0);
        }

        BindVertexArray(src.vao);

        DrawElements(
            TRIANGLES,
            src.indices.len() as GLsizei,
            I::GL_TYPE, // Use trait constant
            ptr::null(),
        );

        BindVertexArray(0);
        BindTexture(TEXTURE_2D, 0);
    }
}
