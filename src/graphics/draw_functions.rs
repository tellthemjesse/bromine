use std::ptr;
use gl::{BindTexture, BindVertexArray, DrawElements, LINES, TEXTURE_2D, TRIANGLES, UNSIGNED_SHORT};
use crate::graphics::mesh::Texture;
use crate::resources::manager::AnyMesh;

pub fn draw<M>(src: &M, texture: Option<&Texture>)
where M: AnyMesh + ?Sized
{
    unsafe {
        if let Some(texture) = texture {
            BindTexture(texture.kind, texture.id);
        } else {
            BindTexture(TEXTURE_2D, 0);
        }

        BindVertexArray(src.vao());

        DrawElements(
            TRIANGLES,
            src.indices_len(),
            UNSIGNED_SHORT, // Use trait constant
            ptr::null(),
        );

        BindVertexArray(0);
        BindTexture(TEXTURE_2D, 0);
    }
}

pub fn draw_lines<M>(src: &M, texture: Option<&Texture>)
where M: AnyMesh + ?Sized
{
    unsafe {
        if let Some(texture) = texture {
            BindTexture(texture.kind, texture.id);
        } else {
            BindTexture(TEXTURE_2D, 0);
        }

        BindVertexArray(src.vao());

        DrawElements(
            LINES,
            src.indices_len(),
            UNSIGNED_SHORT,
            ptr::null(),
        );

        BindVertexArray(0);
        BindTexture(TEXTURE_2D, 0);
    }
}
