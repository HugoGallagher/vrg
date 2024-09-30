use ash::vk;

use crate::descriptors::{Descriptors, DescriptorsBuilder};
use crate::{math::vec::Vec4, layer::Pass};
use crate::{core::Core, descriptors::CreationReference, renderer_data::RendererData};
use crate::device::Device;
use crate::vertex_buffer::{VertexBuffer, VertexAttributes};
use crate::push_constant::PushConstantBuilder;
use crate::graphics_pipeline::GraphicsPipeline;
use crate::framebuffer::Framebuffer;
use crate::push_constant::PushConstant;
use crate::image::Image;

#[derive(Copy, Clone)]
pub struct GraphicsPassDrawInfo {
    pub vertex_count: u32,
    pub index_count: u32,
    pub instance_count: u32,
    pub first_vertex: u32,
    pub first_instance: u32,
    pub vertex_offset: i32,
}

pub struct GraphicsPassBuilder<'a, T: VertexAttributes, U> {
    draw_infos: Option<Vec<GraphicsPassDrawInfo>>,
    targets: Option<Vec<Image>>,
    extent: Option<vk::Extent2D>,
    offset: Option<vk::Offset2D>,
    vs: Option<&'a str>,
    fs: Option<&'a str>,
    verts: Option<&'a Vec<T>>,
    vertex_indices: Option<&'a Vec<U>>,
    has_verts: bool,
    indexed: bool,
    resizable_vertex_buffer: bool,
    vertex_push_constant_builder: Option<PushConstantBuilder>,
    fragment_push_constant_builder: Option<PushConstantBuilder>,
    vertex_descriptors_builder: Option<DescriptorsBuilder>,
    fragment_descriptors_builder: Option<DescriptorsBuilder>,
    with_depth_buffer: bool,
    clear_col: Option<Vec4>,
}

pub struct GraphicsPass {
    pub vertex_push_constant: Option<PushConstant>,
    pub fragment_push_constant: Option<PushConstant>,

    pub vertex_buffer: Option<VertexBuffer>,
    pub vertex_descriptors: Option<Descriptors>,
    pub fragment_descriptors: Option<Descriptors>,

    pub pipeline: GraphicsPipeline,
    pub framebuffers: Vec<Framebuffer>,
    pub draw_infos: Vec<GraphicsPassDrawInfo>,
    pub indexed: bool,

    pub clear_values: Vec<vk::ClearValue>,
    pub target_rect: vk::Rect2D,
}

impl Pass for GraphicsPass {}

impl GraphicsPassDrawInfo {
    pub fn simple_vertex(vertex_count: usize) -> GraphicsPassDrawInfo {
        GraphicsPassDrawInfo {
            vertex_count: vertex_count as u32,
            index_count: 0,
            instance_count: 1,
            first_vertex: 0,
            first_instance: 0,
            vertex_offset: 0,
        }
    }

    pub fn simple_indexed(vertex_count: usize, index_count: usize) -> GraphicsPassDrawInfo {
        GraphicsPassDrawInfo {
            vertex_count: vertex_count as u32,
            index_count: index_count as u32,
            instance_count: 1,
            first_vertex: 0,
            first_instance: 0,
            vertex_offset: 0,
        }
    }

    pub fn instanced_vertex(vertex_count: usize, instance_count: usize) -> GraphicsPassDrawInfo {
        GraphicsPassDrawInfo {
            vertex_count: vertex_count as u32,
            index_count: 0,
            instance_count: instance_count as u32,
            first_vertex: 0,
            first_instance: 0,
            vertex_offset: 0,
        }
    }

    pub fn instanced_indexed(vertex_count: usize, index_count: usize, instance_count: usize) -> GraphicsPassDrawInfo {
        GraphicsPassDrawInfo {
            vertex_count: vertex_count as u32,
            index_count: index_count as u32,
            instance_count: instance_count as u32,
            first_vertex: 0,
            first_instance: 0,
            vertex_offset: 0,
        }
    }

    pub fn simple_empty() -> GraphicsPassDrawInfo {
        GraphicsPassDrawInfo {
            vertex_count: 0,
            index_count: 0,
            instance_count: 1,
            first_vertex: 0,
            first_instance: 0,
            vertex_offset: 0,
        }
    }
}

