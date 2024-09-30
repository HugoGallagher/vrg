use ash::vk;

use crate::{descriptors::CreationReference, graphics_pass::{GraphicsPassBuilder, GraphicsPassDrawInfo}, layer::{self, PassDependency}, renderer_data::ResourceReference, shader::ShaderType, vertex_buffer::NoVertices, Renderer};

pub unsafe fn draw_to_screen(renderer: &mut Renderer, layer_name: &str, final_pass_name: &str, image_name: &str) {
    let draw_to_screen_frag_ref = CreationReference::Sampler("render".to_string());
    let draw_to_screen_pass_builder: GraphicsPassBuilder<'_, NoVertices, u32> = GraphicsPassBuilder::new()
        .vertex_shader("res/shaders/bin/draw_to_screen.vert.spv")
        .fragment_shader("res/shaders/bin/draw_to_screen.frag.spv")
        .fragment_descriptors(vec![draw_to_screen_frag_ref], &renderer.data)
        .draw_info(GraphicsPassDrawInfo::simple_vertex(6))
        .targets(renderer.get_images("swapchain_image"));

    renderer.add_graphics_pass(layer_name, "draw_to_screen", draw_to_screen_pass_builder);
    renderer.get_layer_mut(layer_name).set_root_path("draw_to_screen");

    // TODO: This should't be constant
    let dep = PassDependency {
        resource: ResourceReference::Image(renderer.data.get_image_refs(image_name)),
        src_access: vk::AccessFlags::SHADER_WRITE,
        src_stage: vk::PipelineStageFlags::COMPUTE_SHADER,
        src_shader: ShaderType::Compute,
        dst_access: vk::AccessFlags::SHADER_READ,
        dst_stage: vk::PipelineStageFlags::FRAGMENT_SHADER,
        dst_shader: ShaderType::Fragment,
    };

    renderer.add_pass_dependency(layer_name, final_pass_name, "draw_to_screen", Some(dep));
}