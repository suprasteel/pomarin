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
            VerticesSource::Obj("zodiac_001.obj".to_string()),
            vec![
                GeometryDescriptor::from("hull"),
                GeometryDescriptor::from("inflatable"),
            ],
        ),
        vec![
            (GeometryName::from("hull"), MaterialName::from("wall")),
            (
                GeometryName::from("inflatable"),
                MaterialName::from("default"),
            ),
        ],
        "textures_pipeline".to_string(),
    );
    let model2 = ModelDescriptor::_new_(
        "tex_zodiac".to_string(),
        MeshDescriptor::_new_(
            "zodiac".to_string(),
            VerticesSource::Obj("zodiac_001.obj".to_string()),
            vec![
                GeometryDescriptor::from("hull"),
                GeometryDescriptor::from("inflatable"),
            ],
        ),
        vec![
            (GeometryName::from("hull"), MaterialName::from("color_001")),
            (
                GeometryName::from("inflatable"),
                MaterialName::from("color_002"),
            ),
        ],
        "colors_pipeline".to_string(),
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

    let models = vec![model, model2];

    let materials: Vec<MaterialDescriptor> = vec![
        MaterialDescriptor::Texture(TextureMaterialDescriptor::_new_(
            "default".to_string(),
            TextureDescriptor::_new_(
                "diffuse".to_string(),
                "default_diffuse.jpg".to_string().into(),
                TextureKind::Diffuse,
            ),
            TextureDescriptor::_new_(
                "normal".to_string(),
                "default_normal.jpg".to_string().into(),
                TextureKind::Normal,
            ),
        )),
        MaterialDescriptor::Texture(TextureMaterialDescriptor::_new_(
            "wall".to_string(),
            TextureDescriptor::_new_(
                "wall_diffuse".to_string(),
                "wall_diffuse.jpg".to_string().into(),
                TextureKind::Diffuse,
            ),
            TextureDescriptor::_new_(
                "wall_normal".to_string(),
                "wall_normal.jpg".to_string().into(),
                TextureKind::Normal,
            ),
        )),
        MaterialDescriptor::Color(ColorMaterialDescriptor::_new_(
            "color_001".to_string(),
            [0.1, 0.2, 0.3],
            [0.5, 0.6, 0.7],
            [0.5, 0.5, 0.5],
        )),
    ];

    // Convert the Point to a JSON string.
    let models = ron::to_string(&models).unwrap();
    let mesh = ron::to_string(&mesh).unwrap();
    let materials = ron::to_string(&materials).unwrap();
    print!("\n\n{}\n\n", models);
    print!("\n\n{}\n\n", mesh);
    print!("\n\n{}\n\n", materials);
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
