use ash::vk;

use crate::{compute_pass::ComputePassBuilder, descriptors::CreationReference, graphics_pass::{GraphicsPassBuilder, GraphicsPassDrawInfo}, layer::{self, PassDependency}, renderer_data::ResourceReference, shader::ShaderType, vertex_buffer::NoVertices, Renderer};

pub unsafe fn draw_to_screen<'a>(renderer: &Renderer, src_image_name: &str, dst_image_name: &str) -> (GraphicsPassBuilder<'a, NoVertices, u32>, PassDependency) {
    let draw_to_screen_frag_ref = CreationReference::Sampler(src_image_name.to_string());
    let draw_to_screen_pass_builder: GraphicsPassBuilder<'_, NoVertices, u32> = GraphicsPassBuilder::new()
        .vertex_shader("res/shaders/bin/draw_to_screen.vert.spv")
        .fragment_shader("res/shaders/bin/draw_to_screen.frag.spv")
        .fragment_descriptors(vec![draw_to_screen_frag_ref], &renderer.data)
        .draw_info(GraphicsPassDrawInfo::simple_vertex(6))
        .targets(renderer.get_images(&dst_image_name));

    // TODO: This should't be constant
    let dep = PassDependency {
        resource: ResourceReference::Image(renderer.data.get_image_refs(src_image_name)),
        src_access: vk::AccessFlags::SHADER_WRITE,
        src_stage: vk::PipelineStageFlags::COMPUTE_SHADER,
        src_shader: ShaderType::Compute,
        dst_access: vk::AccessFlags::SHADER_READ,
        dst_stage: vk::PipelineStageFlags::FRAGMENT_SHADER,
        dst_shader: ShaderType::Fragment,
    };

    (draw_to_screen_pass_builder, dep)
}