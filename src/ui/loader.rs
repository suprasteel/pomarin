#[cfg(test)]
mod test {
    use crate::ui::{
        geometry::{GeometryDescriptor, GeometryName},
        material::{
            ColorMaterialDescriptor, MaterialDescriptor, MaterialName, TextureMaterialDescriptor,
        },
        mesh::{MeshDescriptor, MeshName, VerticesSource},
        model::{ModelDescriptor, ModelName},
        resources::NamedHandle,
        texture::TextureName,
    };
    use anyhow::{anyhow, Result};

    const MODEL_ASSET_EXAMPLE_CONFIG: &'static str = r#"
        [
        (
            name:"model0_name",
            mesh:("model0_meshname"),
            geometries_materials:[
            (("model0_geometry0_name"),("model0_material0_name")),
            (("model0_geometry1_name"),("model0_material1_name"))
            ],
            pipeline_name:"model0_pipeline"
        ),
        (
            name:"pink_house",
            mesh:("tiny_house"),
            geometries_materials:[
            (("wall_geometry"),("pink_material")),
            (("door_geometry"),("wood_material"))
            ],
            pipeline_name:"model1_pipeline"
        ),
        (
            name:"green_house",
            mesh:("tiny_house"),
            geometries_materials:[
            (("wall_geometry"),("green_material")),
            (("door_geometry"),("wood_material"))
            ],
            pipeline_name:"model2_pipeline"
        ),
        (
            name:"alien",
            mesh:("body"),
            geometries_materials:[
            (("skin"),("green")),
            ],
            pipeline_name:"model3_pipeline"
        ),
        ]
    "#;

    #[test]
    fn read_models_conf() -> Result<()> {
        let models: Vec<ModelDescriptor> = ron::from_str(&MODEL_ASSET_EXAMPLE_CONFIG)?;
        let model0 = models.get(0).ok_or_else(|| anyhow!("item not found"))?;
        assert_eq!(model0.name(), ModelName::from("model0_name"));
        Ok(())
    }

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
