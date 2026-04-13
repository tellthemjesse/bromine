//! Low-level implementation for [`shaders`](super::shader) and [`uniform`](super::uniform) modules

use super::{shader::*, uniform::*};
use anyhow::{bail, ensure};
use std::{
    ffi::{CString, c_char, c_int, c_uint},
    ptr,
};

type UniformVariableList = Vec<UniformVarType>;

#[derive(Debug)]
/// Represents OpenGL shader
pub struct GlShader {
    id: u32,
    desc: ShaderDesc,
}

impl GlShader {
    /// Returns the underlying object id
    pub fn id(&self) -> u32 {
        self.id
    }
    /// Returns object descriptor
    pub fn desc(&self) -> &ShaderDesc {
        &self.desc
    }
}

#[derive(Debug)]
/// Represents OpenGL shader program
pub struct GlShaderProgram {
    id: u32,
    desc: ShaderProgramDesc,
}

impl GlShaderProgram {
    /// Returns the underlying object id
    pub fn id(&self) -> u32 {
        self.id
    }
    /// Returns object descriptor
    pub fn desc(&self) -> &ShaderProgramDesc {
        &self.desc
    }
    /// Sets uniform value using data record
    pub fn uniform_value_s<T>(&self, u: UniformVariable<T>) {
        if let Some(active_uniform) = self.desc.uniforms.get(u.name()) {
            let location = active_uniform.location as i32;
            unsafe {
                let ptr = u.value_ptr();

                match u.datatype() {
                    GlslDatatype::Float => gl::Uniform1fv(location, 1, ptr as _),
                    GlslDatatype::Vec3 => gl::Uniform3fv(location, 1, ptr as _),
                    GlslDatatype::Mat4 => {
                        gl::UniformMatrix4fv(location, 1, gl::FALSE, ptr as _);
                    }
                    GlslDatatype::Sampler2D => todo!(),
                }
            }
        }
    }
    /// Sets uniform value using trait implemenation for `T`
    pub fn uniform_value_t<T>(&self, uv: impl UniformValue<T>) {
        if let Some(active_uniform) = self.desc.uniforms.get(uv.name()) {
            let location = active_uniform.location as i32;
            let ptr = uv.value_ptr();
            unsafe {
                match uv.datatype() {
                    GlslDatatype::Float => gl::Uniform1fv(location, 1, ptr as _),
                    GlslDatatype::Vec3 => gl::Uniform3fv(location, 1, ptr as _),
                    GlslDatatype::Mat4 => {
                        gl::UniformMatrix4fv(location, 1, gl::FALSE, ptr as _);
                    }
                    GlslDatatype::Sampler2D => todo!(),
                }
            }
        }
    }
    /// Binds the shader program
    pub fn bind(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }
    /// Unbinds the shader program
    pub fn unbind(&self) {
        unsafe {
            gl::UseProgram(0);
        }
    }
}

/// Compiles shader
pub fn compile_shader(source: impl Into<Vec<u8>>, desc: ShaderDesc) -> anyhow::Result<GlShader> {
    let shader;
    let src = CString::new(source)?;
    let mut status = gl::FALSE as i32;

    unsafe {
        shader = gl::CreateShader(desc.stage as c_uint);
        gl::ShaderSource(shader, 1, &src.as_ptr(), ptr::null());
        gl::CompileShader(shader);
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);
    }

    if status != (gl::TRUE as i32) {
        let mut len = 0;

        unsafe {
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
        }

        let mut buf: Vec<u8> = Vec::with_capacity(len as usize);

        unsafe {
            buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
            gl::GetShaderInfoLog(
                shader,
                len,
                ptr::null_mut(),
                buf.as_mut_ptr() as *mut c_char,
            );
        }

        bail!("shader compilation failed: {}", str::from_utf8(&buf)?);
    }

    Ok(GlShader { id: shader, desc })
}

/// Links shader program
///
/// # Note
///
/// Deletes [`GlShader`] objects after done linking
pub fn link_program(shaders: Vec<GlShader>) -> anyhow::Result<GlShaderProgram> {
    let program: u32;
    let mut is_linked = gl::FALSE as i32;

    unsafe {
        program = gl::CreateProgram();
        for shader in &shaders {
            gl::AttachShader(program, shader.id);
        }
        gl::LinkProgram(program);
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut is_linked);
    }

    if is_linked != gl::TRUE as i32 {
        let mut len = 0;

        unsafe {
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
        }

        let mut buf: Vec<u8> = Vec::with_capacity(len as usize);

        unsafe {
            buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
            gl::GetProgramInfoLog(
                program,
                len,
                ptr::null_mut(),
                buf.as_mut_ptr() as *mut gl::types::GLchar,
            );
        }

        bail!("failed to link a program: {}", str::from_utf8(&buf)?);
    }

    // store shader descriptors
    let mut shaders_ = Vec::with_capacity(shaders.len());
    for shader in shaders {
        unsafe {
            gl::DeleteShader(shader.id);
        }
        shaders_.push(shader.desc);
    }

    let var_list = get_uniform_variables(program)?;
    let uniforms = map_to_uniforms(&var_list);
    let block_list = get_uniform_blocks(program)?;
    let blocks = map_to_uniform_blocks(&var_list, &block_list);

    Ok(GlShaderProgram {
        id: program,
        desc: ShaderProgramDesc::new(shaders_, uniforms, blocks),
    })
}

