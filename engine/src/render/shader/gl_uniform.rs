use super::uniform::*;
use super::program::{UniformBlocksList, UniformVariablesList};
use anyhow::{Result, ensure};
use std::ffi::{CString, c_char, c_uint, c_int};
use std::ptr;

pub(crate) struct GlslUniforms(pub c_uint);

impl GlslUniforms {
    /// Returns a list over active uniform variables
    pub fn get_varibles(&self) -> Result<Vec<UniformVarType>> {
        let mut active_uniforms = 0;
        // length of the longest uniform variable name including null terminator
        let mut buf_size = 0;

        unsafe {
            gl::GetProgramiv(self.0, gl::ACTIVE_UNIFORMS, &mut active_uniforms);
            gl::GetProgramiv(self.0, gl::ACTIVE_UNIFORM_MAX_LENGTH, &mut buf_size);
        }

        (0..active_uniforms as c_uint)
            .map(|idx| self.get_variable( idx, buf_size))
            .collect()
    }
    /// Resolves uniform variable by its index
    fn get_variable(&self, index: c_uint, buf_size: c_int) -> Result<UniformVarType> {
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
            return Ok(UniformVarType::Builtin);
        }

        let name_cstring = CString::new(buf_view)?;
        let location = self.get_location(name_cstring.as_ptr());

        let datatype = GlslDatatype::try_from(type_)?;

        let var_meta = UniformVarMeta { name, datatype };

        if location == -1 {
            let mut block_idx = -1;
            unsafe {
                gl::GetActiveUniformsiv(self.0, 1, &index, gl::UNIFORM_BLOCK_INDEX, &mut block_idx);
            }
            // if location is -1, this is either an atomic or a uniform block, so the index is always >= 0
            return Ok(UniformVarType::Scoped(var_meta, block_idx as u32));
        }

        Ok(UniformVarType::Global(var_meta, location as u32)) // this cast won't overflow
    }
    /// Returns a list over active uniform blocks
    pub fn get_uniform_blocks(&self) -> Result<Vec<UniformBlockMeta>> {
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
    fn get_block(
        &self,
        index: c_uint,
        buf_size: c_int,
    ) -> Result<UniformBlockMeta> {
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

        Ok(UniformBlockMeta {
            name,
            binding: binding as u32,
            index,
        })
    }

    fn get_location(&self, name: *const c_char) -> i32 {
        unsafe {
            gl::GetUniformLocation(self.0, name)
        }
    }

    pub fn variables_to_list(&self, vars: &[UniformVarType], blocks: &[UniformBlockMeta]) -> UniformVariablesList {
        vars.iter().filter_map(|var| match var.clone() {
            UniformVarType::Global(meta, location) =>
            Some(UniformVarDesc::new(meta.name, meta.datatype, location)),
            UniformVarType::Scoped(meta, index) => {
                let block_name = blocks.iter().find(|block| block.index == index).unwrap().name.clone();
                let name = format!("{}.{}", block_name, meta.name);
                let name_cstring = CString::new(name.clone()).unwrap();
                let location = self.get_location(name_cstring.as_ptr());
                Some(UniformVarDesc::new(name, meta.datatype, location as u32))
            },
            UniformVarType::Builtin => None,
        })
        .collect()
    }

    pub fn blocks_to_list(&self, blocks: &[UniformBlockMeta]) -> UniformBlocksList {
        blocks.iter().map(|block| UniformBlockDesc::new(block.name.clone(), block.binding)).collect()
    }
}
