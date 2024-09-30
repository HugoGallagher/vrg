use std::{collections::HashMap, os::raw::c_void};

use crate::{buffer::{Buffer, BufferBuilder}, image::{Image, ImageBuilder}, core::Core, device::Device};

#[derive(Copy, Clone)]
pub enum ResourceReference {
    Buffer(usize),
    Image(usize),
}

pub struct RendererData {
    pub count: usize,

    pub buffers: Vec<Vec<Buffer>>,
    pub images: Vec<Vec<Image>>,

    pub buffer_refs: HashMap<String, usize>,
    pub image_refs: HashMap<String, usize>,
}

impl RendererData {
    pub fn new(count: usize) -> RendererData {
        RendererData {
            count,
            buffers: Vec::new(),
            images: Vec::new(),
            buffer_refs: HashMap::new(),
            image_refs: HashMap::new(),
        }
    }

    pub unsafe fn add_buffers<T>(&mut self, c: &Core, d: &Device, name: &str, builder: BufferBuilder, data: Option<*const T>) {
        let new_buffers = if let Some(buffer_data) = data {
            builder.build_many_with_data(c, d, vec![buffer_data as *const c_void; self.count], self.count)
        } else {
            builder.build_many(c, d, self.count)
        };
        self.buffers.push(new_buffers);
        self.buffer_refs.insert(name.to_string(), self.buffers.len() - 1);
    }

    pub unsafe fn add_images(&mut self, c: &Core, d: &Device, name: &str, builder: ImageBuilder) {
        self.images.push(builder.build_many(c, d, self.count));
        self.image_refs.insert(name.to_string(), self.images.len() - 1);
    }

    pub unsafe fn add_images_raw(&mut self, c: &Core, d: &Device, name: &str, images: Vec<Image>) {
        assert!(images.len() == self.count, "Error: invalid number of images provided");

        self.images.push(images.clone());
        self.image_refs.insert(name.to_string(), self.images.len() - 1);
    }

    pub fn get_buffers(&self, name: &str) -> &Vec<Buffer> {
        &self.buffers[*self.buffer_refs.get(name).unwrap()]
    }

    pub fn get_images(&self, name: &str) -> &Vec<Image> {
        &self.images[*self.image_refs.get(name).unwrap()]
    }

    pub fn get_buffer_refs(&self, name: &str) -> usize {
        *self.buffer_refs.get(name).unwrap()
    }

    pub fn get_image_refs(&self, name: &str) -> usize {
        *self.image_refs.get(name).unwrap()
    }

    pub fn get_buffers_from_ref(&self, index: usize) -> &Vec<Buffer> {
        &self.buffers[index]
    }

    pub fn get_images_from_ref(&self, index: usize) -> &Vec<Image> {
        &self.images[index]
    }
}