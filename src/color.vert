#version 450

const vec2 positions[3] = vec2[3](
    vec2(0.0, 0.5),
    vec2(-0.5, -0.5),
    vec2(0.5, -0.5)
);
layout(location=1) out vec3 color_pos;

float dist(vec2 from, vec2 to) {
    float dx = to.x - from.x;
    float dy = to.y - from.y;
    return sqrt(dx * dx + dy * dy);
}

void main() {
    gl_Position = vec4(positions[gl_VertexIndex], 0.0, 1.0);

    float r = dist(positions[gl_VertexIndex], positions[0]);
    float g = dist(positions[gl_VertexIndex], positions[1]);
    float b = dist(positions[gl_VertexIndex], positions[2]);

    color_pos = vec3(r, g, b);
}