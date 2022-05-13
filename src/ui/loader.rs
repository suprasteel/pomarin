#[cfg(test)]
mod test {
    use crate::ui::{
        geometry::{GeometryDescriptor, GeometryName},
        material::{
            ColorMaterialDescriptor, MaterialDescriptor, MaterialName, TextureMaterialDescriptor,
        },
        mesh::{MeshDescriptor, MeshName, VerticesSource},
        model::ModelDescriptor,
        texture::TextureName,
    };

    const MODEL_ASSET_EXAMPLE_CONFIG: &'static str = r#"
        [
        (
            name:"texture_zod",
            mesh:("zodiac"),
            geometries_materials:[
            (("hull"),("wall")),
            (("inflatable"),("default"))
            ],
            pipeline_name:"textures_pipeline"
        ),
        (
            name:"color_zod",
            mesh:("zodiac"),
            geometries_materials:[
            (("hull"),("white")),
            (("inflatable"),("grey"))
            ],
            pipeline_name:"colors_pipeline"
        ),
        (
            name:"sea_square",
            mesh:("sea"),
            geometries_materials:[
            (("surface"),("sea")),
            ],
            pipeline_name:"textures_pipeline"
        ),
        (
            name:"fake_terrain",
            mesh:("terrain"),
            geometries_materials:[
            (("terrain"),("wall")),
            ],
            pipeline_name:"textures_pipeline"
        ),


        ]
    "#;

    #[test]
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
}
