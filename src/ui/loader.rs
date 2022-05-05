use std::{fs, path::Path};

use anyhow::Result;

use super::{
    geometry::{GeometryDescriptor, GeometryName},
    material::{
        ColorMaterialDescriptor, MaterialDescriptor, MaterialName, TextureMaterialDescriptor,
    },
    mesh::{MeshDescriptor, VerticesSource},
    model::ModelDescriptor,
    texture::{TextureDescriptor, TextureKind},
};

pub fn example_model() {
    let model = ModelDescriptor::_new_(
        "test".to_string(),
        MeshDescriptor::_new_(
            "house".to_string(),
            VerticesSource::Obj("file.obj".to_string()),
            vec![
                GeometryDescriptor::from("window"),
                GeometryDescriptor::from("door"),
                GeometryDescriptor::from("wall"),
            ],
        ),
        vec![
            (GeometryName::from("window"), MaterialName::from("glass")),
            (GeometryName::from("door"), MaterialName::from("wood")),
            (GeometryName::from("wall"), MaterialName::from("pink")),
        ],
        "pipeline_1".to_string(),
    );

    let mesh = MeshDescriptor::_new_(
        "house".to_string(),
        VerticesSource::Obj("file.obj".to_string()),
        vec![
            GeometryDescriptor::from("window"),
            GeometryDescriptor::from("door"),
            GeometryDescriptor::from("wall"),
        ],
    );

    let material = MaterialDescriptor::Texture(TextureMaterialDescriptor::_new_(
        "texture_mat".to_string(),
        TextureDescriptor::_new_(
            "diff".to_string(),
            "coco.jpg".to_string().into(),
            TextureKind::Diffuse,
        ),
        TextureDescriptor::_new_(
            "norm".to_string(),
            "coco2.jpg".to_string().into(),
            TextureKind::Normal,
        ),
    ));

    // Convert the Point to a JSON string.
    let model = ron::to_string(&model).unwrap();
    let mesh = ron::to_string(&mesh).unwrap();
    let material = ron::to_string(&material).unwrap();
    print!("\n\n{}\n\n", model);
    print!("\n\n{}\n\n", mesh);
    print!("\n\n{}\n\n", material);
}

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
