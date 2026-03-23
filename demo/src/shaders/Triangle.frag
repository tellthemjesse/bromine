#version 460 core

layout (location = 0) in vec3 f_Color;
layout (location = 0) out vec4 FragColor;

layout (location = 2) uniform float u_Time;

void main() {
    mat3 transform = mat3(
        sin(u_Time), 0.0, 0.0,
        0.0, cos(u_Time), 0.0,
        0.0, 0.0, -sin(u_Time)
    );
    FragColor = vec4(abs(transform * f_Color), 1.0);
}
