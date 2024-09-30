use ash::vk;

use crate::core::Core;
use crate::device::Device;
use crate::image::ImageData;

// TODO: Finish
#[derive(Clone)]
pub struct SamplerBuilder {
    pub address_mode: vk::SamplerAddressMode,
    pub mag_filer: vk::Filter,
    pub min_filter: vk::Filter,
    pub flags: vk::SamplerCreateFlags,
}

#[derive(Copy, Clone)]
pub struct Sampler {
    pub sampler: vk::Sampler,
    pub view: vk::ImageView,
    pub layout: vk::ImageLayout,
}

impl Sampler {
    pub unsafe fn new(c: &Core, d: &Device, img: &ImageData) -> Sampler {
        let sampler_ci = vk::SamplerCreateInfo::builder()
            .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_BORDER)
            .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_BORDER)
            .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_BORDER)
            .mag_filter(vk::Filter::NEAREST)
            .min_filter(vk::Filter::NEAREST);
        
        let sampler = d.device.create_sampler(&sampler_ci, None).unwrap();

        Sampler {
            sampler,
            view: img.view,
            layout: img.layout,
        }
    }
}