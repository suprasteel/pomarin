use cgmath::Zero;

use crate::ui::config::handles::ModelName;

use super::instance::InstanceRaw;

#[derive(PartialEq, Debug)]
pub struct Object {
    name: String,
    model: ModelName,
    pub position: cgmath::Vector3<f32>,
    pub orientation: cgmath::Quaternion<f32>,
    pub mesh_scale: f32,
    pub opacity: f32,
}

impl Object {
    pub fn new(name: String, model: ModelName) -> Self {
        Self {
            name,
            model,
            position: cgmath::Vector3::new(10.0, 10.0, 10.0),
            orientation: cgmath::Quaternion::zero(),
            mesh_scale: 1.0,
            opacity: 1.0,
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn model(&self) -> ModelName {
        self.model.clone()
    }
}

impl PartialOrd for Object {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.model.partial_cmp(&other.model) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.name.partial_cmp(&other.name) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.mesh_scale.partial_cmp(&other.mesh_scale) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.opacity.partial_cmp(&other.opacity)
    }
}

impl From<&Object> for InstanceRaw {
    fn from(o: &Object) -> Self {
        InstanceRaw {
            model: (cgmath::Matrix4::from_translation(o.position)
                * cgmath::Matrix4::from(o.orientation)
                * cgmath::Matrix4::from_scale(o.mesh_scale))
            .into(),
            normal: cgmath::Matrix3::from(o.orientation).into(),
        }
    }
}

impl Into<InstanceRaw> for Object {
    fn into(self) -> InstanceRaw {
        InstanceRaw {
            model: (cgmath::Matrix4::from_translation(self.position)
                * cgmath::Matrix4::from(self.orientation)
                * cgmath::Matrix4::from_scale(self.mesh_scale))
            .into(),
            normal: cgmath::Matrix3::from(self.orientation).into(),
        }
    }
}
