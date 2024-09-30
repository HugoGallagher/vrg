#version 450

layout(location = 0) out vec2 coord;

vec2 pos[6] = vec2[](
    vec2(-1.0, -1.0),
    vec2(1.0, -1.0),
    vec2(1.0, 1.0),
    vec2(-1.0, -1.0),
    vec2(1.0, 1.0),
    vec2(-1.0, 1.0)
);

void main() {
    gl_Position = vec4(pos[gl_VertexIndex], 0.0, 1.0);
    coord = (pos[gl_VertexIndex] + 1.0) / 2.0;
}