pub struct ObjectsPass {}

impl ObjectsPass {
    fn new() -> Self {
        let mut instances_system = InstancesSystem::new(&device);
        let (light_bgl, light) = light::LightSystem::init(LightUniform::default(), &device);

        Self {}
    }
}
