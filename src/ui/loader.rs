use crate::ui::{mesh::MeshName, texture::TextureName};

use super::{
    geometry::{GeometryDescriptor, GeometryName},
    material::{
        ColorMaterialDescriptor, MaterialDescriptor, MaterialName, TextureMaterialDescriptor,
    },
    mesh::{MeshDescriptor, VerticesSource},
    model::ModelDescriptor,
};

pub fn example_model() {
    let model = ModelDescriptor::_new_(
        "test".to_string(),
        MeshName::from("house"),
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
        MeshName::from("zodiac"),
        vec![
            (GeometryName::from("hull"), MaterialName::from("color_001")),
            (
                GeometryName::from("inflatable"),
                MaterialName::from("color_002"),
            ),
        ],
        "colors_pipeline".to_string(),
    );

    let meshes = vec![
        MeshDescriptor::_new_(
            "zodiac".to_string(),
            VerticesSource::Obj("zodiac_001.obj".to_string()),
            vec![
                GeometryDescriptor::from("hull"),
                GeometryDescriptor::from("inflatable"),
            ],
        ),
        MeshDescriptor::_new_(
            "cube".to_string(),
            VerticesSource::Obj("cube_001.obj".to_string()),
            vec![GeometryDescriptor::from("cube")],
        ),
    ];

    let models = vec![model, model2];

    let materials: Vec<MaterialDescriptor> = vec![
        MaterialDescriptor::Texture(TextureMaterialDescriptor::_new_(
            "default".to_string(),
            TextureName::from("d_default"),
            TextureName::from("n_default"),
        )),
        MaterialDescriptor::Texture(TextureMaterialDescriptor::_new_(
            "wall".to_string(),
            TextureName::from("d_wall"),
            TextureName::from("n_wall"),
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
    let meshes = ron::to_string(&meshes).unwrap();
    let materials = ron::to_string(&materials).unwrap();
    print!("\n\n{}\n\n", models);
    print!("\n\n{}\n\n", meshes);
    print!("\n\n{}\n\n", materials);
}
