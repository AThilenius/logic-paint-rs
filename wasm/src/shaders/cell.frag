#version 300 es
precision highp float;
precision highp int;
precision highp usampler2D;

in vec2 v_uv;

uniform usampler2D cells_texture_sampler;

out vec4 out_color;

void main() {
    // Must match size in substrate_render_chunk.rs
    float grid_res = 32.0;
    uvec2 texel = uvec2(floor(v_uv * grid_res));

    // This math was taken from:
    // https://gamedev.stackexchange.com/questions/135282/any-way-to-combine-instantiated-sprite-renderers-into-one-texture-so-i-can-apply/135307#135307
    vec2 canvas_location = v_uv * grid_res;
    vec2 tile_uv = fract(canvas_location);
    canvas_location = (canvas_location - tile_uv) / grid_res;
    tile_uv = tile_uv * 126.0 / 128.0 + 1.0 / 128.0;

    uvec4 cells = texelFetch(cells_texture_sampler, ivec2(texel), 0);
    out_color = vec4(cells) / 255.0;
}