impl <'a, T: VertexAttributes, U> GraphicsPassBuilder<'a, T, U> {
    pub fn new() -> GraphicsPassBuilder<'a, T, U> {
        GraphicsPassBuilder {
            draw_infos: None,
            targets: None,
            extent: None,
            offset: None,
            vs: None,
            fs: None,
            verts: None,
            vertex_indices: None,
            has_verts: false,
            indexed: false,
            resizable_vertex_buffer: false,
            vertex_push_constant_builder: None,
            fragment_push_constant_builder: None,

            vertex_descriptors_builder: None,
            fragment_descriptors_builder: None,
            with_depth_buffer: false,
            clear_col: None,
        }
    }

    pub fn draw_info(mut self, draw_info: GraphicsPassDrawInfo) -> GraphicsPassBuilder<'a, T, U> {
        self.draw_infos = Some(vec![draw_info]);

        self
    }

    pub fn draw_infos(mut self, draw_infos: Vec<GraphicsPassDrawInfo>) -> GraphicsPassBuilder<'a, T, U> {
        self.draw_infos = Some(draw_infos);

        self
    }

    pub fn targets(mut self, targets: &Vec<Image>) -> GraphicsPassBuilder<'a, T, U> {
        self.targets = Some(targets.clone());

        self
    }

    pub fn extent(mut self, extent: vk::Extent2D) -> GraphicsPassBuilder<'a, T, U> {
        self.extent = Some(extent);

        self
    }

    pub fn offset(mut self, offset: vk::Offset2D) -> GraphicsPassBuilder<'a, T, U> {
        self.offset = Some(offset);

        self
    }

    pub fn vertex_shader(mut self, vs: &'a str) -> GraphicsPassBuilder<'a, T, U> {
        self.vs = Some(vs);

        self
    }

    pub fn fragment_shader(mut self, fs: &'a str) -> GraphicsPassBuilder<'a, T, U> {
        self.fs = Some(fs);

        self
    }

    pub fn verts(mut self, verts: &'a Vec<T>) -> GraphicsPassBuilder<'a, T, U> {
        self.has_verts = true;
        self.verts = Some(verts);

        self
    }

    pub fn vertex_indices(mut self, vertex_indices: &'a Vec<U>) -> GraphicsPassBuilder<'a, T, U> {
        self.indexed = true;
        self.vertex_indices = Some(vertex_indices);

        self
    }

    pub fn has_verts(mut self) -> GraphicsPassBuilder<'a, T, U> {
        self.has_verts = true;

        self
    }

    pub fn indexed(mut self) -> GraphicsPassBuilder<'a, T, U> {
        self.indexed = true;

        self
    }

    pub fn resizable_vertex_buffer(mut self) -> GraphicsPassBuilder<'a, T, U> {
        self.resizable_vertex_buffer = true;

        self
    }

    pub fn vertex_push_constant<V>(mut self) -> GraphicsPassBuilder<'a, T, U> {
        self.vertex_push_constant_builder = Some(PushConstantBuilder::new().stage(vk::ShaderStageFlags::VERTEX).size(std::mem::size_of::<V>()));

        self
    }

    pub fn fragment_push_constant<V>(mut self) -> GraphicsPassBuilder<'a, T, U> {
        self.fragment_push_constant_builder = Some(PushConstantBuilder::new().stage(vk::ShaderStageFlags::FRAGMENT).size(std::mem::size_of::<V>()));

        self
    }

    pub fn vertex_descriptors_builder(mut self, descriptors_builder: DescriptorsBuilder) -> GraphicsPassBuilder<'a, T, U> {
        self.vertex_descriptors_builder = Some(descriptors_builder.stage(vk::ShaderStageFlags::VERTEX));

        self
    }

    pub fn fragment_descriptors_builder(mut self, descriptors_builder: DescriptorsBuilder) -> GraphicsPassBuilder<'a, T, U> {
        self.fragment_descriptors_builder = Some(descriptors_builder.stage(vk::ShaderStageFlags::FRAGMENT));

        self
    }

    pub fn vertex_descriptors(mut self, create_refs: Vec<CreationReference>, data: &RendererData) -> GraphicsPassBuilder<'a, T, U> {
        let mut descriptors_builder = DescriptorsBuilder::new()
            .stage(vk::ShaderStageFlags::VERTEX)
            .count(data.count);

        for create_ref in create_refs {
            match create_ref {
                CreationReference::Uniform(name) => { descriptors_builder = descriptors_builder.add_uniform_simple(data.get_buffers(&name)); },
                CreationReference::Storage(name) => { descriptors_builder = descriptors_builder.add_storage_simple(data.get_buffers(&name)); },
                CreationReference::Image(name) => { descriptors_builder = descriptors_builder.add_image_simple(data.get_images(&name)); },
                CreationReference::Sampler(name) => { descriptors_builder = descriptors_builder.add_sampler_simple(data.get_images(&name)); },
            }
        }

        self.vertex_descriptors_builder = Some(descriptors_builder);
        
        self
    }

    pub fn fragment_descriptors(mut self, create_refs: Vec<CreationReference>, data: &RendererData) -> GraphicsPassBuilder<'a, T, U> {
        let mut descriptors_builder = DescriptorsBuilder::new()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .count(data.count);

        for create_ref in create_refs {
            match create_ref {
                CreationReference::Uniform(name) => { descriptors_builder = descriptors_builder.add_uniform_simple(data.get_buffers(&name)); },
                CreationReference::Storage(name) => { descriptors_builder = descriptors_builder.add_storage_simple(data.get_buffers(&name)); },
                CreationReference::Image(name) => { descriptors_builder = descriptors_builder.add_image_simple(data.get_images(&name)); },
                CreationReference::Sampler(name) => { descriptors_builder = descriptors_builder.add_sampler_simple(data.get_images(&name)); },
            }
        }

        self.fragment_descriptors_builder = Some(descriptors_builder);
        
        self
    }

    pub fn with_depth_buffer(mut self) -> GraphicsPassBuilder<'a, T, U> {
        self.with_depth_buffer = true;

        self
    }

    pub fn clear_col(mut self, clear_col: Vec4) -> GraphicsPassBuilder<'a, T, U> {
        self.clear_col = Some(clear_col);

        self
    }

    pub unsafe fn build(self, c: &Core, d: &Device) -> GraphicsPass {
        GraphicsPass::new(c, d, self.targets.expect("Error: Graphics pass builder has no targets"), self.extent, self.offset, self.verts, self.vertex_indices, self.has_verts, self.indexed, self.resizable_vertex_buffer, self.vertex_descriptors_builder, self.fragment_descriptors_builder, self.vertex_push_constant_builder, self.fragment_push_constant_builder, self.vs.expect("Error: Graphics pass builder has no vertex shader"), self.fs.expect("Error: Graphics pass builder has no fragment shader"), self.with_depth_buffer, self.clear_col, self.draw_infos.unwrap_or(vec![]))
    }
}

