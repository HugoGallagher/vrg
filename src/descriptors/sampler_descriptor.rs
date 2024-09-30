use ash::vk;

use crate::core::Core;
use crate::device::Device;
use crate::image::{Image, ImageData};
use crate::sampler::Sampler;

pub struct SamplerDescriptor {
    pub samplers: Vec<Sampler>,
    pub data: Vec<ImageData>,
}

pub struct SamplerDescriptorBuilder {
    image_datas: Option<Vec<ImageData>>,
}

impl SamplerDescriptorBuilder {
    pub fn new() -> SamplerDescriptorBuilder {
        SamplerDescriptorBuilder {
            image_datas: None,
        }
    }

    pub fn images(&self, images: &Vec<Image>) -> SamplerDescriptorBuilder {
        let image_datas = images.iter().map(|image| { ImageData { image: image.image, view: image.view, layout: image.layout} }).collect();
        SamplerDescriptorBuilder {
            image_datas: Some(image_datas),
        }
    }

    pub unsafe fn build(&self, c: &Core, d: &Device, binding: u32, sets: &Vec<vk::DescriptorSet>) -> SamplerDescriptor {
        if self.image_datas.is_none() {
            panic!("Error: Sampler descriptor builder has no images");
        }
        
        let mut samplers = Vec::<Sampler>::new();

        for image_data in self.image_datas.as_ref().unwrap() {
            samplers.push(Sampler::new(c, d, image_data));
        }

        SamplerDescriptor::new(c, d, binding, self.image_datas.as_ref().unwrap(), &samplers, sets)
    }
}

impl SamplerDescriptor {
    pub unsafe fn new(c: &Core, d: &Device, binding: u32, images: &Vec<ImageData>, samplers: &Vec<Sampler>, sets: &Vec<vk::DescriptorSet>) -> SamplerDescriptor {
        let mut write_sets = Vec::<vk::WriteDescriptorSet>::new();

        for i in 0..samplers.len() {
            let image_is = [vk::DescriptorImageInfo::builder()
                .sampler(samplers[i as usize].sampler)
                .image_view(samplers[i as usize].view)
                .image_layout(samplers[i as usize].layout)
                .build()];

            let write_set = vk::WriteDescriptorSet::builder()
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .dst_binding(binding)
                .dst_set(sets[i as usize])
                .image_info(&image_is)
                .build();

            write_sets.push(write_set);
        }

        d.device.update_descriptor_sets(&write_sets, &[]);

        SamplerDescriptor {
            samplers: samplers.clone(),
            data: images.clone(),
        }
    }
}