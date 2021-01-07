use super::Shader;
use std::collections::HashMap;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader, WebGlUniformLocation};

impl<V, I, U> Shader<'_, V, I, U> {
    pub fn compile(
        &mut self,
        attributes: Vec<&'static str>,
        vert: &str,
        frag: &str,
    ) -> Result<(), String> {
        if self.program.program.is_some() {
            return Err(String::from("instance has already used for another shader"));
        }
        let vert = compile_shader(self.ctx, WebGl2RenderingContext::VERTEX_SHADER, vert)?;
        let frag = compile_shader(self.ctx, WebGl2RenderingContext::FRAGMENT_SHADER, frag)?;
        let program = link_program(self.ctx, &vert, &frag)?;
        collect_attrib_locations(
            self.ctx,
            &mut self.program.attrib_locations,
            &program,
            &attributes,
        )?;
        setup_uniform_block(self.ctx, &program)?;
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

fn collect_attrib_locations<'a>(
    ctx: &WebGl2RenderingContext,
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

fn setup_uniform_block<'a>(
    ctx: &WebGl2RenderingContext,
    program: &WebGlProgram,
) -> Result<(), String> {
    let index = ctx.get_uniform_block_index(program, "uniforms_");
    ctx.uniform_block_binding(program, index, 0);
    Ok(())
}

// macro_rules! attrib_definition {
//     (
//         $namespace:ident
//         ---attributes(0, $($v_offset:literal),*)
//         $(layout (location = $v_loc:literal) in $v_type:ident $v_name:ident);*;
//         ---instances(0, $($i_offset:literal),*)
//         $(layout (location = $i_loc:literal) in $i_type:ident $i_name:ident);*;
//         ---uniforms
//         $(layout (std140) uniform $u_name:ident {
//             $($u_item_type:ident $u_item_name:ident);*
//         });*;
//     ) => {
//         mod $namespace {
//             type vec4 = [f32; 4];
//             type vec3 = [f32; 3];
//             type vec2 = [f32; 2];
//             pub static VERTEX_LAYOUT: &'static [u32] = &[0, $($v_offset,)*];
//             #[repr(C)]
//             pub struct Vertex {
//                 $($v_name: $v_type,)*
//             }
//             pub static INSTANCE_LAYOUT: &'static [u32] = &[0, $($i_offset,)*];
//             #[repr(C)]
//             pub struct Instance {
//                 $($i_name: $i_type,)*
//             }
//         }
//     };
// }

// attrib_definition!(
//     AA
//     ---attributes(0, 12)
//     layout (location = 0) in vec3 position;
//     layout (location = 1) in vec2 uv;
//     ---instances(0,)
//     ;
//     ---uniforms
//     ;
// );
