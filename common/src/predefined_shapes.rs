use crate::materials::Material;
use crate::shape::{Shape, Shapes};

pub fn add_hardcoded_shapes(shapes: &mut Shapes) {
    let aluminum_cube = Shape::new(
        10,
        10,
        10,
        vec!(Material::Aluminum; 10 * 10 * 10)
    );
    shapes.add(aluminum_cube);
}