impl GraphicsPass {
    pub unsafe fn new<T: VertexAttributes, U>(c: &Core, d: &Device, targets: Vec<Image>, extent: Option<vk::Extent2D>, offset: Option<vk::Offset2D>, verts: Option<&Vec<T>>, indices: Option<&Vec<U>>, has_verts: bool, indexed: bool, resizable_vertex_buffer: bool, vertex_descriptors_builder: Option<DescriptorsBuilder>, fragment_descriptors_builder: Option<DescriptorsBuilder>, vertex_push_constant_builder: Option<PushConstantBuilder>, fragment_push_constant_builder: Option<PushConstantBuilder>, vs: &str, fs: &str, with_depth_buffer: bool, clear_col: Option<Vec4>, draw_infos: Vec<GraphicsPassDrawInfo>) -> GraphicsPass {
        let vertex_descriptors = match vertex_descriptors_builder {
            Some(de_b) => Some(de_b.build(c, d)),
            None => None
        };
        
        let fragment_descriptors = match fragment_descriptors_builder {
            Some(de_b) => Some(de_b.build(c, d)),
            None => None
        };

        let vertex_descriptor_set_layout = match vertex_descriptors.as_ref() {
            Some(de) => Some(de.set_layout),
            None => None
        };

        let fragment_descriptor_set_layout = match fragment_descriptors.as_ref() {
            Some(de) => Some(de.set_layout),
            None => None
        };

        let vertex_push_constant = match vertex_push_constant_builder {
            Some(builder) => Some(builder.build()),
            None => None
        };

        let fragment_push_constant = match fragment_push_constant_builder {
            Some(builder) => Some(builder.build()),
            None => None
        };

        let vertex_buffer = match has_verts {
            true => Some(VertexBuffer::new(c, d, verts, indices, resizable_vertex_buffer)),
            false => None
        };

        let target_extent = match extent {
            Some(e) => e,
            None => vk::Extent2D { width: targets[0].width, height: targets[0].height },
        };

        let offset = match offset {
            Some(o) => o,
            None => vk::Offset2D { x: 0, y: 0 },
        };

        let target_rect = vk::Rect2D { extent: target_extent, offset };
        
        let pipeline = GraphicsPipeline::new(c, d, target_rect, vertex_buffer.as_ref(), vertex_descriptor_set_layout, fragment_descriptor_set_layout, vertex_push_constant.as_ref(), fragment_push_constant.as_ref(), vs, fs, targets[0].layout, with_depth_buffer, clear_col.is_some());

        let framebuffers = Framebuffer::new_many(d, &pipeline, &targets, extent);

        let mut clear_values = Vec::<vk::ClearValue>::new();
        
        if let Some(col) = clear_col {
            clear_values.push(vk::ClearValue { color: vk::ClearColorValue { float32: [col.x, col.y, col.z, col.w] } });
        };

        if with_depth_buffer {
            clear_values.push(vk::ClearValue { depth_stencil: vk::ClearDepthStencilValue { depth: 1.0, stencil: 0 } });
        }

        GraphicsPass {
            vertex_push_constant,
            fragment_push_constant,
            vertex_descriptors,
            fragment_descriptors,
            vertex_buffer,
            pipeline,
            framebuffers,
            draw_infos,
            indexed,
            clear_values,
            target_rect,
        }
    }

    pub unsafe fn update_vertex_buffer<T: VertexAttributes, U>(&mut self, c: &Core, d: &Device, verts: Option<&Vec<T>>, indices: Option<&Vec<U>>) {
        assert!(self.vertex_buffer.is_some(), "Error: No vertex buffer present to be updated");

        if let Some(vb) = &mut self.vertex_buffer {
            vb.update(c, d, verts, indices);
        }
    }
}