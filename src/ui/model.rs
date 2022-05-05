use std::{ops::Deref, rc::Rc};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use super::{
    error::ModelError,
    geometry::GeometryName,
    material::{Material, MaterialName},
    mesh::{MeshBuf, MeshDescriptor},
    pipeline::NamedPipeline,
    resources::NamedHandle,
    wgpu_state::WgpuResourceLoader,
};

impl Ord for Model {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.pipeline.as_ref().name() != other.pipeline.as_ref().name() {
            self.pipeline
                .as_ref()
                .name()
                .cmp(&other.pipeline.as_ref().name())
        } else {
            self.mesh.name.cmp(&other.mesh.name)
        }
    }
}

impl Eq for Model {
    fn assert_receiver_is_total_eq(&self) {}
}

impl PartialOrd for Model {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self
            .pipeline
            .as_ref()
            .name()
            .partial_cmp(&other.pipeline.as_ref().name())
        {
            Some(core::cmp::Ordering::Equal) => Some(self.mesh.name.cmp(&other.mesh.name)),
            ord => return ord,
        }
    }
}

impl PartialEq for Model {
    fn eq(&self, other: &Self) -> bool {
        self.pipeline.as_ref().name() == other.pipeline.as_ref().name()
            && self.mesh.name == other.mesh.name
            && self.materials.len() == other.materials.len()
            && self
                .materials
                .iter()
                .fold("".to_string(), |acc, mat| format!("{} {}", acc, mat.name()))
                == other
                    .materials
                    .iter()
                    .fold("".to_string(), |acc, mat| format!("{} {}", acc, mat.name()))
    }
}

#[derive(Debug)]
pub struct Model {
    name: String,
    pipeline: Rc<NamedPipeline>,
    mesh: Rc<MeshBuf>,
    materials: Vec<Rc<dyn Material>>,
    //TODO: delete both
    /*instances: RefCell<Vec<I>>,
    instances_names: RefCell<Vec<String>>,*/
}

impl Model {
    pub fn new(name: String, pipeline: Rc<NamedPipeline>, mesh: Rc<MeshBuf>) -> Self {
        Self {
            name,
            pipeline,
            mesh,
            materials: vec![],
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    /*    pub fn instances_count(&self) -> u32 {
        self.instances.borrow().len() as u32
    }

    pub fn instances(&self) -> Vec<I> {
        self.instances.borrow().to_vec()
    }

    // set instance by name for this entity
    pub fn set_instance<S: AsRef<str>>(&self, name: S, instance: I) {
        let push_it = |name, instance: I| {
            self.instances.borrow_mut().push(instance);
            self.instances_names.borrow_mut().push(name);
        };
        let name = name.as_ref().to_string();
        if self.instances.borrow().len() == 0 {
            push_it(name, instance);
        } else {
            let item_index = self
                .instances_names
                .borrow()
                .iter()
                .position(|instance_name| instance_name == &name);
            match item_index {
                Some(index) => {
                    log::debug!("replace instance {} in entity", index);
                    log::debug!("instances len {}", self.instances.borrow().len());
                    dbg!(self.instances.borrow()[index]);
                    {
                        // self.instances.borrow_mut().push(instance);

                        let _ =
                            std::mem::replace(&mut self.instances.borrow_mut()[index], instance);
                    }
                    log::debug!("instances len {}", self.instances.borrow().len());
                    dbg!(self.instances.borrow()[index]);
                }
                None => {
                    log::debug!("add instance {} in entity {}", name.clone(), self.name());
                    push_it(name, instance);
                }
            }
        }
    }

    // rm instance by name for this entity
    pub fn _rm_instance(&self, name: String) {
        let item_index = self
            .instances_names
            .borrow()
            .iter()
            .position(|instance_name| instance_name == &name);
        match item_index {
            Some(index) => {
                self.instances.borrow_mut().remove(index);
                self.instances_names.borrow_mut().remove(index);
            }
            None => { /* should emit err */ }
        }
    }*/
}

