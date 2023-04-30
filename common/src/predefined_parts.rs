use crate::part::materials::Material;
use crate::part::{Part, Parts, PartHandle};

pub fn add_hardcoded_parts(parts: &mut Parts) -> Vec<PartHandle> {
    let mut part_handles = Vec::new();

    let aluminum_cube = Part::new(
        10,
        10,
        10,
        vec!(Material::Aluminum; 10 * 10 * 10),
        None
    );
    part_handles.push(parts.add_static(aluminum_cube));

    let test_prism_2x1x3 = Part::new(
        2,
        1,
        3,
        vec!(Material::Aluminum; 2 * 1 * 3),
        None
    );
    part_handles.push(parts.add_static(test_prism_2x1x3));

    part_handles
}