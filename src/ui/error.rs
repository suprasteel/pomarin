use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("Trying to build material from an incomplete material: missing {field}")]
    IncompleteMaterialBuilder { field: String },
    //#[error("Mtl texture config {material} has no {texture} texture")]
    //TextureNotFound { material: String, texture: String },
    #[error("Pipeline {pipeline} not found")]
    PipelineNotFound { pipeline: String },
    #[error("Entity {entity} (mesh {mesh}) cannot use material {material} due to : {reason}")]
    IncompatibleEntityMaterial {
        entity: String,
        mesh: String,
        material: String,
        reason: String,
    },
    #[error("Mesh {geometry} not found for mesh {mesh}")]
    MeshNotFound { mesh: String, geometry: String },
    #[error("Material {material} not found")]
    MaterialNotFound { material: String },
    #[error("Mesh {name} not found")]
    ModelNotFound { name: String },
    #[error("Entity missing {field} to be built")]
    EntityBuilderIncomplete { field: String },
    #[error(
        "[EntityBuilder] Materials count ({builder_materials_count}) does not match with with models meshes count ({model_meshes_count}) for entity {entity_name} (model: {model_name})"
        )]
    EntityMaterialsMismatch {
        entity_name: String,
        model_name: String,
        builder_materials_count: usize,
        model_meshes_count: usize,
    },
    #[error("Invalid materials config for model: {reason}")]
    EntityMaterialsInvalid { reason: String },
}

#[derive(ThisError, Debug)]
pub enum AppError {
    #[error("Material {material} not found in store")]
    MaterialNotFound { material: String },
    #[error("Mesh {name} not found in store")]
    MeshNotFound { name: String },
    #[error("Pipeline {pipeline} not found in store")]
    PipelineNotFound { pipeline: String },
    #[error("Mesh {geometry} not found for mesh {mesh}")]
    GeometryNotFound { mesh: String, geometry: String },
}
