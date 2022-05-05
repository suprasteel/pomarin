use std::ops::Range;

use super::model::Model;

pub trait DrawModel<'m> {
    fn draw_meshes(&mut self, model: &'m Model, instances_range: Range<u32>);

    /// draw an entity and all its instances using an instance buffer previously set.
    /// The offset parameter refers to the offset in the instance buffer
    /// Example: if the instance buffer has 4 values
    /// if the entity has two instances
    /// if the entity instances are linked to the instance buffer from the index 1 in the instance
    /// buffer (2nd plce)
    /// then we call this function with offset 1 and the range given to draw will be 1..3
    fn draw_model<M: AsRef<Model>>(
        &mut self,
        model: &'m M,
        offset: u32,
        camera_bg: &'m wgpu::BindGroup,
        light_bg: &'m wgpu::BindGroup,
    );

    fn draw_models<M: AsRef<Model>>(
        &mut self,
        models: Vec<&'m M>,
        camera_bg: &'m wgpu::BindGroup,
        light_bg: &'m wgpu::BindGroup,
    );
}

impl<'m, 'p> DrawModel<'m> for wgpu::RenderPass<'p>
where
    'm: 'p,
{
    fn draw_models<M: AsRef<Model>>(
        &mut self,
        models: Vec<&'m M>,
        camera_bg: &'m wgpu::BindGroup,
        light_bg: &'m wgpu::BindGroup,
    ) {
        let mut offset = 0;
        for model in models {
            //dbg!(&entity.as_ref());
            self.draw_model(model, offset, camera_bg, light_bg);
            offset += 1; //model.as_ref().instances_count();
        }
    }

    fn draw_meshes(&mut self, model: &'m Model, instances_range: Range<u32>) {
        let mut mesh_index = 0;

        for mesh in &model.mesh.geometries {
            if model.pipeline.needs_material() {
                self.set_bind_group(
                    2,
                    model
                        .materials
                        .get(mesh_index)
                        .expect("mesh material not present during render"),
                    &[],
                );
            }
            self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            self.draw_indexed(0..mesh.num_elements, 0, instances_range.clone());
            mesh_index += 1;
        }
    }

    fn draw_model<M: AsRef<Model>>(
        &mut self,
        model: &'m M,
        instances_offset: u32,
        camera_bg: &'m wgpu::BindGroup,
        light_bg: &'m wgpu::BindGroup,
    ) {
        let model = model.as_ref();
        let instances_end = instances_offset + 1; //model.instances_count();
        self.set_pipeline(&model.pipeline);
        self.set_bind_group(0, &camera_bg, &[]);
        self.set_bind_group(1, &light_bg, &[]);
        self.draw_meshes(model, instances_offset..instances_end);
    }
}
