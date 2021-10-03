#version 300 es
precision highp float;

in vec3 Vertex_Position;
in vec2 Vertex_Uv;
out vec2 v_Uv;

layout(std140) uniform CameraViewProj { // set = 0, binding = 0
    mat4 ViewProj;
};
layout(std140) uniform Transform { // set = 1, binding = 0
    mat4 Model;
};

void main() {
    gl_Position = ViewProj * Model * vec4(Vertex_Position, 1.0);
    v_Uv = Vertex_Uv;
}
