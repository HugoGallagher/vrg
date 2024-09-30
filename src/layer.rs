use ash::vk;

use crate::{compute_pass::ComputePass, core::Core, renderer_data::{RendererData, ResourceReference}, semaphore::Semaphore, shader::ShaderType, util::graph::Graph, vertex_buffer::{self, VertexAttributes}};
use crate::device::Device;
use crate::commands::Commands;
use crate::graphics_pass::GraphicsPass;

#[derive(Copy, Clone)]
pub enum LayerExecution {
    Main,
    Async,
}

#[derive(Copy, Clone)]
pub enum PassType {
    Compute,
    Graphics,
}

#[derive(Copy, Clone)]
pub struct PassRef {
    pass_type: PassType,
    pub index: usize,
}

#[derive(Copy, Clone)]
pub struct PassDependency {
    pub resource: ResourceReference,

    pub src_access: vk::AccessFlags,
    pub src_stage: vk::PipelineStageFlags,
    pub src_shader: ShaderType,
    
    pub dst_access: vk::AccessFlags,
    pub dst_stage: vk::PipelineStageFlags,
    pub dst_shader: ShaderType,
}

#[derive(Copy, Clone)]
pub struct LayerDependencyInfo {
    pub stage: vk::PipelineStageFlags,
}

pub trait Pass {}

pub struct LayerSubmitInfo {
    pub wait_semaphores: Vec<vk::Semaphore>,
    pub wait_stages: Vec<vk::PipelineStageFlags>,
    pub signal_semaphores: Vec<vk::Semaphore>,
    pub command_buffers: Vec<vk::CommandBuffer>,
    pub queue: vk::Queue,
    pub fence: vk::Fence,
    pub submit_i: vk::SubmitInfo,
}

pub struct Layer {
    pub count: usize,

    pub commands: Commands,
    pub exec: LayerExecution,

    pub graphics_passes: Vec<GraphicsPass>,
    pub compute_passes: Vec<ComputePass>,

    pub pass_graph: Graph<PassRef, Option<PassDependency>>,

    pub root_pass: String,

    pub semaphore: Semaphore,

    pub present: bool,
}

impl Layer {
    pub unsafe fn new(c: &Core, d: &Device, count: usize, present: bool, exec: LayerExecution) -> Layer {
        let commands = Commands::new(d, d.get_queue(exec).1, count, false);
        let semaphore = Semaphore::new(d);

        Layer {
            count,
            commands,
            exec,
            graphics_passes: Vec::new(),
            compute_passes: Vec::new(),
            pass_graph: Graph::new(),
            root_pass: String::new(),
            semaphore,
            present,
        }
    }

    pub unsafe fn add_compute_pass(&mut self, name: &str, pass: ComputePass) {
        self.compute_passes.push(pass);
        self.pass_graph.add_node(name, PassRef { pass_type: PassType::Compute, index: self.compute_passes.len() - 1 });
    }

    pub unsafe fn add_graphics_pass(&mut self, name: &str, pass: GraphicsPass) {
        self.graphics_passes.push(pass);
        self.pass_graph.add_node(name, PassRef { pass_type: PassType::Graphics, index: self.graphics_passes.len() - 1 });
    }

    pub fn add_pass_dependency(&mut self, src_name: &str, dst_name: &str, dep: Option<PassDependency>) {
        self.pass_graph.add_edge(src_name, dst_name, dep);
    }

    pub fn set_root_path(&mut self, name: &str) {
        self.root_pass = name.to_string();
    }

    pub fn get_compute_pass(&self, name: &str) -> &ComputePass {
        &self.compute_passes[self.pass_graph.get_node(name).data.index]
    }

    pub fn get_graphics_pass(&self, name: &str) -> &GraphicsPass {
        &self.graphics_passes[self.pass_graph.get_node(name).data.index]
    }

    pub fn get_compute_pass_mut(&mut self, name: &str) -> &mut ComputePass {
        &mut self.compute_passes[self.pass_graph.get_node(name).data.index]
    }

    pub fn get_graphics_pass_mut(&mut self, name: &str) -> &mut GraphicsPass {
        &mut self.graphics_passes[self.pass_graph.get_node(name).data.index]
    }

    pub unsafe fn fill_compute_push_constant<T>(&mut self, name: &str, data: &T) {
        self.get_compute_pass_mut(name).push_constant.as_mut().expect("Error: Graphics pass has no vertex push constant to fill").set_data(data);
    }

    pub unsafe fn fill_vertex_push_constant<T>(&mut self, name: &str, data: &T) {
        self.get_graphics_pass_mut(name).vertex_push_constant.as_mut().expect("Error: Graphics pass has no vertex push constant to fill").set_data(data);
    }

    pub unsafe fn fill_fragment_push_constant<T>(&mut self, name: &str, data: &T) {
        self.get_graphics_pass_mut(name).fragment_push_constant.as_mut().expect("Error: Graphics pass has no fragment push constant to fill").set_data(data);
    }

    pub unsafe fn update_vertex_buffer<T: VertexAttributes, U>(&mut self, name: &str, c: &Core, d: &Device, verts: Option<&Vec<T>>, indices: Option<&Vec<U>>) {
        self.get_graphics_pass_mut(name).update_vertex_buffer(c, d, verts, indices)
    }

