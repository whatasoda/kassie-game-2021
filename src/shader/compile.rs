use super::Shader;
use std::collections::HashMap;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader, WebGlUniformLocation};

pub struct Program {
    pub program: Option<WebGlProgram>,
    pub uniform_locations: HashMap<&'static str, WebGlUniformLocation>,
}

impl Program {
    pub fn empty() -> Self {
        Self {
            program: None,
            uniform_locations: HashMap::new(),
        }
    }
}

impl<V, I, U> Shader<'_, V, I, U> {
    fn_ensure_option!(
        [pub(super) fn ensure_program],
        program.program,
        if_none: "program uninitialized",
        if_some: "program already exists",
    );

    pub fn activate(&self) {
        self.ctx.use_program(self.program.program.as_ref());
    }

    pub fn compile(&mut self, vert: &str, frag: &str) -> Result<(), String> {
        self.ensure_program(None)?;
        let vert = compile_shader(self.ctx, WebGl2RenderingContext::VERTEX_SHADER, vert)?;
        let frag = compile_shader(self.ctx, WebGl2RenderingContext::FRAGMENT_SHADER, frag)?;
        let program = link_program(self.ctx, &vert, &frag)?;
        setup_uniform_block(self.ctx, &program)?;
        self.program.program = Some(program);
        Ok(())
    }
}

fn compile_shader(
    ctx: &WebGl2RenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = ctx
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    ctx.shader_source(&shader, source);
    ctx.compile_shader(&shader);

    if ctx
        .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(ctx
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("Unknown error creating shader")))
    }
}

fn link_program(
    ctx: &WebGl2RenderingContext,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = ctx
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader object"))?;

    ctx.attach_shader(&program, vert_shader);
    ctx.attach_shader(&program, frag_shader);
    ctx.link_program(&program);

    if ctx
        .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(ctx
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("Unknown error creating program object")))
    }
}

fn setup_uniform_block<'a>(
    ctx: &WebGl2RenderingContext,
    program: &WebGlProgram,
) -> Result<(), String> {
    let index = ctx.get_uniform_block_index(program, "uniforms_");
    ctx.uniform_block_binding(program, index, 0);
    Ok(())
}