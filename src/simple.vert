#version 450

layout(location=0) in vec3 pos;

layout(location=0) out vec2 v_tex_coord;

layout(set = 1, binding = 0) uniform Uniforms {
   vec3 color;
};

void main() {
    gl_Position = vec4(pos, 1.0);
    v_tex_coord = vec2((pos.x + 1.0) / 2.0, 1.0 - (pos.y + 1.0) / 2.0);
}