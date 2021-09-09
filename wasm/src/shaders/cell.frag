#version 300 es
precision highp float;
precision highp int;
precision highp usampler2D;

in vec2 v_Uv;
out vec4 o_Target;

layout(std140) uniform CellMaterial_grid_color { // set = 2, binding = 0
    vec4 grid_color;
};
layout(std140) uniform CellMaterial_grid_res { // set = 2, binding = 1
    vec2 grid_res;
};
layout(std140) uniform CellMaterial_n_color { // set = 2, binding = 2
    vec4 n_color;
};
layout(std140) uniform CellMaterial_p_color { // set = 2, binding = 3
    vec4 p_color;
};
uniform usampler2D CellMaterial_texture_sampler; // set = 2, binding = 4

bool get_bit(uint byte, uint shift) {
    return ((byte >> shift) & 1u) == 1u;
}

void main() {
    uvec2 texel = uvec2(floor(v_Uv * grid_res));

    uvec4 tex = texelFetch(
        // usampler2D(CellMaterial_texture, CellMaterial_texture_sampler),
        CellMaterial_texture_sampler,
        ivec2(texel),
        0
    );

    // Unpack first 8 bites
    bool lower_n = get_bit(tex.r, 7);
    bool lower_p = get_bit(tex.r, 6);
    bool upper_n = get_bit(tex.r, 5);
    bool upper_p = get_bit(tex.r, 4);
    bool lower_error = get_bit(tex.r, 3);
    bool lower_active = get_bit(tex.r, 2);
    bool upper_error = get_bit(tex.r, 1);
    bool upper_active = get_bit(tex.r, 0);

    // Unpack second 8 bits
    bool has_metal = get_bit(tex.g, 7);
    bool has_via = get_bit(tex.g, 6);
    bool metal_error = get_bit(tex.g, 5);
    bool metal_active = get_bit(tex.g, 4);

    // Misc flags
    bool painted = (tex.r >> 4) != 0u || has_metal;
    bool grid_1 = (((texel.x % 2u) + (texel.y % 2u)) % 2u) == 0u;
    bool grid_8 = ((((texel.x >> 3) % 2u) + ((texel.y >> 3) % 2u)) % 2u) == 0u;
    bool has_upper_layer = upper_n || upper_p;
    bool has_lower_layer = lower_n || lower_p;
    bool has_si = has_upper_layer || has_lower_layer;

    // Via drawing
    vec2 texel_uv = (v_Uv * grid_res) - floor(v_Uv * grid_res);
    vec2 dist = texel_uv - vec2(0.5);
    float via_step = 1.0 - smoothstep(
        0.1,
        0.3,
        dot(dist, dist) * 4.0
    );

    float grid = grid_8 && grid_1 ? 0.6 : (grid_1 ? 0.8 : 1.0);

    // Calculate colors
    vec3 c_n = n_color.rgb;
    vec3 c_p = p_color.rgb;
    vec3 c_lower = lower_n ? c_n : (lower_p ? c_p : vec3(0.0));
    vec3 c_upper = upper_n ? c_n : (upper_p ? c_p : vec3(0.0));
    vec3 c_metal = vec3(0.6);
    vec3 c_si = has_upper_layer ? c_upper - (c_upper * 0.4) + (c_lower * 0.1) : c_lower;
    vec3 c_base = has_si ? c_si : vec3(0.4);
    vec3 c_si_and_metal = has_metal ? c_base * 0.2 : c_base;
    vec3 c_via = has_via ? vec3(via_step) : vec3(0.0);
    vec3 c_cell = c_si_and_metal + c_via;
    vec3 c_out = c_cell * (painted ? 1.0 : grid);

    // All done.
    o_Target = vec4(c_out, 1.0);
}
