use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;

use anyhow::anyhow;
use winit::event_loop::EventLoop;
use winit::window::Window;

use crate::app::event::{Emitter, PomarinEvent};
use crate::render::config::assets::{AssetDescriptor, TryAsRef};
use crate::render::config::model::ModelDescriptor;
use crate::render::config::WgpuResourceLoader;
use crate::render::names::ModelName;
use crate::render::state::WgpuState;

use super::camera::{CameraSystem, OrbitController};
use super::draw_ext::DrawModel;
use super::instance::{InstanceRaw, InstancesSystem};
use super::light::{self, LightSystem, LightUniform};
use super::material::MaterialKind;
use super::model::Model;
use super::object::Object;
use super::pipeline::{
    create_colored_model_pipeline, create_light_pipeline, create_textured_model_pipeline,
    NamedPipeline,
};
use super::texture::{self, Texture};

/// A struct mapping the object and the model
/// The model is displayed based on the object data
pub struct LinkedObject {
    object: Object,
    model: Rc<Model>,
}

impl LinkedObject {
    fn name(&self) -> String {
        self.object.name()
    }
}

/// Scene initialisation and redrawing
pub struct ScenePass {
    _emitter: Arc<Emitter<PomarinEvent>>,
    depth_texture: Texture,
    objects: Vec<LinkedObject>,
    instances_system: InstancesSystem<InstanceRaw>,
    camera_system: CameraSystem<OrbitController>,
    light_system: LightSystem<LightUniform>, //TODO: rm useless trait/generic
    last_render_time: Instant,
}

impl ScenePass {
    pub fn new(wgpu: &WgpuState, _window: &Window, event_loop: &EventLoop<PomarinEvent>) -> Self {
        let _emitter = Arc::new(Emitter::new(event_loop));

        let mut z2 = Object::new("z2".to_string(), ModelName::from("texture_zod"));
        z2.set_position((10.0, 0.0, 10.0));

        let mut objects_desc = vec![];
        objects_desc.push(Object::new(
            "zodiac".to_string(),
            ModelName::from("color_zod"),
        ));
        objects_desc.push(z2);
        objects_desc.push(Object::new(
            "sea".to_string(),
            ModelName::from("sea_square"),
        ));
        objects_desc.push(Object::new(
            "surface".to_string(),
            ModelName::from("fake_terrain"),
        ));

        let mut instances_system: InstancesSystem<InstanceRaw> = InstancesSystem::new(&wgpu.device);
        let (light_bgl, light_system) =
            light::LightSystem::init(LightUniform::default(), &wgpu.device);

        let camera_controler = OrbitController::default();

        let (camera_bgl, camera_system) = CameraSystem::init(&wgpu.device, camera_controler);

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
        // TODO: terrain pipeline to colr according to height

        let mut objects = vec![];

        objects_desc.into_iter().for_each(|object| {
            wgpu.assets
                .find(object.model())
                .ok_or(anyhow!("obj asset model not found"))
                .and_then(|model: &AssetDescriptor| model.try_as_ref())
                .and_then(|zd: &ModelDescriptor| zd.load(wgpu))
                .and_then(|model| {
                    instances_system
                        .set_instances_raw(vec![InstanceRaw::from(&object)], &wgpu.queue);
                    objects.push(LinkedObject { object, model });
                    Ok(())
                })
                .err()
                .iter()
                .for_each(|e| log::warn!("Failed while trying to load object: {}", e));
        });

        Self {
            _emitter,
            instances_system,
            depth_texture,
            objects,
            camera_system,
            light_system,
            last_render_time: Instant::now(),
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
        // TODO: receive objects here
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
        let now = std::time::Instant::now();
        let dt = now - self.last_render_time;
        self.last_render_time = now;
        self.update_instance_system(wgpu);
        self.camera_system.update(&wgpu.queue, dt);
        self.light_system.update(&wgpu.queue, dt);
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
                            r: 0.0,
                            g: 0.05,
                            b: 0.1,
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
                render_pass.set_vertex_buffer(1, self.instances_system.buffer().slice(..));

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
