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
layout(set = 2, binding = 6) uniform texture2D CellMaterial_atlas_texture;
layout(set = 2, binding = 7) uniform sampler CellMaterial_atlas_texture_sampler;

bool get_bit(uint byte, uint shift) {
    return (byte & (1u << shift)) > 0u;
}

bool connection(
    vec2 texel_uv,
    bool up,
    bool right,
    bool down,
    bool left
) {
    // Cutoffs
    float l = 0.2;
    float h = 1.0 - l;

    float x = texel_uv.x;
    float y = 1.0 - texel_uv.y;

    if (y > h) {
        if (x > l && x < h && up) {
            return true;
        } else {
            return false;
        }
    }

    if (y < l) {
        if (x > l && x < h && down) {
            return true;
        } else {
            return false;
        }
    }

    if (x > h) {
        if (y > l && y < h && right) {
            return true;
        } else {
            return false;
        }
    }

    if (x < l) {
        if (y > l && y < h && left) {
            return true;
        } else {
            return false;
        }
    }

    return true;
}

float sample_atlas_cell(
    vec2 texel_uv,
    vec2 cell
) {
    vec2 scaled_uv = texel_uv / 4.0;
    vec2 offset_uv = scaled_uv + cell;
    return texture(
        sampler2D(CellMaterial_atlas_texture, CellMaterial_atlas_texture_sampler),
        vec2(offset_uv.x, 1.0 - offset_uv.y)
    ).r;
}

float get_tile(
    vec2 texel_uv,
    bool set,
    bool up,
    bool right,
    bool down,
    bool left
) {
    texel_uv.y = 1.0 - texel_uv.y;
    return clamp(
          (set ? sample_atlas_cell(texel_uv, vec2(0.0, 0.75)) : 0.0)
        + (up ? sample_atlas_cell(texel_uv, vec2(0.0, 0.5)) : 0.0)
        + (right ? sample_atlas_cell(texel_uv, vec2(0.25, 0.5)) : 0.0)
        + (down ? sample_atlas_cell(texel_uv, vec2(0.5, 0.5)) : 0.0)
        + (left ? sample_atlas_cell(texel_uv, vec2(0.75, 0.5)) : 0.0),
        0.0,
        1.0
    );
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

    // Misc flags
    // bool painted = (cells.r >> 4) != 0u || has_metal;
    // bool has_upper_layer = upper_n || upper_p;
    // bool has_lower_layer = lower_n || lower_p;
    // bool has_si = has_upper_layer || has_lower_layer;

    // Via drawing
    // vec2 dist = texel_uv - vec2(0.5);
    // float via_step = 1.0 - smoothstep(
    //     0.1,
    //     0.3,
    //     dot(dist, dist) * 4.0
    // );

    bool grid_1 = (((texel.x % 2u) + (texel.y % 2u)) % 2u) == 0u;
    bool grid_8 = ((((texel.x >> 3) % 2u) + ((texel.y >> 3) % 2u)) % 2u) == 0u;
    float grid = grid_8 && grid_1 ? 0.6 : (grid_1 ? 0.8 : 1.0);

    // Calculate colors
    // vec3 c_n = n_color.rgb;
    // vec3 c_p = p_color.rgb;
    // vec3 c_lower = lower_n ? c_n : (lower_p ? c_p : vec3(0.0));
    // vec3 c_upper = upper_n ? c_n : (upper_p ? c_p : vec3(0.0));
    // vec3 c_metal = vec3(0.6);
    // vec3 c_si = has_upper_layer ? c_upper - (c_upper * 0.4) + (c_lower * 0.1) : c_lower;
    // vec3 c_base = has_si ? c_si : vec3(0.4);
    // vec3 c_si_and_metal = has_metal ? c_base * 0.2 : c_base;
    // vec3 c_via = has_via ? vec3(via_step) : vec3(0.0);
    // vec3 c_cell = c_si_and_metal + c_via;
    // vec3 c_out = c_cell * (painted ? 1.0 : grid);

    // All done.
    // o_Target = vec4(c_out, 1.0);

    bool metal_connection = connection(
        tile_uv,
        metal_dir_up,
        metal_dir_right,
        metal_dir_down,
        metal_dir_left
    );

    // float tile = get_tile(
    //     tile_uv,
    //     metal,
    //     metal_dir_up,
    //     metal_dir_right,
    //     metal_dir_down,
    //     metal_dir_left
    // );

    float tile = (metal && metal_connection) ? 1.0 : 0.0;
    o_Target = vec4(tile, tile, grid * 0.1, 1.0);
}
