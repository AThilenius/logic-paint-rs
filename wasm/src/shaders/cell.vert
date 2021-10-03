#version 300 es

in vec2 position_uv;

uniform mat4 view_proj;
uniform mat4 model;

out vec2 v_uv;

void main() {
    gl_Position = view_proj * model * vec4(position_uv.x, position_uv.y, 0.0, 1.0);
    v_uv = position_uv;
}
