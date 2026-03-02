use std::{alloc::Layout, ffi::c_void};

use anyhow::anyhow;

use super::buffer_object::*;
#[derive(Debug)]
pub struct GlBufferObject {
    id: u32,
    desc: BufferObjDesc,
    layout: Layout,
}

impl GlBufferObject {
    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn desc(&self) -> &BufferObjDesc {
        &self.desc
    }

    pub fn layout(&self) -> &Layout {
        &self.layout
    }
}

pub fn new_buffer_object<T>(
    data: Vec<T>,
    desc: BufferObjDesc,
) -> anyhow::Result<GlBufferObject> {
    let layout = Layout::for_value(&data);
    let mut buffer = 0;

    unsafe {
        gl::GenBuffers(1, &mut buffer);
        gl::BindBuffer(desc.kind as u32, buffer);
        gl::BufferData(
            desc.kind as u32,
            layout.size() as isize,
            data.as_ptr() as *const c_void,
            desc.usage as u32,
        );

        let err = gl::GetError();
        if err != gl::NO_ERROR {
            gl::DeleteBuffers(1, &buffer);
            gl::BindBuffer(desc.kind as u32, 0);
            return Err(anyhow!("failed to buffer data, err code: {err}"));
        }

        gl::BindBuffer(desc.kind as u32, 0);
    }

    Ok(GlBufferObject {
        id: buffer,
        desc,
        layout,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_operations() {
        let mut tfn = gl_headless::GlHeadless::new(|| {
            let position = vec![0.0, 1.0, 0.0];
            let desc = BufferObjDesc::new(BufferObjKind::Vertex, BufferUsage::StaticDraw);

            let _ = new_buffer_object(position, desc).unwrap();
        });
        let _ = tfn.run_once();
    }
}
