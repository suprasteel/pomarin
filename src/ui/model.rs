use std::{ops::Deref, rc::Rc};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::settings::assets::{AssetDescriptor, AssetName};

use super::{
    error::ModelError,
    geometry::GeometryName,
    material::{Material, MaterialName},
    mesh::{MeshBuf, MeshName},
    pipeline::NamedPipeline,
    resources::NamedHandle,
    wgpu_state::WgpuResourceLoader,
};

#[derive(Debug)]
pub struct Model {
    name: String,
    pub pipeline: Rc<NamedPipeline>,
    pub mesh: Rc<MeshBuf>,
    pub materials: Vec<Rc<dyn Material>>,
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
}

/// # Describe a model.
///
/// ## Example:
///
/// ```
/// {
///     name: "pink_house",
///     mesh: MeshName("house"),
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
    mesh: MeshName,
    geometries_materials: Vec<(GeometryName, MaterialName)>,
    pipeline_name: String, // Pipeline descriptor...
}

impl ModelDescriptor {
    pub fn _new_(
        name: String,
        mesh: MeshName,
        geometries_materials: Vec<(GeometryName, MaterialName)>,
        pipeline_name: String,
    ) -> Self {
        Self {
            name,
            mesh, // mesh: MeshName(String)
            geometries_materials,
            pipeline_name,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Eq, Ord, PartialEq, PartialOrd, Clone, Hash)]
pub struct ModelName(String);

impl WgpuResourceLoader for ModelDescriptor {
    type Output = Rc<Model>;

    /// Does the heavy job:
    /// - check if the "wgpu entities store" has this model already
    /// - check taht the mesh description exists
    /// - get the mesh description from the assets store
    /// - check that the materials associated to the geometries are valid with the pipeline
    /// - check that the geometries on which set materials are valid for this mesh
    /// - check that either all geometries have one material or all have none
    fn load(&self, wgpu_state: &super::wgpu_state::WgpuState) -> Result<Self::Output> {
        let store = &wgpu_state.store;

        // already loaded ? -> return wgpu store cache
        if store.contains_model(&self.name) {
            log::info!(target: "load", "wgpu store already contains {}", self.name());
            return Ok(store.get_model(&self.name).unwrap());
        }
        let model_name = self.name();
        let mesh_name = &self.mesh;

        let desc_not_fnd = |asset: AssetName| {
            anyhow!(
                "Asset descriptor {} not found (model {})",
                asset,
                model_name
            )
        };

        // retrieve mesh description from name in assets store
        // or err
        let mesh_descriptor = &wgpu_state.assets.find(mesh_name.clone()).map_or_else(
            || Err(desc_not_fnd(mesh_name.clone().into())),
            |md| {
                if let AssetDescriptor::Mesh(mesh) = md {
                    Ok(mesh)
                } else {
                    Err(desc_not_fnd(mesh_name.clone().into()))
                }
            },
        )?;

        let pipeline_name = self.pipeline_name.clone();
        // load mesh from store
        let mesh = store
            .get_mesh(&mesh_name)
            // should load mesh in this case
            .ok_or_else(|| ModelError::MeshNotFoundInStore {
                mesh: mesh_name.clone(),
                model: model_name.clone(),
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
                let model = Model::new(model_name.to_string(), pipeline, mesh.clone());
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
            (mat_cnt, true) if mat_cnt != mesh_descriptor.count_geometries() => {
                Err(anyhow!(ModelError::InvalidMaterialCount {
                    model_name: model_name.clone(),
                    mesh_name: mesh_name.clone(),
                    descriptor_materials_count: mat_cnt,
                    model_geometries_count: mesh_descriptor.count_geometries(),
                }))
            }
            _ => {
                let mut model = Model::new(model_name.to_string(), pipeline, mesh.clone());
                let geometries_names = mesh_descriptor.geometries_names();

                for (g_name, m_name) in &self.geometries_materials {
                    // does the mesh declares the same geometries we are setting materials to ?
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

impl From<&str> for ModelName {
    fn from(value: &str) -> Self {
        ModelName(value.to_string())
    }
}

impl std::fmt::Display for ModelName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Model({})", self.0)
    }
}

impl Deref for ModelName {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl NamedHandle<ModelName> for ModelDescriptor {
    fn name(&self) -> ModelName {
        ModelName(self.name.to_string())
    }
}
impl NamedHandle<ModelName> for Model {
    fn name(&self) -> ModelName {
        ModelName(self.name.to_string())
    }
}

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
