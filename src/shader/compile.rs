use super::Shader;
use std::collections::HashMap;
use web_sys::{WebGlProgram, WebGlRenderingContext, WebGlShader, WebGlUniformLocation};

impl<V, I> Shader<'_, V, I> {
    pub fn compile(
        &mut self,
        attributes: Vec<&'static str>,
        uniforms: Vec<&'static str>,
        vert: &str,
        frag: &str,
    ) -> Result<(), String> {
        if self.program.program.is_some() {
            return Err(String::from("instance has already used for another shader"));
        }
        let vert = compile_shader(self.ctx, WebGlRenderingContext::VERTEX_SHADER, vert)?;
        let frag = compile_shader(self.ctx, WebGlRenderingContext::FRAGMENT_SHADER, frag)?;
        let program = link_program(self.ctx, &vert, &frag)?;
        collect_attrib_locations(
            self.ctx,
            &mut self.program.attrib_locations,
            &program,
            &attributes,
        )?;
        collect_uniform_locations(
            self.ctx,
            &mut self.program.uniform_locations,
            &program,
            &uniforms,
        )?;
        self.program.program = Some(program);
        Ok(())
    }
}

pub struct Program {
    pub program: Option<WebGlProgram>,
    pub attrib_locations: HashMap<&'static str, u32>,
    pub uniform_locations: HashMap<&'static str, WebGlUniformLocation>,
}

impl Program {
    pub fn empty() -> Self {
        Self {
            program: None,
            attrib_locations: HashMap::new(),
            uniform_locations: HashMap::new(),
        }
    }
}

fn compile_shader(
    ctx: &WebGlRenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = ctx
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    ctx.shader_source(&shader, source);
    ctx.compile_shader(&shader);

    if ctx
        .get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS)
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
    ctx: &WebGlRenderingContext,
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
        .get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS)
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

fn collect_attrib_locations<'a>(
    ctx: &WebGlRenderingContext,
    acc: &mut HashMap<&'a str, u32>,
    program: &WebGlProgram,
    attirbutes: &Vec<&'a str>,
) -> Result<(), String> {
    let mut iter = attirbutes.iter();
    while let Some(name) = iter.next() {
        let loc = ctx.get_attrib_location(&program, name);
        if loc == -1 {
            return Err(format!("attribute '{}' was not defined with program", name));
        }
        acc.insert(name, loc as u32);
    }
    Ok(())
}

fn collect_uniform_locations<'a>(
    ctx: &WebGlRenderingContext,
    acc: &mut HashMap<&'a str, WebGlUniformLocation>,
    program: &WebGlProgram,
    uniforms: &Vec<&'a str>,
) -> Result<(), String> {
    let mut iter = uniforms.iter();
    while let Some(name) = iter.next() {
        let loc = ctx
            .get_uniform_location(&program, name)
            .ok_or_else(|| format!("attribute '{}' was not defined with program", name))?;
        acc.insert(name, loc);
    }
    Ok(())
}
