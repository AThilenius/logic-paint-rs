#version 450
layout(location = 0) in vec2 v_Uv;
layout(location = 0) out vec4 o_Target;

layout(set = 2, binding = 0) uniform CellMaterial_grid_color {
    vec4 grid_color;
};
layout(set = 2, binding = 1) uniform CellMaterial_grid_res {
    vec2 grid_res;
};
layout(set = 2, binding = 2) uniform CellMaterial_n_color {
    vec4 n_color;
};
layout(set = 2, binding = 3) uniform CellMaterial_p_color {
    vec4 p_color;
};
layout(set = 2, binding = 4) uniform utexture2D CellMaterial_cells_texture;
layout(set = 2, binding = 5) uniform sampler CellMaterial_cells_texture_sampler;

bool get_bit(uint byte, uint shift) {
    return (byte & (1u << shift)) > 0u;
}

bool connection(vec2 texel_uv, bool up, bool right, bool down, bool left) {
    // Configure
    float l = 0.2;
    float h = 1.0 - l;

    float x = texel_uv.x;
    float y = 1.0 - texel_uv.y;

    return false
        || (y < h && y > l && x > l && x < h)
        || (y > h && x > l && x < h && up)
        || (y < l && x > l && x < h && down)
        || (x > h && y > l && y < h && right)
        || (x < l && y > l && y < h && left);
}

bool connection_gate(vec2 texel_uv, bool up, bool right, bool down, bool left) {
    // Configure
    float l = 0.2;
    float h = 1.0 - l;

    // Configure
    float gl = 0.3;
    float gh = 1.0 - gl;

    float x = texel_uv.x;
    float y = 1.0 - texel_uv.y;

    return false
        // Gate changes orientation based on up-down or left-right
        || ((up || down) && (y < h && y > l && x > gl && x < gh))
        || ((left || right) && (y < gh && y > gl && x > l && x < h))

        // Draw the side attachments the same as `connection`.
        || (y > h && x > l && x < h && up)
        || (y < l && x > l && x < h && down)
        || (x > h && y > l && y < h && right)
        || (x < l && y > l && y < h && left);
}

void main() {
    uvec2 texel = uvec2(floor(v_Uv * grid_res));

    // This math was taken from:
    // https://gamedev.stackexchange.com/questions/135282/any-way-to-combine-instantiated-sprite-renderers-into-one-texture-so-i-can-apply/135307#135307
    vec2 canvas_location = v_Uv * grid_res;
    vec2 tile_uv = fract(canvas_location);
    canvas_location = (canvas_location - tile_uv) / grid_res;
    tile_uv = tile_uv * 126.0 / 128.0 + 1.0 / 128.0;

    uvec4 cells = texelFetch(
        usampler2D(CellMaterial_cells_texture, CellMaterial_cells_texture_sampler),
        ivec2(texel),
        0
    );

    // Mirrors the format in cell.rs
    bool si_n = get_bit(cells.r, 7);
    bool si_p = get_bit(cells.r, 6);
    bool si_active = get_bit(cells.r, 5);
    bool si_dir_up = get_bit(cells.r, 4);
    bool si_dir_right = get_bit(cells.r, 3);
    bool si_dir_down = get_bit(cells.r, 2);
    bool si_dir_left = get_bit(cells.r, 1);

    bool gate_dir_up = get_bit(cells.g, 7);
    bool gate_dir_right = get_bit(cells.g, 6);
    bool gate_dir_down = get_bit(cells.g, 5);
    bool gate_dir_left = get_bit(cells.g, 4);
    bool gate_active = get_bit(cells.g, 3);

    bool metal = get_bit(cells.b, 7);
    bool metal_dir_up = get_bit(cells.b, 6);
    bool metal_dir_right = get_bit(cells.b, 5);
    bool metal_dir_down = get_bit(cells.b, 4);
    bool metal_dir_left = get_bit(cells.b, 3);
    bool metal_active = get_bit(cells.b, 2);
    bool via = get_bit(cells.b, 1);
    bool is_io = get_bit(cells.b, 0);

    bool metal_connection = connection(
        tile_uv,
        metal_dir_up,
        metal_dir_right,
        metal_dir_down,
        metal_dir_left
    );

    bool si_connection = connection(
        tile_uv,
        si_dir_up,
        si_dir_right,
        si_dir_down,
        si_dir_left
    );

    bool gate_connection = connection_gate(
        tile_uv,
        gate_dir_up,
        gate_dir_right,
        gate_dir_down,
        gate_dir_left
    );

    vec2 stripe_uv = tile_uv * 2.0;
    float stripe_blend = smoothstep(
        0.5,
        0.6,
        mod(stripe_uv.x + stripe_uv.y, 2) * 0.5
    );

    // Configure
    vec3 background_color = vec3(0.0);
    // Configure
    vec3 active_color = vec3(0.0);

    bool grid_1 = (((texel.x % 2u) + (texel.y % 2u)) % 2u) == 0u;
    bool grid_8 = ((((texel.x >> 3) % 2u) + ((texel.y >> 3) % 2u)) % 2u) == 0u;
    // Configure
    vec3 grid_color = vec3(1.0);
    // Configure
    float grid_blend_strength = 0.003;
    float grid_blend =
          (grid_8 ? grid_blend_strength * 0.4 : 0.0)
        + (grid_1 ? grid_blend_strength : 0.0);

    vec3 si_color = si_n ? n_color.rgb : p_color.rgb;
    si_color = mix(
        si_color,
        active_color,
        si_active ? stripe_blend * 0.5 : 0.0
    );
    float si_blend = (si_n || si_p) && si_connection ? 1.0 : 0.0;

    vec3 gate_color = si_n ? p_color.rgb : n_color.rgb;
    gate_color = mix(
        gate_color,
        active_color,
        gate_active ? stripe_blend * 0.5 : 0.0
    );
    float gate_blend = gate_connection ? 1.0 : 0.0;

    // Configure
    vec3 metal_color = vec3(0.2);
    metal_color = mix(
        metal_color,
        active_color,
        metal_active ? stripe_blend * 0.5 : 0.0
    );
    // Configure
    float metal_over_si_blend = 0.6;
    float metal_blend = metal && metal_connection ? 1.0 : 0.0;

    vec3 via_color = mix(si_color, vec3(1.0), 1.0);
    vec2 dist = tile_uv - vec2(0.5);
    float via_blend = 1.0 - smoothstep(
        0.1,
        0.3,
        dot(dist, dist) * 4.0
    );
    via_blend = via ? via_blend : 0.0;

    // Linear color blending.
    vec3 base_color = mix(background_color, grid_color, grid_blend);

    // Si totally overrides base color.
    base_color = mix(base_color, si_color, si_blend);

    // Gate totally overrides si
    base_color = mix(base_color, gate_color, gate_blend);

    // Metal is only blended if there is si.
    vec3 with_metal_color = mix(
        base_color,
        metal_color,
        si_blend > 0.5 ? metal_blend * metal_over_si_blend : metal_blend
    );

    // Vias are on or off.
    vec3 with_via_color = mix(with_metal_color, via_color, via_blend);

    o_Target = vec4(with_via_color, 1.0);
}
