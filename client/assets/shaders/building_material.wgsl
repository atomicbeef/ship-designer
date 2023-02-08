struct BuildingMaterial {
    color: vec4<f32>,
};

@group(1) @binding(0)
var<uniform> material: BuildingMaterial;

@fragment
fn fragment(
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {
    let grid_color = vec4(0.0, 0.0, 0.0, 1.0);

    var result = 0.0;
    result += 1.0 - smoothstep(0.0, 0.04, uv.x);
    result += 1.0 - smoothstep(0.0, 0.04, uv.y);

    return mix(material.color, grid_color, result);
}