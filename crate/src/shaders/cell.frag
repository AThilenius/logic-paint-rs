#version 300 es
precision highp float;
precision highp int;
precision highp usampler2D;

in vec2 v_uv;

uniform float time;
uniform usampler2D cells_texture_sampler;
uniform usampler2D mask_texture_sampler;

// Style uniforms
uniform ivec2 chunk_start_cell_offset;
uniform vec4 n_color;
uniform vec4 p_color;
uniform vec3 metal_color;
uniform vec3 io_color;
uniform vec3 active_color;
uniform vec3 grid_color;
uniform vec3 background_color;
uniform float grid_blend_strength;
uniform float metal_over_si_blend;

// Selection
uniform vec3 cell_select_color;
uniform ivec2 cell_select_ll;
uniform ivec2 cell_select_ur;

// Cursor-follow
uniform vec3 cursor_follow_color;
uniform ivec2 cursor_coord;

out vec4 out_color;

bool connection(float l, vec2 texel_uv, bool up, bool right, bool down, bool left) {
    // Configure
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

bool connection_gate(
    vec2 texel_uv,
    bool horizontal,
    bool vertical,
    bool up_left,
    bool down_right
) {
    // Configure
    float l = 0.2;
    float h = 1.0 - l;

    // Configure
    float gl = 0.3;
    float gh = 1.0 - gl;

    float x = texel_uv.x;
    float y = texel_uv.y;

    return false
        || (vertical && (y < h && y > l && x > gl && x < gh))
        || (horizontal && (y < gh && y > gl && x > l && x < h))

        // Draw the side attachments the same as `connection`.
        || (y > h && x > l && x < h && up_left && vertical)
        || (y < l && x > l && x < h && down_right && vertical)
        || (x > h && y > l && y < h && down_right && horizontal)
        || (x < l && y > l && y < h && up_left && horizontal);
}

void main() {
    float epsilon = 0.00001;
    vec2 float_local_coord = clamp(
        v_uv * 32.0,
        vec2(0.0),
        vec2(32.0 - epsilon)
    );
    uvec2 local_coord = clamp(
        uvec2(floor(float_local_coord)),
        uvec2(0),
        uvec2(31)
    );
    ivec2 cell_coord = chunk_start_cell_offset + ivec2(local_coord);
    vec2 tile_uv = fract(float_local_coord);

    uvec4 cells = texelFetch(cells_texture_sampler, ivec2(local_coord), 0);
    uvec4 mask = texelFetch(mask_texture_sampler, ivec2(local_coord), 0);

    // Mirrors the format in upc.rs
    bool si_n = (cells.r & (1u << 7u)) > 0u;
    bool si_p = (cells.r & (1u << 6u)) > 0u;
    bool mosfet_horizontal = (cells.r & (1u << 5u)) > 0u;
    bool mosfet_vertical = (cells.r & (1u << 4u)) > 0u;
    bool si_dir_up = (cells.r & (1u << 3u)) > 0u;
    bool si_dir_right = (cells.r & (1u << 2u)) > 0u;
    bool si_dir_down = (cells.r & (1u << 1u)) > 0u;
    bool si_dir_left = (cells.r & (1u << 0u)) > 0u;

    bool metal = (cells.g & (1u << 7u)) > 3u;
    bool metal_dir_up = (cells.g & (1u << 6u)) > 0u;
    bool metal_dir_right = (cells.g & (1u << 5u)) > 0u;
    bool metal_dir_down = (cells.g & (1u << 4u)) > 0u;
    bool metal_dir_left = (cells.g & (1u << 3u)) > 0u;
    bool via = (cells.g & (1u << 2u)) > 0u;

    bool is_io = (cells.b & (1u << 7u)) > 0u;
    bool is_root = (cells.b & (1u << 6u)) > 0u;

    // Derrived values
    bool is_mosfet = mosfet_horizontal || mosfet_vertical;

    bool metal_active = (mask.r & (1u << 0u)) > 0u;
    bool gate_active = (mask.g & (1u << 0u)) > 0u;
    bool si_ul_active = (mask.b & (1u << 0u)) > 0u || (!is_mosfet && gate_active);
    bool si_dr_active = (mask.a & (1u << 0u)) > 0u || (!is_mosfet && gate_active);

    bool cell_selected =
        cell_coord.x >= cell_select_ll.x &&
        cell_coord.y >= cell_select_ll.y &&
        cell_coord.x < cell_select_ur.x &&
        cell_coord.y < cell_select_ur.y;

    bool cursor =
        cursor_coord.x == cell_coord.x ||
        cursor_coord.y == cell_coord.y;

    bool metal_connection = connection(
        0.35,
        tile_uv,
        metal_dir_up,
        metal_dir_right,
        metal_dir_down,
        metal_dir_left
    );

    bool si_connection = connection(
        0.2,
        tile_uv,
        si_dir_up,
        si_dir_right,
        si_dir_down,
        si_dir_left
    );

    bool gate_connection = connection_gate(
        tile_uv,
        mosfet_horizontal,
        mosfet_vertical,
        // Up-left
        (mosfet_vertical && si_dir_up) || (mosfet_horizontal && si_dir_left),
        // Down-right
        (mosfet_vertical && si_dir_down) || (mosfet_horizontal && si_dir_right)
    );

    vec2 stripe_uv = tile_uv * 2.0;
    float stripe_blend = smoothstep(
        0.5,
        0.6,
        mod(stripe_uv.x + stripe_uv.y + time, 2.0) * 0.5
    );

    bool grid_1 = (((local_coord.x % 2u) + (local_coord.y % 2u)) % 2u) == 0u;
    bool grid_8 = ((((local_coord.x >> 3) % 2u) + ((local_coord.y >> 3) % 2u)) % 2u) == 0u;
    float grid_blend =
          (grid_8 ? grid_blend_strength * 0.6 : 0.0)
        + (grid_1 ? grid_blend_strength : 0.0);

    vec3 si_color = si_n ? n_color.rgb : p_color.rgb;
    bool si_active = mosfet_vertical
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

    vec2 io_dist = tile_uv - vec2(0.5);
    float io_blend = 1.0 - smoothstep(
        0.1,
        0.3,
        dot(io_dist, io_dist) * 8.0
    );
    io_blend = is_io ? io_blend : 0.0;


    vec3 via_color = mix(si_color, vec3(1.0), 1.0);
    vec2 via_dist = tile_uv - vec2(0.5);
    float via_blend = 1.0 - smoothstep(
        0.1,
        0.3,
        dot(via_dist, via_dist) * 8.0
    );
    via_blend = via ? via_blend : 0.0;

    // Linear color blending.
    // Start with base (checker) color.
    vec3 color = mix(background_color, grid_color, grid_blend);

    // Cursor follow highlight.
    color = mix(
        color,
        cursor_follow_color,
        cursor ? 0.5 : 0.0
    );

    // Si totally overrides base color.
    color = mix(color, si_color, si_blend);

    // Gate totally overrides si
    color = mix(color, gate_color, gate_blend);

    // Metal is only blended if there is si.
    color = mix(
        color,
        blended_metal_color,
        si_blend > 0.5 ? metal_blend * metal_over_si_blend : metal_blend
    );

    // Vias are on or off.
    color = mix(color, via_color, via_blend);

    // I/O pins are drawn like Vias
    color = mix(color, io_color, io_blend);

    // Cell selection highlights the whole cell
    color = mix(
        color,
        cell_select_color,
        cell_selected ? 0.5 : 0.0
    );

    out_color = vec4(color, 1.0);
}
