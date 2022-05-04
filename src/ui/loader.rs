use std::{fs, path::Path};

use anyhow::Result;

use super::{material::MaterialDescriptor, mesh::MeshDescriptor};

pub fn read_materials_descriptors<P: AsRef<Path>>(file: P) -> Result<Vec<MaterialDescriptor>> {
    let string_content = fs::read_to_string(file)?;
    let list: Vec<MaterialDescriptor> = ron::from_str(&string_content)?;
    Ok(list)
}

pub fn read_mesh_descriptors<P: AsRef<Path>>(file: P) -> Result<Vec<MeshDescriptor>> {
    let string_content = fs::read_to_string(file)?;
    let list: Vec<MeshDescriptor> = ron::from_str(&string_content)?;
    Ok(list)
}
