#version 450
layout(location = 0) in vec2 v_Uv;
layout(location = 0) out vec4 o_Target;

layout(set = 2, binding = 0) uniform CellMaterial_grid_color {
    vec4 grid_color;
};
layout(set = 2, binding = 1) uniform CellMaterial_grid_res {
    vec2 grid_res;
};
layout(set = 2, binding = 2) uniform texture2D CellMaterial_texture;
layout(set = 2, binding = 3) uniform sampler CellMaterial_texture_sampler;

float grid(vec2 loc, float size) {
    vec2 scaled = loc / size;
    return mod(floor(mod(scaled.x, 2)) + floor(mod(scaled.y, 2)), 2);
}

void main() {
    vec2 grid_uv = v_Uv * grid_res;
    float grid_a = (grid(grid_uv, 1) * 0.3) + (grid(grid_uv, 8) * 0.7);
    vec4 grid = vec4(grid_color.rgb, grid_color.a * grid_a);

    vec4 tex = texture(
        sampler2D(CellMaterial_texture, CellMaterial_texture_sampler),
        v_Uv
    );

    o_Target = vec4(tex.rgb, tex.a * grid_a);
}
