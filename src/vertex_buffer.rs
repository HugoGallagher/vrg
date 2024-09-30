use std::cmp::{max, min};
use std::{ffi::c_void, mem};

use ash::vk;

use crate::{core::Core, math::vec::Vec4};
use crate::device::Device;
use crate::buffer::{self, Buffer, BufferBuilder};

pub struct VertexAttribute {
    pub format: vk::Format,
    pub offset: usize,
}

pub trait VertexAttributes {
    fn get_attribute_data() -> Vec<VertexAttribute>;
}

pub struct NoVertices {}

impl VertexAttributes for NoVertices {
    fn get_attribute_data() -> Vec<VertexAttribute> {
        vec![]
    }
}

impl VertexAttributes for Vec4 {
    fn get_attribute_data() -> Vec<VertexAttribute> {
        vec![
            VertexAttribute { format: vk::Format::R32G32B32A32_SFLOAT, offset: 0 },
        ]
    }
}

pub struct VertexBuffer {
    pub binding_desc: vk::VertexInputBindingDescription,
    pub attrib_descs: Vec<vk::VertexInputAttributeDescription>,
    pub vertex_buffer: Option<Buffer>,
    pub index_buffer: Option<Buffer>,
    pub buffer_memory_flags: vk::MemoryPropertyFlags,
    pub resizable: bool,
    pub index_size: Option<vk::IndexType>,
}

impl VertexBuffer {
    pub unsafe fn new<T: VertexAttributes, U>(c: &Core, d: &Device, verts: Option<&Vec<T>>, indices: Option<&Vec<U>>, resizable: bool) -> VertexBuffer {
        let binding_desc = vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(std::mem::size_of::<T>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build();

        let vertex_attribs = T::get_attribute_data();
        let mut attrib_descs: Vec<vk::VertexInputAttributeDescription> = Vec::with_capacity(vertex_attribs.len());

        for (i, a) in vertex_attribs.iter().enumerate() {
            attrib_descs.push(vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(i as u32)
                .format(a.format)
                .offset(a.offset as u32)
                .build());
        }

        let buffer_memory_flags = match resizable {
            true => vk::MemoryPropertyFlags::HOST_VISIBLE,
            false => vk::MemoryPropertyFlags::DEVICE_LOCAL,
        };

        let vertex_buffer = if let Some(vs) = verts {
            let buffer_builder = BufferBuilder::new()
                .size(mem::size_of::<T>() * vs.len())
                .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
                .sharing_mode(vk::SharingMode::EXCLUSIVE)
                .properties(buffer_memory_flags);

            Some(buffer_builder.build_with_data(c, d, vs.as_ptr() as *const c_void))
        } else {
            None
        };

        let (index_buffer, index_size) = if let Some(is) = indices {
            let index_buffer_builder = BufferBuilder::new()
                .size(mem::size_of::<U>() * is.len())
                .usage(vk::BufferUsageFlags::INDEX_BUFFER)
                .sharing_mode(vk::SharingMode::EXCLUSIVE)
                .properties(buffer_memory_flags);

            (Some(match resizable {
                true => index_buffer_builder.build(c, d),
                false => index_buffer_builder.build_with_data(c, d, is.as_ptr() as *const c_void),
            }), Some(match mem::size_of::<U>() {
                2 => vk::IndexType::UINT16,
                4 => vk::IndexType::UINT32,
                _ => panic!("Error: incompatible type of index buffer"),
            }))
        } else {
            (None, None)
        };

        VertexBuffer {
            binding_desc,
            attrib_descs,
            vertex_buffer,
            index_buffer,
            buffer_memory_flags,
            resizable,
            index_size,
        }
    }

    // TODO: This shouldn't create new buffers
    pub unsafe fn update<T: VertexAttributes, U>(&mut self, c: &Core, d: &Device, verts: Option<&Vec<T>>, indices: Option<&Vec<U>>) {
        assert!(self.resizable, "Error: vertex buffer is not resizable");

        if let Some(vs) = verts {
            if let Some(vb) = self.vertex_buffer {
                //d.device.destroy_buffer(vb.buffer, None)
            }

            self.vertex_buffer = Some(BufferBuilder::new()
                .size(mem::size_of::<T>() * vs.len())
                .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
                .sharing_mode(vk::SharingMode::EXCLUSIVE)
                .properties(self.buffer_memory_flags)
                .build_with_data(c, d, vs.as_ptr() as *const c_void));
        }

        if let Some(is) = indices {
            if let Some(ib) = self.index_buffer {
                //d.device.destroy_buffer(ib.buffer, None)
            }

            //assert!(self.index_buffer.is_some(), "Error: no index buffer is present to be updated");
            
            self.index_buffer = Some(BufferBuilder::new()
                .size(mem::size_of::<U>() * is.len())
                .usage(vk::BufferUsageFlags::INDEX_BUFFER)
                .sharing_mode(vk::SharingMode::EXCLUSIVE)
                .properties(self.buffer_memory_flags)
                .build_with_data(c, d, is.as_ptr() as *const c_void));

            self.index_size = Some(match mem::size_of::<U>() {
                2 => vk::IndexType::UINT16,
                4 => vk::IndexType::UINT32,
                _ => panic!("Error: incompatible type of index buffer"),
            });
        }
    }
}