/// # Describe a model.
///
/// ## Example:
///
/// ```
/// {
///     name: "pink_house",
///     mesh_desc: MeshDescriptor {
///         name: "house",
///         geometries: vec![
///             GeometryDescriptor { name: "window" },
///             GeometryDescriptor { name: "door" },
///             GeometryDescriptor { name: "wall" },
///         ],
///     }
///     geometries_materials: vec![
///         ("window", "glass"),
///         ("door", "wood"),
///         ("wall", "pink"),
///     ],
///     pipeline_name: "pipeline_1"
/// }
/// ```
///
#[derive(Deserialize, Serialize, Debug)]
pub struct ModelDescriptor {
    name: String,
    mesh_desc: MeshDescriptor,
    pub(crate) geometries_materials: Vec<(GeometryName, MaterialName)>,
    pipeline_name: String, // Pipeline descriptor...
}

impl ModelDescriptor {
    pub fn _new_(
        name: String,
        mesh_desc: MeshDescriptor,
        geometries_materials: Vec<(GeometryName, MaterialName)>,
        pipeline_name: String,
    ) -> Self {
        Self {
            name,
            mesh_desc,
            geometries_materials,
            pipeline_name,
        }
    }
}

impl WgpuResourceLoader for ModelDescriptor {
    type Output = Rc<Model>;

    fn load(&self, wgpu_state: &super::wgpu_state::WgpuState) -> Result<Self::Output> {
        let store = &wgpu_state.store;
        if store.contains_model(&self.name) {
            return Ok(store.get_model(&self.name).expect("Impossible error 1"));
        }
        let model_name = self.name.clone();
        let mesh_name = self.mesh_desc.name();
        let pipeline_name = self.pipeline_name.clone();
        // load mesh from store
        let mesh = store
            .get_mesh(&*mesh_name)
            .ok_or_else(|| ModelError::MeshNotFoundInStore {
                mesh: mesh_name.clone(),
                model: model_name.to_string(),
            })?;

        // load pipeline from store
        let pipeline = store.get_pipeline(&pipeline_name).ok_or_else(|| {
            ModelError::PipelineNotFoundInStore {
                model: model_name.clone(),
                pipeline: pipeline_name.clone(),
            }
        })?;

        let model = match (self.geometries_materials.len(), pipeline.needs_material()) {
            (0, false) => {
                // no material and pipeline does not use any
                let model = Model::new(model_name, pipeline, mesh.clone());
                Ok(model)
            }
            (mat_cnt, false) if mat_cnt != 0 => {
                // fount material whereas pipeline doesnt use any
                Err(anyhow!(ModelError::InvalidMaterialAndPipeline {
                    model: model_name.clone(),
                    pipeline: pipeline_name.clone(),
                    reason: "Pipeline does not expect material".to_string()
                }))
            }
            (mat_cnt, true) if mat_cnt != self.mesh_desc.count_geometries() => {
                Err(anyhow!(ModelError::InvalidMaterialCount {
                    model_name: model_name.clone(),
                    mesh_name: mesh_name.clone(),
                    descriptor_materials_count: mat_cnt,
                    model_geometries_count: self.mesh_desc.count_geometries(),
                }))
            }
            _ => {
                let mut model = Model::new(model_name.clone(), pipeline, mesh.clone());
                let geometries_names = self.mesh_desc.geometries_names();

                for (g_name, m_name) in &self.geometries_materials {
                    if !geometries_names.contains(&g_name) {
                        return Err(anyhow!(ModelError::MaterialNotSetForGeometry {
                            geometry: g_name.clone(),
                            model: model_name.clone(),
                        }));
                    }
                    let material = store.get_material(m_name.deref()).ok_or_else(|| {
                        ModelError::MaterialNotFoundInStore {
                            material: m_name.clone(),
                            model: model_name.clone(),
                        }
                    })?;
                    model.materials.push(material.clone());
                }

                Ok(model)
            }
        }?;

        let model = Rc::new(model);
        store.add_model(model.clone());
        Ok(model)
    }
}
