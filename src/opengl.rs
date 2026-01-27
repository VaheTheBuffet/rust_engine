use crate::renderer::Api;

pub struct GLinner {

}

impl GLinner {
    pub fn new(mut window: glfw::PWindow) -> GLinner {
        GLinner{}
    }
}

impl Api for GLinner {
    fn create_buffer(&self, buffer_info: u32) -> Result<u32, ()> {
        todo!()
    }

    fn create_pipeline(&self, pipeline_info: crate::renderer::PipelineInfo) -> Result<u32, ()> {
        todo!()
    }

    fn destroy_buffer(&self, buffer: u32) -> Result<(), ()> {
        todo!()
    }

    fn destroy_pipeline(&self, id: u32) -> Result<(), ()> {
        todo!()
    }

    fn draw(&self, start: i32, end: i32) {
        todo!()
    }

    fn draw_indexed(&self, start: i32, end: i32) {
        todo!()
    }
}