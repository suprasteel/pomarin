use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::{
    instance::RawInstanceTrait, mesh::Mesh, object::Object, pipeline::NamedPipeline, texture,
};

pub struct Store<I>
where
    I: RawInstanceTrait,
{
    /// wgpu textures (with sampler and view)
    pub textures: RefCell<HashMap<String, Rc<texture::Texture>>>,
    /// materials dereferencing to bing group
    pub materials: RefCell<HashMap<String, Rc<dyn Material>>>,
    /// meshes made of geometries
    pub meshes: RefCell<HashMap<String, Rc<Mesh>>>,
    /// render unit using a shader and able to process a model with its mesh and material
    pub pipelines: RefCell<HashMap<String, Rc<NamedPipeline>>>,
    // TODO: rename to models
    /// an aggregation of material and geometries (via mesh)
    pub models: RefCell<HashMap<String, Rc<Model<I>>>>,
    /// objects instances describing whick entity to use
    pub objects: RefCell<HashMap<String, Rc<Object>>>,
}

impl<I> Store<I>
where
    I: RawInstanceTrait,
{
    pub fn new() -> Self {
        Self {
            textures: RefCell::new(HashMap::new()),
            materials: RefCell::new(HashMap::new()),
            meshes: RefCell::new(HashMap::new()),
            models: RefCell::new(HashMap::new()), // RefCell::new(BinaryHeap::new()),
            pipelines: RefCell::new(HashMap::new()),
            objects: RefCell::new(HashMap::new()),
        }
    }

    pub fn add_instance(&self, instance: Rc<Object>) {
        self.objects
            .borrow_mut()
            .insert(instance.as_ref().name.clone(), instance);
    }

    pub fn get_instance<S: AsRef<str>>(&self, name: S) -> Option<Rc<Object>> {
        self.objects
            .borrow()
            .get(&name.as_ref().to_string())
            .map(|m| m.clone())
    }

    pub fn add_pipeline(&self, pipeline: Rc<NamedPipeline>) {
        self.pipelines
            .borrow_mut()
            .insert(pipeline.as_ref().name.clone(), pipeline);
    }

    pub fn add_mesh(&self, mesh: Rc<Mesh>) {
        self.meshes
            .borrow_mut()
            .insert(mesh.as_ref().name.clone(), mesh);
    }

    pub fn add_entity(&self, entity: Rc<Model<I>>) {
        self.models
            .borrow_mut()
            .insert(entity.as_ref().name.clone(), entity);
    }

    pub fn add_material(&self, material: Rc<dyn Material>) {
        self.materials
            .borrow_mut()
            .insert(material.name(), material);
    }

    pub fn add_texture<S: AsRef<str>>(&self, name: S, texture: Rc<texture::Texture>) {
        self.textures
            .borrow_mut()
            .insert(name.as_ref().to_string(), texture);
    }

    pub fn get_texture<S: AsRef<str>>(&self, name: S) -> Option<Rc<texture::Texture>> {
        self.textures
            .borrow()
            .get(&name.as_ref().to_string())
            .map(|m| m.clone())
    }

    pub fn get_mesh<S: AsRef<str>>(&self, name: S) -> Option<Rc<Mesh>> {
        self.meshes
            .borrow()
            .get(&name.as_ref().to_string())
            .map(|m| m.clone())
    }

    pub fn get_material<S: AsRef<str>>(&self, name: S) -> Option<Rc<dyn Material>> {
        self.materials
            .borrow()
            .get(&name.as_ref().to_string())
            .map(|m| m.clone())
    }

    pub fn get_pipeline<S: AsRef<str>>(&self, name: S) -> Option<Rc<NamedPipeline>> {
        self.pipelines
            .borrow()
            .get(&name.as_ref().to_string())
            .map(|m| m.clone())
    }

    pub fn get_entity<S: AsRef<str>>(&self, name: S) -> Option<Rc<Model<I>>> {
        self.models
            .borrow()
            .get(&name.as_ref().to_string())
            .map(|m| m.clone())
    }

    pub fn entities(&self) -> Vec<Rc<Model<I>>> {
        self.models
            .borrow()
            .iter()
            .map(|(_name, entity)| entity.clone())
            .collect()
    }
}
