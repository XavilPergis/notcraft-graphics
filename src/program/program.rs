use super::{super::error::GlError, shader::CompiledShader};
use crate::context::Context;
use crate::program::uniform::*;
use gl::{self, types::*};
use std::collections::HashMap;
use std::marker::PhantomData;

pub struct RawProgram {
    id: u32,
}

impl RawProgram {
    pub fn new(_ctx: &Context) -> Self {
        let id = gl_call!(assert CreateProgram());
        assert!(id > 0);
        RawProgram { id }
    }
}

pub struct ProgramBuilder {
    raw: RawProgram,
}

impl ProgramBuilder {
    pub fn new(ctx: &Context) -> Self {
        ProgramBuilder {
            raw: RawProgram::new(ctx),
        }
    }

    pub fn attach_shader(&self, shader: CompiledShader) {
        gl_call!(assert AttachShader(self.raw.id, shader.shader.id));
    }

    // FIXME: actually validate the shader interface and don't just return a typed version!
    pub fn link<I>(self) -> Result<Program<I>, LinkError> {
        gl_call!(LinkProgram(self.raw.id))?;
        check_program_status(self.raw.id, gl::LINK_STATUS)?;
        gl_call!(ValidateProgram(self.raw.id))?;
        check_program_status(self.raw.id, gl::VALIDATE_STATUS)?;
        Ok(Program {
            raw: self.raw,
            uniform_cache: HashMap::new(),
            _marker: PhantomData,
        })
    }
}

pub struct Program<I> {
    raw: RawProgram,
    uniform_cache: HashMap<String, UniformLocation>,
    _marker: PhantomData<*const I>,
}

impl<I> Program<I> {
    pub fn set_uniform<U: Uniform>(&mut self, ctx: &Context, name: &str, uniform: &U) {
        self.bind();
        if let Some(&location) = self.uniform_cache.get(name) {
            uniform.set_uniform(ctx, location);
        } else {
            let location = self.get_uniform_location(name);
            self.uniform_cache.insert(name.into(), location);
            uniform.set_uniform(ctx, location);
        }
    }

    pub fn bind(&self) {
        gl_call!(assert UseProgram(self.raw.id));
    }

    fn get_uniform_location(&self, name: &str) -> UniformLocation {
        use std::ffi::CString;
        let c_string = CString::new(name).unwrap();
        // UNWRAP: program ID is valid, and the program has been successfully linked
        gl_call!(assert GetUniformLocation(self.raw.id, c_string.as_ptr()))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum LinkError {
    Other(String),
    Gl(GlError),
}

impl From<crate::error::GlError> for LinkError {
    fn from(err: crate::error::GlError) -> Self {
        LinkError::Gl(err)
    }
}

fn check_program_status(id: GLuint, ty: GLenum) -> Result<(), LinkError> {
    let mut status = 1;
    gl_call!(assert GetProgramiv(id, ty, &mut status));

    if status == 0 {
        Err(LinkError::Other(
            program_info_log(id).unwrap_or(String::new()),
        ))
    } else {
        Ok(())
    }
}

fn program_info_log(id: GLuint) -> Option<String> {
    let mut length = 0;
    gl_call!(assert GetProgramiv(id, gl::INFO_LOG_LENGTH, &mut length));
    if length == 0 {
        None
    } else {
        let mut buffer = Vec::<u8>::with_capacity(length as usize);
        gl_call!(assert GetProgramInfoLog(
            id,
            length,
            ::std::ptr::null_mut(),
            buffer.as_mut_ptr() as *mut i8
        ));

        unsafe {
            buffer.set_len((length - 1) as usize);
        }

        Some(String::from_utf8(buffer).expect("Program info log was not UTF-8"))
    }
}
