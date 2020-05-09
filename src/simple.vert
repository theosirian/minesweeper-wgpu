#version 450

layout(location=0) in vec3 pos;

layout(set=0, binding=0)
uniform Uniforms {
    vec3 color;
};

layout(location=0) out vec3 v_color;

void main() {
    gl_Position = vec4(pos, 1.0);
    v_color = color;
}