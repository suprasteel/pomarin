use std::rc::Rc;
use std::sync::Arc;

use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::EventLoop;
use winit::{event::Event, window::Window};

use super::camera::{CameraSystem, CameraUpdater};
use super::event::{Emitter, PomarinEvent};
use super::instance::{InstanceRaw, InstancesSystem};
use super::light::{self, LightUniform};
use super::material::MaterialKind;
use super::pipeline::{
    create_colored_model_pipeline, create_light_pipeline, create_textured_model_pipeline,
    NamedPipeline,
};
use super::texture::{self, Texture};
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
    depth_texture: Texture,
}

impl ObjectsPass {
    pub fn new(wgpu: &WgpuState, _window: &Window, event_loop: &EventLoop<PomarinEvent>) -> Self {
        let emitter = Arc::new(Emitter::new(event_loop));

        // Define object
        // load object assets
        // create renderable object

        let mut instances_system: InstancesSystem<InstanceRaw> = InstancesSystem::new(&wgpu.device);
        let (light_bgl, light) = light::LightSystem::init(LightUniform::default(), &wgpu.device);

        let (camera_bgl, camera_system) = CameraSystem::init(&wgpu.device, CamCtrl::default());

        let depth_texture =
            texture::Texture::create_depth_texture(&wgpu.device, &wgpu.config, "depth_texture");

        let textured_model_pipeline = NamedPipeline::new(
            "textures_pipeline",
            create_textured_model_pipeline(&wgpu.device, &wgpu.config, &camera_bgl, &light_bgl),
            vec![MaterialKind::Texture],
        );
        let colored_model_pipeline = NamedPipeline::new(
            "colors_pipeline",
            create_colored_model_pipeline(&wgpu.device, &wgpu.config, &camera_bgl, &light_bgl),
            vec![MaterialKind::Color],
        );
        let light_pipeline = NamedPipeline::new(
            "light_pipeline",
            create_light_pipeline(&wgpu.device, &wgpu.config, &camera_bgl, &light_bgl),
            vec![],
        );
        wgpu.store.add_pipeline(Rc::new(textured_model_pipeline));
        wgpu.store.add_pipeline(Rc::new(colored_model_pipeline));
        wgpu.store.add_pipeline(Rc::new(light_pipeline));

        Self {
            emitter,
            depth_texture,
        }
    }

    pub(crate) fn resize(&mut self, wgpu_state: &WgpuState) {
        // self.projection.resize(size);
        self.depth_texture = texture::Texture::create_depth_texture(
            &wgpu_state.device,
            &wgpu_state.config,
            "depth_texture",
        );
    }

    pub(crate) fn handle_event(&mut self, ref event: PomarinEvent) {
        match event {
            PomarinEvent::SomeEvent => {
                log::info!(target: "event", "some user event");
            }
            _ => {}
        };
    }

    pub(crate) fn render(
        &mut self,
        wgpu: &WgpuState,
        window: &winit::window::Window,
        output_view: &wgpu::TextureView,
        mut encoder: wgpu::CommandEncoder,
    ) -> wgpu::CommandEncoder {
        todo!()
    }
}