    pub unsafe fn record_one(&self, d: &Device, resources: &RendererData, i: usize, present_index: usize) {
        let mut dependencies = self.pass_graph.breadth_first_backwards(&self.root_pass);
        dependencies.reverse();

        self.commands.record_one(d, i, |b| {
            for dependency in &dependencies {
                let pass_ref = dependency.data;

                match pass_ref.pass_type {
                    PassType::Compute => {
                        let pass = &self.compute_passes[pass_ref.index];
                        
                        d.device.cmd_bind_pipeline(b, vk::PipelineBindPoint::COMPUTE, pass.pipeline.pipeline);
        
                        if pass.push_constant.is_some() {
                            d.device.cmd_push_constants(b, pass.pipeline.pipeline_layout, pass.push_constant.as_ref().unwrap().stage, 0, &pass.push_constant.as_ref().unwrap().data);
                        }
        
                        if pass.descriptors.is_some() {
                            let descriptors = pass.descriptors.as_ref().unwrap();
                            descriptors.bind(d, &b, vk::PipelineBindPoint::COMPUTE, &pass.pipeline.pipeline_layout, i);
                        }
        
                        d.device.cmd_dispatch(b, pass.dispatch_info.x, pass.dispatch_info.y, pass.dispatch_info.z);
                    },
                    PassType::Graphics => {
                        let pass = &self.graphics_passes[pass_ref.index];

                        let render_pass_bi = vk::RenderPassBeginInfo::builder()
                            .render_pass(pass.pipeline.render_pass)
                            .framebuffer(pass.framebuffers[present_index].framebuffer)
                            .render_area(pass.target_rect)
                            .clear_values(&pass.clear_values);

                        if let Some(push_constant) = &pass.vertex_push_constant {
                            d.device.cmd_push_constants(b, pass.pipeline.pipeline_layout, push_constant.stage, 0, &push_constant.data);
                        }

                        if let Some(push_constant) = &pass.fragment_push_constant {
                            d.device.cmd_push_constants(b, pass.pipeline.pipeline_layout, push_constant.stage, 0, &push_constant.data);
                        }

                        if let Some(descriptors) = &pass.vertex_descriptors {
                            descriptors.bind(d, &b, vk::PipelineBindPoint::GRAPHICS, &pass.pipeline.pipeline_layout, i);
                        }

                        if let Some(descriptors) = &pass.fragment_descriptors {
                            descriptors.bind(d, &b, vk::PipelineBindPoint::GRAPHICS, &pass.pipeline.pipeline_layout, i);
                        }
                        
                        d.device.cmd_begin_render_pass(b, &render_pass_bi, vk::SubpassContents::INLINE);

                        d.device.cmd_bind_pipeline(b, vk::PipelineBindPoint::GRAPHICS, pass.pipeline.pipeline);
                        
                        d.device.cmd_set_viewport(b, 0, &[pass.pipeline.viewport]);
                        d.device.cmd_set_scissor(b, 0, &[pass.pipeline.scissor]);

                        if let Some(vbw) = pass.vertex_buffer.as_ref() {
                            if let Some(vb) = vbw.vertex_buffer {
                                d.device.cmd_bind_vertex_buffers(b, 0, &[vb.buffer], &[0]);
                            }

                            if pass.indexed {
                                d.device.cmd_bind_index_buffer(b, vbw.index_buffer.unwrap().buffer, 0, vbw.index_size.unwrap());
                            }
                        }
                        
                        if pass.indexed {
                            pass.draw_infos.iter().for_each(|draw_info| {
                                d.device.cmd_draw_indexed(b, draw_info.index_count, draw_info.instance_count, draw_info.first_vertex, draw_info.vertex_offset, draw_info.first_instance);
                            });

                        } else {
                            pass.draw_infos.iter().for_each(|draw_info| {
                                d.device.cmd_draw(b, draw_info.vertex_count, draw_info.instance_count, draw_info.first_vertex, draw_info.first_instance);
                            });
                        }

                        d.device.cmd_end_render_pass(b);
                    }
                }

                let dependant_edges = self.pass_graph.get_next_edges(&dependency.name);

                for dependant_edge in dependant_edges {
                    if let Some(dependant_info) = dependant_edge.info {
                        let mut memory_barriers = Vec::<vk::MemoryBarrier>::new();
                        let mut buffer_memory_barriers = Vec::<vk::BufferMemoryBarrier>::new();
                        let mut image_memory_barriers = Vec::<vk::ImageMemoryBarrier>::new();

                        match dependant_info.resource {
                            ResourceReference::Buffer(index) => {
                                let memory_barrier = vk::MemoryBarrier::builder()
                                    .src_access_mask(dependant_info.src_access)
                                    .dst_access_mask(dependant_info.dst_access)
                                    .build();

                                memory_barriers.push(memory_barrier);
                            },
                            ResourceReference::Image(index) => {
                                let subresource_range = vk::ImageSubresourceRange::builder()
                                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                                    .layer_count(1)
                                    .level_count(1)
                                    .build();

                                let image = resources.get_images_from_ref(index)[i];

                                let image_memory_barrier = vk::ImageMemoryBarrier::builder()
                                    .src_access_mask(dependant_info.src_access)
                                    .dst_access_mask(dependant_info.dst_access)
                                    .old_layout(image.layout)
                                    .new_layout(image.layout)
                                    .image(image.image)
                                    .subresource_range(subresource_range)
                                    .src_queue_family_index(d.get_queue(self.exec).1)
                                    .dst_queue_family_index(d.get_queue(self.exec).1)
                                    .build();

                                image_memory_barriers.push(image_memory_barrier);
                            }
                        }

                        d.device.cmd_pipeline_barrier(b, dependant_info.src_stage, dependant_info.dst_stage, vk::DependencyFlags::empty(), &memory_barriers, &buffer_memory_barriers, &image_memory_barriers);
                    }
                }
            }
        })
    }
}