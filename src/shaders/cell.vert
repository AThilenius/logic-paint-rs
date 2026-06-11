#version 300 es

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 uv;

uniform mat4 view_proj;

// Chunk-space world position (1.0 = one chunk). Stable at extreme zoom.
out vec2 v_world;

void main() {
    gl_Position = view_proj * vec4(position.x, position.y, 0.0, 1.0);
    v_world = position;
}
