use thiserror::Error as ThisError;

use super::{geometry::GeometryName, material::MaterialName, mesh::MeshName};

#[derive(ThisError, Debug)]
pub enum MaterialError {
    #[error("Missing {field} to build material {material}")]
    MaterialBuilderIncomplete { material: String, field: String },
    #[error("Cannot build {type_to_deser} from {input}")]
    DeserialisationError {
        type_to_deser: String,
        input: String,
    },
}

#[derive(ThisError, Debug)]
pub enum TextureError {
    #[error("Cannot build {type_to_deser} from {input}")]
    DeserialisationError {
        type_to_deser: String,
        input: String,
    },
}

#[derive(ThisError, Debug)]
pub enum ModelError {
    #[error("Mesh {mesh} not found in store for model {model}")]
    MeshNotFoundInStore { mesh: MeshName, model: String },
    #[error(
        "Pipeline {pipeline} not found in store while trying to build {model} model description"
    )]
    PipelineNotFoundInStore { model: String, pipeline: String },
    #[error("Geometry {geometry} not found in store for model {model}")]
    GeometryNotFound { geometry: String, model: String },
    #[error("Material {material} not found in store for model {model}")]
    MaterialNotFoundInStore {
        material: MaterialName,
        model: String,
    },
    #[error("Model {model} (mesh {mesh}) cannot use material {material} due to : {reason}")]
    IncompatibleModelMaterial {
        model: String,
        mesh: String,
        material: String,
        reason: String,
    },
    #[error("Missing field {field} when trying to build {model} model description")]
    IncompleteModelDescription { model: String, field: String },
    #[error(
        "Materials count ({descriptor_materials_count}) does not match with geometries count ({model_geometries_count}) for model {model_name} (mesh: {mesh_name})"
        )]
    InvalidMaterialCount {
        model_name: String,
        mesh_name: MeshName,
        descriptor_materials_count: usize,
        model_geometries_count: usize,
    },
    #[error(
        "Invalid materials configuration for model {model} using pipeline {pipeline}: {reason}"
    )]
    InvalidMaterialAndPipeline {
        model: String,
        pipeline: String,
        reason: String,
    },
    #[error("Material not set for geometry {geometry} for model {model}")]
    MaterialNotSetForGeometry {
        geometry: GeometryName,
        model: String,
    },
}
