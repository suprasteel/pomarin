use std::{cell::RefCell, ops::Deref, rc::Rc};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use super::{
    error::ModelError,
    geometry::GeometryName,
    instance::RawInstanceTrait,
    material::{Material, MaterialName},
    mesh::{Mesh, MeshDescriptor},
    pipeline::NamedPipeline,
    resources::NamedHandle,
    store::Store,
};

impl<I> Ord for Model<I>
where
    I: RawInstanceTrait,
{
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

impl<I> std::cmp::Eq for Model<I>
where
    I: RawInstanceTrait,
{
    fn assert_receiver_is_total_eq(&self) {}
}

impl<I> PartialOrd for Model<I>
where
    I: RawInstanceTrait,
{
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

impl<I> PartialEq for Model<I>
where
    I: RawInstanceTrait,
{
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
pub struct Model<I>
where
    I: RawInstanceTrait,
{
    name: String,
    pipeline: Rc<NamedPipeline>,
    mesh: Rc<Mesh>,
    materials: Vec<Rc<dyn Material>>,
    //TODO: delete both
    instances: RefCell<Vec<I>>,
    instances_names: RefCell<Vec<String>>,
}

impl<I> Model<I>
where
    I: RawInstanceTrait + std::fmt::Debug,
{
    pub fn new(name: String, pipeline: Rc<NamedPipeline>, mesh: Rc<Mesh>) -> Self {
        Self {
            name,
            pipeline,
            mesh,
            materials: vec![],
            instances: RefCell::new(vec![]),
            instances_names: RefCell::new(vec![]),
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn set_material<S: AsRef<str>>(
        &mut self,
        geometry_name: S,
        material: Rc<dyn Material>,
    ) -> Result<&Self> {
        if !self.pipeline.can_use(material.as_ref().kind()) {
            Err(ModelError::IncompatibleModelMaterial {
                model: self.name.to_string(),
                mesh: self.mesh.name.to_string(),
                material: material.as_ref().to_string(),
                reason: format!(
                    "material is supported by pipeline {} while model will be rendered by pipeline {}",
                    String::from(material.as_ref().kind()),
                    &self.pipeline.as_ref().name()
                    ),
            })?;
        }

        // geometry name exists for this mesh
        self.mesh
            .geometries
            .iter()
            .find(|g| g.name.eq(&geometry_name.as_ref().to_string()))
            .ok_or(ModelError::GeometryNotFound {
                geometry: geometry_name.as_ref().to_string(),
                model: self.name.clone(),
            })?;

        self.materials.push(material.clone());
        Ok(self)
    }

    pub fn instances_count(&self) -> u32 {
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
    }
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
    geometries_materials: Vec<(GeometryName, MaterialName)>,
    pipeline_name: String, // Pipeline descriptor...
}

impl ModelDescriptor {
    /// builds a model from resources available in store (resources have to be loaded beforehand)
    pub fn build_model_from_store_resources<I>(&self, store: &Store<I>) -> Result<Model<I>>
    where
        I: RawInstanceTrait + std::fmt::Debug,
    {
        //TODO: validate

        let model_name = self.name.clone();
        let mesh_name = self.mesh_desc.named_handle();
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

        match (self.geometries_materials.len(), pipeline.needs_material()) {
            (0, false) => {
                // no material and pipeline does not use any
                let model = Model::new(model_name, pipeline, mesh.clone());
                return Ok(model);
            }
            (mat_cnt, false) if mat_cnt != 0 => {
                // fount material whereas pipeline doesnt use any
                return Err(anyhow!(ModelError::InvalidMaterialAndPipeline {
                    model: model_name.clone(),
                    pipeline: pipeline_name.clone(),
                    reason: "Pipeline does not expect material".to_string()
                }));
            }
            (mat_cnt, true) if mat_cnt != self.mesh_desc.count_geometries() => {
                return Err(anyhow!(ModelError::InvalidMaterialCount {
                    model_name: model_name.clone(),
                    mesh_name: mesh_name.clone(),
                    descriptor_materials_count: mat_cnt,
                    model_geometries_count: self.mesh_desc.count_geometries(),
                }));
            }
            _ => {
                let mut model = Model::new(model_name.clone(), pipeline, mesh.clone());
                let geometries_names = self.mesh_desc.geometries_names();
                for geometry in geometries_names {
                    let geo_mat = &self
                        .geometries_materials
                        .iter()
                        .find(|gm| gm.0.eq(&geometry))
                        .ok_or_else(|| ModelError::MaterialNotSetForGeometry {
                            geometry: geometry.clone(),
                            model: model_name.clone(),
                        })?;
                    let material = store.get_material(&*geo_mat.1).ok_or_else(|| {
                        ModelError::MaterialNotFoundInStore {
                            material: geo_mat.1.clone(),
                            model: model_name.clone(),
                        }
                    })?;
                    model.set_material(geometry.deref(), material)?;
                }
                Ok(model)
            }
        }
    }
}
