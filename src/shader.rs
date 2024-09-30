use std::fs::File;

use ash::util::read_spv;
use ash::vk;

use crate::device::Device;

#[derive(Copy, Clone)]
pub enum ShaderType {
    Compute,
    Vertex,
    Fragment,
}

pub struct Shader {
    pub module: vk::ShaderModule,
    pub flags: vk::ShaderStageFlags,
    pub bytecode: Vec<u32>,
}

impl Shader {
    pub unsafe fn new(d: &Device, path: &str, flags: vk::ShaderStageFlags) -> Shader {
        let mut shader_file = File::open(path).expect(format!("Error: Shader file at {path} doesn't exist").as_str());
        let bytecode = read_spv(&mut shader_file).expect("Error reading shader");
        let shader_ci = vk::ShaderModuleCreateInfo::builder().code(&bytecode);
        let module = d.device.create_shader_module(&shader_ci, None).expect("Error creating shader module");

        Shader {
            module,
            flags,
            bytecode,
        }
    }
}