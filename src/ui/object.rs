#[derive(PartialEq, Debug)]
pub struct Object {
    name: String,
    model: String,
    position: [f32; 3],
    orientation: [f32; 4],
    mesh_scale: f32,
    opacity: f32,
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
        match self.position.partial_cmp(&other.position) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.orientation.partial_cmp(&other.orientation) {
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
