#version 450

layout(location=0) out vec4 color;
layout(location=1) in vec3 pos;

void main() {
    color = vec4(pos, 1.0);
}