use anyhow::anyhow;
use anyhow::Result;
use serde::Deserialize;
use std::rc::Rc;

use crate::render::config::assets::TryAsRef;
use crate::render::names::NamedHandle;
use crate::render::{
    config::{assets::AssetDescriptor, material::MaterialDescriptor, mesh::MeshDescriptor},
    error::ModelError,
    scene::model::Model,
    state::WgpuState,
};

use super::{
    handles::{GeometryName, MaterialName, MeshName},
    WgpuResourceLoader,
};

/// Describe a model.
///
/// To be used to describe to composition of the model:
/// - what mesh to use by mesh name
/// - what material apply to mesh's geometry
/// - what pipeline will be used to handle the geometries and material bind groups
///
/// This struct is deserlisable from ron string.
///
/// # Example:
///
/// ```
/// let EXAMPLE: &'static str = "(
///     name:"model0_name",
///     mesh:("model0_meshname"),
///     geometries_materials:[
///         (("model0_geometry0_name"),("model0_material0_name")),
///         (("model0_geometry1_name"),("model0_material1_name"))
///     ],
///     pipeline_name:"model0_pipeline"
/// )";
/// let model: ModelDescriptor = ron::from_str(&EXAMPLE)?;
/// assert_eq!(mode.name(), ModelName::from("model0_name"));
/// # Ok::<(), ron::Result>(())
///
/// ```
///
#[derive(Deserialize, Debug)]
pub struct ModelDescriptor {
    pub(crate) name: String,
    mesh: MeshName,
    geometries_materials: Vec<(GeometryName, MaterialName)>,
    pipeline_name: String, // Pipeline descriptor...
}

// TODO: should be moved in a test
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

impl WgpuResourceLoader for ModelDescriptor {
    type Output = Rc<Model>;

    /// Does the heavy job:
    /// - check if the "wgpu entities store" has this model already
    /// - check taht the mesh description exists
    /// - get the mesh description from the assets store
    /// - check that the materials associated to the geometries are valid with the pipeline
    /// - check that the geometries on which set materials are valid for this mesh
    /// - check that either all geometries have one material or all have none
    fn load(&self, wgpu_state: &WgpuState) -> Result<Self::Output> {
        let store = &wgpu_state.store;
        let assets = &wgpu_state.assets;
        log::info!("load {}", self.name());

        // already loaded ? -> return wgpu store cache
        if store.contains_model(&self.name) {
            log::info!("Hit wgpu store cache for {}", self.name());
            return Ok(store.get_model(&self.name).unwrap());
        }
        let model_name = self.name();
        let mesh_name = &self.mesh;

        // retrieve mesh description from name in assets store
        // or err
        let mesh_descriptor: &MeshDescriptor = assets
            .get(mesh_name.clone())
            .and_then(|desc| desc.try_as_ref())?;

        let pipeline_name = self.pipeline_name.clone();

        // load mesh from store of add it to store from desc
        let mesh = store
            .get_mesh(&mesh_name)
            .map_or_else(|| mesh_descriptor.load(wgpu_state), |f| Ok(f))?;

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
            (_, false) => {
                // found some material whereas pipeline doesnt use any
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
                // maybe defer instanciation after all checks are ok ?
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

                    let material = assets
                        .get(m_name.clone())
                        .and_then(|desc: &AssetDescriptor| desc.try_as_ref())
                        .and_then(|descriptor: &MaterialDescriptor| descriptor.load(wgpu_state))?;

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
