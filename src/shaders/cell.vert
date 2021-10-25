#version 300 es

in vec2 position;
in vec2 uv;

uniform mat4 view_proj;

out vec2 v_uv;

void main() {
    gl_Position = view_proj * vec4(position.x, position.y, 0.0, 1.0);
    v_uv = uv;
}
