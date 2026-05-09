//! Low-level implementation for [`uniforms`](super::uniform)

use super::{
    program::{UniformBlocksList, UniformVariablesList}, uniform::*
};
use crate::render::prelude::{BufferObjDesc, BufferObjKind, GlBufferObject};
use anyhow::ensure;
use std::{
    ffi::{CString, c_char, c_int, c_uint}, ptr
};

/// Wraps shader program to perform a set of operations on its uniforms
pub(crate) struct GlslUniforms(pub c_uint);

impl GlslUniforms {
    /// Returns a list over active uniform variables
    pub fn get_globals(&self) -> anyhow::Result<Vec<GlobalIdentifier>> {
        let mut active_uniforms = 0;
        // length of the longest uniform variable name including null terminator
        let mut buf_size = 0;

        unsafe {
            gl::GetProgramiv(self.0, gl::ACTIVE_UNIFORMS, &mut active_uniforms);
            gl::GetProgramiv(self.0, gl::ACTIVE_UNIFORM_MAX_LENGTH, &mut buf_size);
        }

        (0..active_uniforms as c_uint)
            .flat_map(|idx| self.get_global(idx, buf_size).transpose())
            .collect()
    }
    /// Resolves uniform variable by its index
    fn get_global(
        &self,
        index: c_uint,
        buf_size: c_int,
    ) -> anyhow::Result<Option<GlobalIdentifier>> {
        // raw pointer cast between *mut u8 and *mut c_char (*mut i8) is safe
        // and the type signature for Vec<T> can be inferred from the context,
        // but specifying T to be u8 is clearer
        let mut buf: Vec<u8> = Vec::with_capacity(buf_size as usize);
        let mut buf_length = 0;
        let buf_view;
        let mut type_ = 0;

        unsafe {
            gl::GetActiveUniform(
                self.0,
                index,
                buf_size,
                &mut buf_length,
                ptr::null_mut(),
                &mut type_,
                buf.as_mut_ptr() as *mut c_char,
            );
            // safety: buf_length is guarateed to be less then buf_size
            buf.set_len(buf_length as usize);
            // safety: utf8 encoding is forced in shaders
            buf_view = str::from_utf8_unchecked(&buf);
        }

        let name = String::from(buf_view);

        if name.starts_with("gl_") {
            return Ok(None);
        }

        let name_cstring = CString::new(buf_view)?;
        let location = self.get_location(name_cstring.as_ptr());

        let datatype = GlslDatatype::try_from(type_)?;

        let var_meta = UniformVar { name, datatype };

        if location == -1 {
            return Ok(None);
        }

        Ok(Some((var_meta, location as u32))) // this cast won't overflow
    }
    /// Returns a list over active uniform blocks
    pub fn get_blocks(&self) -> anyhow::Result<Vec<BlockIdentifier>> {
        let mut active_blocks = 0;
        // length of the longest uniform block name including null terminator
        let mut buf_size = 0;

        unsafe {
            gl::GetProgramiv(self.0, gl::ACTIVE_UNIFORM_BLOCKS, &mut active_blocks);
            gl::GetProgramiv(
                self.0,
                gl::ACTIVE_UNIFORM_BLOCK_MAX_NAME_LENGTH,
                &mut buf_size,
            );
        }

        (0..active_blocks as c_uint)
            .map(|idx| self.get_block(idx, buf_size))
            .collect()
    }
    /// Resolves uniform block by its index
    fn get_block(&self, index: c_uint, buf_size: c_int) -> anyhow::Result<BlockIdentifier> {
        let mut buf: Vec<u8> = Vec::with_capacity(buf_size as usize);
        // length of the uniform block name excluding null terminator
        let mut buf_length = 0;

        unsafe {
            gl::GetActiveUniformBlockName(
                self.0,
                index,
                buf_size,
                &mut buf_length,
                buf.as_mut_ptr() as *mut c_char,
            );
            // safety: buf_length is guarateed to be less then buf_size
            buf.set_len(buf_length as usize);
        }

        let name = String::from_utf8(buf)?;

        let mut binding = -1;

        unsafe {
            gl::GetActiveUniformBlockiv(self.0, index, gl::UNIFORM_BLOCK_BINDING, &mut binding);
        }

        ensure!(binding >= 0);

        Ok((name, binding as u32))
    }
    /// Resolves uniform variable location by its name
    fn get_location(&self, name: *const c_char) -> c_int {
        unsafe { gl::GetUniformLocation(self.0, name) }
    }
    /// Transforms the list over active uniform variables to the format, suitable for a program desciptor
    pub fn globals_to_list(
        &self,
        globals: impl IntoIterator<Item = GlobalIdentifier>,
    ) -> UniformVariablesList {
        globals
            .into_iter()
            .map(|(var, location)| UniformVarDesc::new(var.name, var.datatype, location))
            .collect()
    }
    /// Transforms the list over active uniform blocks to the format, suitable for a program descriptor
    pub fn blocks_to_list(
        &self,
        blocks: impl IntoIterator<Item = BlockIdentifier>,
    ) -> UniformBlocksList {
        blocks
            .into_iter()
            .map(|(name, binding)| UniformBlockDesc::new(name, binding))
            .collect()
    }
}

pub struct GlUniformBuffer {
    buf: GlBufferObject,
}

impl GlUniformBuffer {
    /// Creates new empty buffer object
    ///
    /// # Errors
    /// Fails if the provided descriptor kind isn't [`BufferObjKind::Uniform`]
    pub fn generate(desc: BufferObjDesc) -> anyhow::Result<Self> {
        ensure!(
            matches!(desc.kind, BufferObjKind::Uniform),
            "buffer kind mismatch: expected {:?}, got {:?}",
            BufferObjKind::Uniform,
            desc.kind
        );

        let buf = GlBufferObject::generate(desc);
        Ok(Self { buf })
    }
    /// Binds this buffer
    pub fn bind(&self) {
        self.buf.bind();
    }
    /// Submits data to the GPU
    ///
    /// # Safety
    /// The caller must ensure that this buffer object is active
    ///
    /// # Errors
    /// Fails if the `data` is empty
    pub unsafe fn write<T>(&mut self, data: Vec<T>) -> anyhow::Result<()> {
        unsafe { self.buf.write(data) }
    }
    /// Allocates memory for `count` elements
    ///
    /// # Safety
    /// The caller must ensure that this buffer object is active
    pub unsafe fn write_zeroed<T>(&mut self, count: usize) {
        unsafe {
            self.buf.write_zeroed::<T>(count);
        }
    }
    /// Unbinds this buffer
    pub fn unbind(&self) {
        self.buf.unbind();
    }
    /// Binds this buffer to the specified binding point
    ///
    /// # Errors
    /// Fails if the provided `binding` is greater or equal to the indexed number of binding points
    /// supported by *target*,
    /// or if the underlying buffer is empty
    pub fn buffer_base(&self, binding: c_uint) -> anyhow::Result<()> {
        ensure!(
            self.buf.count() > 0,
            "cannot call this operation on an empty buffer"
        );

        unsafe {
            gl::BindBufferBase(self.buf.desc().kind as c_uint, binding, self.buf.id());
        }

        let err = unsafe { gl::GetError() };
        ensure!(
            err == gl::NO_ERROR,
            "operation failed, err code {err} ({err:#X})"
        );

        Ok(())
    }
}
