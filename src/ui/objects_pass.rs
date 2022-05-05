use std::rc::Rc;
use std::sync::Arc;

use anyhow::anyhow;
use winit::event_loop::EventLoop;
use winit::window::Window;

use crate::settings::assets::{AssetDescriptor, TryAsRef};

use super::camera::{CameraSystem, CameraUpdater};
use super::event::{Emitter, PomarinEvent};
use super::instance::{InstanceRaw, InstancesSystem};
use super::light::{self, LightSystem, LightUniform};
use super::material::MaterialKind;
use super::model::{Model, ModelDescriptor, ModelName};
use super::object::Object;
use super::pipeline::{
    create_colored_model_pipeline, create_light_pipeline, create_textured_model_pipeline,
    NamedPipeline,
};
use super::rpass::DrawModel;
use super::texture::{self, Texture};
use super::wgpu_state::{WgpuResourceLoader, WgpuState};

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
    objects: Vec<LinkedObject>,
    instances_system: InstancesSystem<InstanceRaw>,
    camera_system: CameraSystem<CamCtrl>,
    light_system: LightSystem<LightUniform>, //TODO: rm useless trait/generic
}

pub struct LinkedObject {
    object: Object,
    model: Rc<Model>,
}

impl LinkedObject {
    fn name(&self) -> String {
        self.object.name()
    }
}

impl ObjectsPass {
    pub fn new(wgpu: &WgpuState, _window: &Window, event_loop: &EventLoop<PomarinEvent>) -> Self {
        let emitter = Arc::new(Emitter::new(event_loop));

        // Define object
        // load object assets
        // create renderable object
        let object = Object::new("test".to_string(), ModelName::from("zodiac"));

        let mut instances_system: InstancesSystem<InstanceRaw> = InstancesSystem::new(&wgpu.device);
        let (light_bgl, light_system) =
            light::LightSystem::init(LightUniform::default(), &wgpu.device);

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

        let mut objects = vec![];

        wgpu.assets
            .find(object.model())
            .ok_or(anyhow!("obj asset model not found"))
            .and_then(|model: &AssetDescriptor| model.try_as_ref())
            .and_then(|zd: &ModelDescriptor| zd.load(wgpu))
            .and_then(|model| {
                instances_system.set_instances_raw(vec![InstanceRaw::from(&object)], &wgpu.queue);
                objects.push(LinkedObject { object, model });
                Ok(())
            })
            .err()
            .iter()
            .for_each(|e| log::warn!("Trying to load text object: {}", e));

        Self {
            emitter,
            instances_system,
            depth_texture,
            objects,
            camera_system,
            light_system,
        }
    }

    fn update_instance_system(&mut self, wgpu: &WgpuState) {
        let mut instances = vec![];
        //let mut i = 0;
        for o in &self.objects {
            log::debug!("Object: {:?}", o.name());
            instances.push(InstanceRaw::from(&o.object));
            //i += 1;
        }
        self.instances_system
            .set_instances_raw(instances, &wgpu.queue);
        log::debug!("total instances count : {}", self.instances_system.count());
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
        _window: &winit::window::Window,
        output_view: &wgpu::TextureView,
        mut encoder: wgpu::CommandEncoder,
    ) -> wgpu::CommandEncoder {
        self.update_instance_system(wgpu);
        let objects = self.objects.iter().map(|o| &o.model).collect();
        //
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &output_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            {
                // instances of entities
                // entities are put in a vec
                // an instances buffer is made from those entities and their instances
                // drawing is done by draw for each entity instance mapped with the correct
                // instance bytes in the instance buffer
                render_pass.set_vertex_buffer(1, self.instances_system.buffer().slice(..));

                // log::debug!("DRAW");
                render_pass.draw_models(
                    objects,
                    &self.camera_system.bind_group,
                    &self.light_system.bind_group,
                );
            }
        }
        encoder
    }
}
