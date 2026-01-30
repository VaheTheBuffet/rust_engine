use ash::{vk, Entry, self};
use crate::renderer::{Api, Pipeline, Buffer, PipelineInfo};

#[derive(Default)]
pub struct VKinner {
    instance: Option<ash::Instance>,
}

impl VKinner {
    pub fn new() -> VKinner {
        let application_info = vk::ApplicationInfo::default()
            .application_name(c"Minecraft Clone")
            .application_version(1)
            .api_version(vk::API_VERSION_1_3)
            .engine_name(c"Voxel Engine")
            .engine_version(1);

        let instance_create_info = vk::InstanceCreateInfo::default()
            .application_info(&application_info);
        
        let instance = unsafe {
            let entry = Entry::load().expect("failed to initialize vulkan loader");
            entry.create_instance(&instance_create_info, Some(&vk::AllocationCallbacks::default()))
                .expect("failed to create instance")
        };

        VKinner { 
            instance: Some(instance)
        }
    }
}

impl Api for VKinner {
    fn create_buffer(&self, buffer_info: u32) -> Result<Box<dyn Buffer>, ()> {
        todo!()
    }

    fn create_pipeline(&self, pipeline_info: PipelineInfo) -> Result<Box<dyn Pipeline>, ()> {
        todo!()
    }

    fn draw(&self, start: i32, end: i32) {
        
    }

    fn draw_indexed(&self, start: i32, end: i32) {
        todo!()
    }
}