/// Returns a list over active uniform variables
fn get_uniform_variables(program: c_uint) -> anyhow::Result<UniformVariableList> {
    let mut active_uniforms = 0;
    // length of the longest uniform variable name including null terminator
    let mut buf_size = 0;

    unsafe {
        gl::GetProgramiv(program, gl::ACTIVE_UNIFORMS, &mut active_uniforms);
        gl::GetProgramiv(program, gl::ACTIVE_UNIFORM_MAX_LENGTH, &mut buf_size);
    }

    (0..active_uniforms as c_uint)
        .map(|idx| get_uniform_var(program, idx, buf_size))
        .collect()
}

fn get_uniform_var(
    program: c_uint,
    index: c_uint,
    buf_size: c_int,
) -> anyhow::Result<UniformVarType> {
    // raw pointer cast between *mut u8 and *mut c_char (*mut i8) is safe
    // and the type signature for Vec<T> can be inferred from the context,
    // but specifying T to be u8 is clearer
    let mut buf: Vec<u8> = Vec::with_capacity(buf_size as usize);
    let mut buf_length = 0;
    let buf_view;
    let mut type_ = 0;

    unsafe {
        gl::GetActiveUniform(
            program,
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
    let location = unsafe { gl::GetUniformLocation(program, name_cstring.as_ptr()) };

    let datatype = GlslDatatype::try_from(type_)?;

    let var_meta = UniformVarMeta { name, datatype };

    if location == -1 {
        let mut block_idx = -1;
        unsafe {
            gl::GetActiveUniformsiv(program, 1, &index, gl::UNIFORM_BLOCK_INDEX, &mut block_idx);
        }
        // if location is -1, this is either an atomic or a uniform block, so the index is always >= 0
        return Ok(UniformVarType::Scoped(var_meta, block_idx as u32));
    }

    Ok(UniformVarType::Global(var_meta, location as u32))
}

/// Maps a list of uniform variables to HashMap
fn map_to_uniforms(var_list: &UniformVariableList) -> UniformVariablesMap {
    var_list
        .clone()
        .iter()
        .filter_map(|var| match var.clone() {
            UniformVarType::Global(meta, location) => {
                Some((meta.name, UniformVarDesc::new(meta.datatype, location)))
            }
            _ => None,
        })
        .collect()
}

fn get_uniform_blocks(program: c_uint) -> anyhow::Result<Vec<UniformBlockMeta>> {
    let mut active_blocks = 0;
    // length of the longest uniform block name including null terminator
    let mut buf_size = 0;

    unsafe {
        gl::GetProgramiv(program, gl::ACTIVE_UNIFORM_BLOCKS, &mut active_blocks);
        gl::GetProgramiv(
            program,
            gl::ACTIVE_UNIFORM_BLOCK_MAX_NAME_LENGTH,
            &mut buf_size,
        );
    }

    (0..active_blocks as c_uint)
        .map(|idx| get_uniform_block(program, idx, buf_size))
        .collect()
}

fn get_uniform_block(
    program: c_uint,
    index: c_uint,
    buf_size: c_int,
) -> anyhow::Result<UniformBlockMeta> {
    let mut buf: Vec<u8> = Vec::with_capacity(buf_size as usize);
    // length of the uniform block name excluding null terminator
    let mut buf_length = 0;

    unsafe {
        gl::GetActiveUniformBlockName(
            program,
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
        gl::GetActiveUniformBlockiv(program, index, gl::UNIFORM_BLOCK_BINDING, &mut binding);
    }

    ensure!(binding >= 0);

    Ok(UniformBlockMeta {
        name,
        binding: binding as u32,
        index,
    })
}

fn map_to_uniform_blocks(
    var_list: &UniformVariableList,
    block_list: &Vec<UniformBlockMeta>,
) -> UniformBlocksMap {
    let fields: Vec<(u32, String, GlslDatatype)> = var_list
        .clone()
        .iter()
        .filter_map(|var| match var.clone() {
            UniformVarType::Scoped(meta, index) => Some((index, meta.name, meta.datatype)),
            _ => None,
        })
        .collect();

    block_list
        .clone()
        .iter()
        .map(|var| {
            let block_idx = var.index;
            let block_fields = fields
                .iter()
                .filter_map(|(idx, name, datatype)| match block_idx == *idx {
                    true => Some((name.clone(), datatype.clone())),
                    false => None,
                })
                .collect();

            (
                var.name.clone(),
                UniformBlockDesc::new(var.binding, block_fields),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const VERTEX_SHADER: &'static str = r"
        #version 460 core

        layout (location = 0) in vec3 v_Position;
        layout (location = 1) in vec2 v_TexCoords;
        layout (location = 0) out vec2 f_TexCoords;

        void main() {
            gl_Position = vec4(v_Position, 1.0);
            f_TexCoords = v_TexCoords;
        }
    ";

    const FRAGMENT_SHADER: &'static str = r"
        #version 460 core

        layout (location = 0) in vec2 f_TexCoords;
        layout (location = 0) out vec4 FragColor;
        layout (location = 0) uniform sampler2D u_Texture;

        layout (std140, binding = 1) uniform ubo_Light {
            vec3 position;
            vec3 color;
        };

        void main() {
            FragColor = texture(u_Texture, f_TexCoords);
        }
    ";

    #[test]
    fn test_shader_operations() {
        let display = gl_headless::build_display();
        display.load_gl();

        let v_desc = ShaderDesc::vert("v_test");
        let v_shader = compile_shader(VERTEX_SHADER, v_desc).unwrap();

        let f_desc = ShaderDesc::frag("f_test");
        let f_shader = compile_shader(FRAGMENT_SHADER, f_desc).unwrap();

        let _ = link_program(vec![v_shader, f_shader]).unwrap();
    }
}
