use shaderc::*;
use std::{borrow::Cow, fs::read_to_string};

use super::RenderContext; // bad form

trait ShaderUnwrap {
    fn shader_unwrap(self) -> CompilationArtifact;
}

impl ShaderUnwrap for Result<CompilationArtifact> {
    fn shader_unwrap(self) -> CompilationArtifact {
        match self {
            Ok(value) => value,
            Err(error) => match error {
                Error::CompilationError(_, error_text) => panic!("{}", error_text),
                error => panic!("{}", error),
            },
        }
    }
}

pub struct ShaderBundle<'a> {
    pub vertex: Cow<'a, [u32]>,
    pub fragment: Cow<'a, [u32]>,
}

impl<'a> ShaderBundle<'a> {
    pub fn from_path<S: Into<String>>(path: S) -> Self {
        let path = path.into();
        let source_frag =
            read_to_string(format!("shaders/{}.frag", path)).expect("unable to read shader file");
        let source_vert =
            read_to_string(format!("shaders/{}.vert", path)).expect("unable to read shader file");

        let mut opts = CompileOptions::new().unwrap();
        let mut compiler = Compiler::new().unwrap();

        opts.set_source_language(SourceLanguage::GLSL);
        opts.set_optimization_level(OptimizationLevel::Performance);
        opts.set_target_env(TargetEnv::Vulkan, EnvVersion::WebGPU as u32); // Platform compatibility issues will come from this. Needa force vulkan for every platform

        opts.set_include_callback(move |_, _, _, _| {
            todo!();
        }); // Panic on include

        let fragment = compiler
            .compile_into_spirv(
                &source_frag,
                ShaderKind::Fragment,
                &format!("shaders/{}.frag", path),
                "main",
                Some(&opts),
            )
            .shader_unwrap();

        let vertex = compiler
            .compile_into_spirv(
                &source_vert,
                ShaderKind::Vertex,
                &format!("shaders/{}.vert", path),
                "main",
                Some(&opts),
            )
            .shader_unwrap();

        Self {
            vertex: Cow::Owned(vertex.as_binary().to_owned()),
            fragment: Cow::Owned(fragment.as_binary().to_owned()),
        }
    }

    pub unsafe fn create_shader_module_spirv(
        self,
        context: &RenderContext,
    ) -> (wgpu::ShaderModule, wgpu::ShaderModule) {
        let shader_vertex =
            context
                .device
                .create_shader_module_spirv(&wgpu::ShaderModuleDescriptorSpirV {
                    label: Some("raytrace_vertex"),
                    source: self.vertex,
                });

        let shader_fragment =
            context
                .device
                .create_shader_module_spirv(&wgpu::ShaderModuleDescriptorSpirV {
                    label: Some("raytrace_fragment"),
                    source: self.fragment,
                });

        (shader_vertex, shader_fragment)
    }
}
