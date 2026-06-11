#version 300 es
precision highp float;
precision highp int;

in vec2 v_world;

uniform ivec2 chunk_start_cell_offset;
uniform ivec2 cell_select_ll;
uniform ivec2 cell_select_ur;
uniform ivec2 cursor_coord;

const float CELLS_PER_CHUNK = 128.0;
const vec3 grid_color = vec3(1.0, 1.0, 1.0);
const vec3 background_color = vec3(0.0, 0.0, 0.0);
const vec3 cell_select_color = vec3(0.32, 0.6, 0.8);
const vec3 cursor_follow_color = vec3(0.0, 0.0, 0.0);
const float grid_blend_strength = 0.065;

out vec4 out_color;

void main() {
    vec2 cell_float = v_world * CELLS_PER_CHUNK;
    ivec2 cell_coord = ivec2(floor(cell_float));
    uvec2 local_coord = uvec2(cell_coord & ivec2(127));

    bool cell_selected =
        cell_coord.x >= cell_select_ll.x &&
        cell_coord.y >= cell_select_ll.y &&
        cell_coord.x < cell_select_ur.x &&
        cell_coord.y < cell_select_ur.y;

    bool cursor =
        cursor_coord.x == cell_coord.x ||
        cursor_coord.y == cell_coord.y;

    bool grid_1 = (((local_coord.x & 1u) + (local_coord.y & 1u)) & 1u) == 0u;
    bool grid_8 = ((((local_coord.x >> 3u) & 1u) + ((local_coord.y >> 3u) & 1u)) & 1u) == 0u;
    float grid_blend =
          (grid_8 ? grid_blend_strength * 0.6 : 0.0)
        + (grid_1 ? grid_blend_strength : 0.0);

    vec3 color = mix(background_color, grid_color, grid_blend);

    color = mix(
        color,
        cursor_follow_color,
        cursor ? 0.5 : 0.0
    );

    color = mix(
        color,
        cell_select_color,
        cell_selected ? 0.5 : 0.0
    );

    out_color = vec4(color, 1.0);
}
