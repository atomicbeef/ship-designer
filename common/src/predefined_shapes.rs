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
    shape_handles.push(shapes.add_static(aluminum_cube));

    let test_prism_2x1x3 = Shape::new(
        2,
        1,
        3,
        vec!(Material::Aluminum; 2 * 1 * 3),
        None
    );
    shape_handles.push(shapes.add_static(test_prism_2x1x3));

    shape_handles
}