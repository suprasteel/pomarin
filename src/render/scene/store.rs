use super::{
    material::Material, mesh::MeshBuf, model::Model, pipeline::NamedPipeline, texture::Texture,
};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub struct Store {
    /// wgpu textures (with sampler and view)
    pub textures: RefCell<HashMap<String, Rc<Texture>>>,
    /// materials dereferencing to bing group
    pub materials: RefCell<HashMap<String, Rc<dyn Material>>>,
    /// meshes made of geometries
    pub meshes: RefCell<HashMap<String, Rc<MeshBuf>>>,
    /// render unit using a shader and able to process a model with its mesh and material
    pub pipelines: RefCell<HashMap<String, Rc<NamedPipeline>>>,
    /// an aggregation of material and geometries (via mesh)
    pub models: RefCell<HashMap<String, Rc<Model>>>,
}

impl Store {
    pub fn new() -> Self {
        Self {
            textures: RefCell::new(HashMap::new()),
            materials: RefCell::new(HashMap::new()),
            meshes: RefCell::new(HashMap::new()),
            models: RefCell::new(HashMap::new()), // RefCell::new(BinaryHeap::new()),
            pipelines: RefCell::new(HashMap::new()),
        }
    }

    pub fn add_pipeline(&self, pipeline: Rc<NamedPipeline>) {
        self.pipelines
            .borrow_mut()
            .insert(pipeline.as_ref().name().clone(), pipeline);
    }

    pub fn get_pipeline<S: AsRef<str>>(&self, name: S) -> Option<Rc<NamedPipeline>> {
        self.pipelines
            .borrow()
            .get(&name.as_ref().to_string())
            .map(|m| m.clone())
    }

    pub fn add_mesh(&self, mesh: Rc<MeshBuf>) {
        self.meshes
            .borrow_mut()
            .insert(mesh.as_ref().name.clone(), mesh);
    }

    pub fn contains_mesh(&self, mesh: &str) -> bool {
        self.meshes.borrow().contains_key(mesh)
    }

    pub fn get_mesh<S: AsRef<str>>(&self, name: S) -> Option<Rc<MeshBuf>> {
        self.meshes
            .borrow()
            .get(&name.as_ref().to_string())
            .map(|m| m.clone())
    }

    pub fn add_model(&self, entity: Rc<Model>) {
        self.models
            .borrow_mut()
            .insert(entity.as_ref().name.clone(), entity);
    }

    pub fn contains_model(&self, model: &str) -> bool {
        self.models.borrow().contains_key(model)
    }

    pub fn add_material(&self, material: Rc<dyn Material>) {
        self.materials
            .borrow_mut()
            .insert(material.name(), material);
    }

    pub fn contains_material(&self, material: &str) -> bool {
        self.materials.borrow().contains_key(material)
    }

    pub fn get_material<S: AsRef<str>>(&self, name: S) -> Option<Rc<dyn Material>> {
        self.materials
            .borrow()
            .get(&name.as_ref().to_string())
            .map(|m| m.clone())
    }

    pub fn add_texture<S: AsRef<str>>(&self, name: S, texture: Rc<Texture>) {
        self.textures
            .borrow_mut()
            .insert(name.as_ref().to_string(), texture);
    }

    pub fn contains_texture(&self, texture: &str) -> bool {
        self.textures.borrow().contains_key(texture)
    }

    pub fn get_texture<S: AsRef<str>>(&self, name: S) -> Option<Rc<Texture>> {
        self.textures
            .borrow()
            .get(&name.as_ref().to_string())
            .map(|m| m.clone())
    }

    pub fn get_model<S: AsRef<str>>(&self, name: S) -> Option<Rc<Model>> {
        self.models
            .borrow()
            .get(&name.as_ref().to_string())
            .map(|m| m.clone())
    }

    pub fn _models(&self) -> Vec<Rc<Model>> {
        self.models
            .borrow()
            .iter()
            .map(|(_name, model)| model.clone())
            .collect()
    }
}
