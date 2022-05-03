use std::{cell::RefCell, rc::Rc};

use anyhow::{anyhow, Result};

use super::{
    error::ModelError, instance::RawInstanceTrait, material::Material, mesh::Mesh,
    pipeline::NamedPipeline, store::Store,
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

#[derive(Debug)]
pub struct ModelDescription {
    model_name: String,
    mesh_name: Option<String>,
    geometry_materials: Vec<(String, String)>,
    pipeline_name: Option<String>,
}

impl ModelDescription {
    /// describe a new model, starting by its name
    pub fn new<S: AsRef<str>>(name: S) -> Self {
        Self {
            model_name: name.as_ref().to_string(),
            mesh_name: None,
            geometry_materials: vec![],
            pipeline_name: None,
        }
    }

    /// set the mesh (geometries) to use
    pub fn mesh_name<S: AsRef<str>>(mut self, name: S) -> Self {
        self.mesh_name = Some(name.as_ref().to_string());
        self
    }

    /// set materials for geometries. For now, we can only use geometries that have the same
    /// materialkind
    pub fn add_geometry_material<S: AsRef<str>>(mut self, mesh: S, material: S) -> Self {
        self.geometry_materials
            .push((mesh.as_ref().to_string(), material.as_ref().to_string()));
        self
    }

    pub fn pipeline_name<S: AsRef<str>>(mut self, name: S) -> Self {
        self.pipeline_name = Some(name.as_ref().to_string());
        self
    }
}

impl<I> TryFrom<(ModelDescription, &Store<I>)> for Model<I>
where
    I: RawInstanceTrait + std::fmt::Debug,
{
    type Error = anyhow::Error;

    fn try_from(value: (ModelDescription, &Store<I>)) -> Result<Self, Self::Error> {
        let (description, store) = value;
        let incomplete = |name: &str, field: &str| {
            Err(anyhow!(ModelError::IncompleteModelDescription {
                model: name.to_string(),
                field: field.to_string(),
            }))
        };

        let model_name = description.model_name;

        if description.mesh_name.is_none() {
            return incomplete(&model_name, "mesh_name");
        }
        let mesh_name = description.mesh_name.unwrap();

        if description.pipeline_name.is_none() {
            return incomplete(&model_name, "pipeline_name");
        }
        let pipeline_name = description.pipeline_name.unwrap();

        // load mesh
        let mesh = store
            .get_mesh(&mesh_name)
            .ok_or_else(|| ModelError::MeshNotFoundInStore {
                mesh: mesh_name.to_string(),
                model: model_name.to_string(),
            })?;

        // load pipeline
        let pipeline = store.get_pipeline(&pipeline_name).ok_or_else(|| {
            ModelError::PipelineNotFoundInStore {
                model: model_name.clone(),
                pipeline: pipeline_name.clone(),
            }
        })?;

        // check material def matching pipeline needs
        let materials_count = description.geometry_materials.len();
        let mesh_count = mesh.geometries.len();

        if !pipeline.needs_material() {
            if materials_count != 0 && !pipeline.needs_material() {
                return Err(anyhow!(ModelError::InvalidMaterialAndPipeline {
                    model: model_name.clone(),
                    pipeline: pipeline_name.clone(),
                    reason: "Pipeline does not expect material".to_string()
                }));
            } else {
                let model = Model::new(model_name, pipeline, mesh.clone());
                return Ok(model);
            }
        } else {
            // should have materials
            if materials_count != mesh_count {
                return Err(anyhow!(ModelError::InvalidMaterialCount {
                    model_name: model_name.clone(),
                    mesh_name: mesh_name.clone(),
                    descriptor_materials_count: materials_count,
                    model_geometries_count: mesh_count
                }));
            }
        }

        // ready to build entity
        let mut model = Model::new(model_name.clone(), pipeline, mesh.clone());

        // add this point, all mesh should have a material
        for geometry in &mesh.as_ref().geometries {
            let geometry_name = geometry.name.to_string();
            let material_name = description
                .geometry_materials
                .iter()
                .find(|gm| gm.0.eq(&geometry_name))
                .ok_or_else(|| anyhow!("Geometry {:?} material not set !", geometry_name))?
                .1
                .to_string();

            let material = store.get_material(&material_name).ok_or_else(|| {
                ModelError::MaterialNotFoundInStore {
                    material: material_name.clone(),
                    model: model_name.clone(),
                }
            })?;

            model.set_material(geometry_name, material)?;
        }
        Ok(model)
    }
}
