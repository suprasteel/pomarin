use std::rc::Rc;
use std::sync::Arc;

use winit::event_loop::EventLoop;
use winit::window::Window;

use super::camera::{CameraSystem, CameraUpdater};
use super::event::{Emitter, PomarinEvent};
use super::instance::{InstanceRaw, InstancesSystem};
use super::light::{self, LightUniform};
use super::material::MaterialKind;
use super::pipeline::{
    create_colored_model_pipeline, create_light_pipeline, create_textured_model_pipeline,
    NamedPipeline,
};
use super::texture;
use super::wgpu_state::WgpuState;

pub struct CamCtrl {}

impl CameraUpdater for CamCtrl {
    fn update(
        &mut self,
        camera_uniform: super::camera::CameraUniform,
    ) -> super::camera::CameraUniform {
        camera_uniform
    }
}

impl Default for CamCtrl {
    fn default() -> Self {
        Self {}
    }
}

pub struct ObjectsPass {
    emitter: Arc<Emitter<PomarinEvent>>,
}

impl ObjectsPass {
    fn new(wgpu: &WgpuState, window: &Window, event_loop: &EventLoop<PomarinEvent>) -> Self {
        // load assets descriptions
        // load assets

        let emitter = Arc::new(Emitter::new(event_loop));

        let mut instances_system: InstancesSystem<InstanceRaw> = InstancesSystem::new(&wgpu.device);
        let (light_bgl, light) = light::LightSystem::init(LightUniform::default(), &wgpu.device);

        let (camera_bgl, camera_system) = CameraSystem::init(&wgpu.device, CamCtrl::default());

        let depth_texture =
            texture::Texture::create_depth_texture(&wgpu.device, &wgpu.config, "depth_texture");

        let res_dir = std::path::Path::new(env!("OUT_DIR")).join("res");

        let textured_model_pipeline = NamedPipeline::new(
            "textures_ppln",
            create_textured_model_pipeline(&wgpu.device, &wgpu.config, &camera_bgl, &light_bgl),
            vec![MaterialKind::Texture],
        );
        let colored_model_pipeline = NamedPipeline::new(
            "colors_ppln",
            create_colored_model_pipeline(&wgpu.device, &wgpu.config, &camera_bgl, &light_bgl),
            vec![MaterialKind::Color],
        );
        let light_pipeline = NamedPipeline::new(
            "light_ppln",
            create_light_pipeline(&wgpu.device, &wgpu.config, &camera_bgl, &light_bgl),
            vec![],
        );
        wgpu.store.add_pipeline(Rc::new(textured_model_pipeline));
        wgpu.store.add_pipeline(Rc::new(colored_model_pipeline));
        wgpu.store.add_pipeline(Rc::new(light_pipeline));
        Self { emitter }
    }
}
