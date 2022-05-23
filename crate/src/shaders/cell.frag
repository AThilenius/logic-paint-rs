#version 300 es
precision highp float;
precision highp int;
precision highp usampler2D;

in vec2 v_uv;

uniform float time;
uniform usampler2D cells_texture_sampler;
uniform usampler2D mask_texture_sampler;

// Style uniforms
uniform vec4 n_color;
uniform vec4 p_color;
uniform vec3 metal_color;
uniform vec3 io_color;
uniform vec3 active_color;
uniform vec3 grid_color;
uniform vec3 background_color;
uniform vec2 grid_res;
uniform float grid_blend_strength;
uniform float metal_over_si_blend;

out vec4 out_color;

bool connection(vec2 texel_uv, bool up, bool right, bool down, bool left) {
    // Configure
    float l = 0.2;
    float h = 1.0 - l;

    float x = texel_uv.x;
    float y = texel_uv.y;

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
    float y = texel_uv.y;

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
    uvec2 texel = uvec2(floor(v_uv * grid_res));

    // This math was taken from:
    // https://gamedev.stackexchange.com/questions/135282/any-way-to-combine-instantiated-sprite-renderers-into-one-texture-so-i-can-apply/135307#135307
    vec2 canvas_location = v_uv * grid_res;
    vec2 tile_uv = fract(canvas_location);
    canvas_location = (canvas_location - tile_uv) / grid_res;
    tile_uv = tile_uv * 126.0 / 128.0 + 1.0 / 128.0;

    uvec4 cells = texelFetch(cells_texture_sampler, ivec2(texel), 0);
    uvec4 mask = texelFetch(mask_texture_sampler, ivec2(texel), 0);

    // Mirrors the format in upc.rs
    bool si_n = (cells.r & (1u << 7u)) > 0u;
    bool si_p = (cells.r & (1u << 6u)) > 0u;
    bool si_dir_up = (cells.r & (1u << 5u)) > 0u;
    bool si_dir_right = (cells.r & (1u << 4u)) > 0u;
    bool si_dir_down = (cells.r & (1u << 3u)) > 0u;
    bool si_dir_left = (cells.r & (1u << 2u)) > 0u;
    bool gate_dir_up = (cells.r & (1u << 1u)) > 0u;
    bool gate_dir_right = (cells.r & (1u << 0u)) > 0u;

    bool gate_dir_down = (cells.g & (1u << 7u)) > 0u;
    bool gate_dir_left = (cells.g & (1u << 6u)) > 0u;
    bool metal = (cells.g & (1u << 5u)) > 0u;
    bool metal_dir_up = (cells.g & (1u << 4u)) > 0u;
    bool metal_dir_right = (cells.g & (1u << 3u)) > 0u;
    bool metal_dir_down = (cells.g & (1u << 2u)) > 0u;
    bool metal_dir_left = (cells.g & (1u << 1u)) > 0u;
    bool via = (cells.g & (1u << 0u)) > 0u;

    bool is_io = (cells.b & (1u << 7u)) > 0u;

    bool is_mosfet =
        gate_dir_up ||
        gate_dir_right ||
        gate_dir_down ||
        gate_dir_left;

    bool metal_active = (mask.r & (1u << 0u)) > 0u;
    bool gate_active = (mask.g & (1u << 0u)) > 0u;
    bool si_ul_active = (mask.b & (1u << 0u)) > 0u || (!is_mosfet && gate_active);
    bool si_dr_active = (mask.a & (1u << 0u)) > 0u || (!is_mosfet && gate_active);

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
        mod(stripe_uv.x + stripe_uv.y + time, 2.0) * 0.5
    );

    bool grid_1 = (((texel.x % 2u) + (texel.y % 2u)) % 2u) == 0u;
    bool grid_8 = ((((texel.x >> 3) % 2u) + ((texel.y >> 3) % 2u)) % 2u) == 0u;
    float grid_blend =
          (grid_8 ? grid_blend_strength * 0.6 : 0.0)
        + (grid_1 ? grid_blend_strength : 0.0);

    vec3 si_color = si_n ? n_color.rgb : p_color.rgb;
    bool vertical_gate = gate_dir_up || gate_dir_down;
    bool si_active = vertical_gate
        ?  (tile_uv.x < 0.5 ? si_ul_active : si_dr_active)
        :  (tile_uv.y > 0.5 ? si_ul_active : si_dr_active);
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

    vec3 blended_metal_color = mix(
        metal_color,
        active_color,
        metal_active ? stripe_blend * 0.5 : 0.0
    );
    float metal_blend = metal && metal_connection ? 1.0 : 0.0;

    vec3 blended_io_color = mix(
        io_color,
        active_color,
        metal_active ? stripe_blend * 0.5 : 0.0
    );

    vec3 via_color = mix(si_color, vec3(1.0), 1.0);
    vec2 via_dist = tile_uv - vec2(0.5);
    float via_blend = 1.0 - smoothstep(
        0.1,
        0.3,
        dot(via_dist, via_dist) * 8.0
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
        blended_metal_color,
        si_blend > 0.5 ? metal_blend * metal_over_si_blend : metal_blend
    );

    // And I/O overrides all of it and fills the entire cell.
    vec3 with_io_color = mix(
        with_metal_color,
        blended_io_color,
        is_io ? 1.0 : 0.0
    );

    // Vias are on or off.
    vec3 with_via_color = mix(with_io_color, via_color, via_blend);

    out_color = vec4(with_via_color, 1.0);
}