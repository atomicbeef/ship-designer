use crate::materials::Material;
use crate::shape::{Shape, Shapes, ShapeHandle};

pub fn add_hardcoded_shapes(shapes: &mut Shapes) -> Vec<ShapeHandle> {
    let mut shape_handles = Vec::new();

    let aluminum_cube = Shape::new(
        10,
        10,
        10,
        vec!(Material::Aluminum; 10 * 10 * 10),
        None
    );
    shape_handles.push(shapes.add(aluminum_cube));

    shape_handles
}