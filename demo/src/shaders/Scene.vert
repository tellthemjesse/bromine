#version 460 core

layout (location = 0) in vec3 v_Position;
layout (location = 1) in vec3 v_Normal;

layout (location = 0) out vec3 f_Position;
layout (location = 1) out vec3 f_Normal;

layout (location = 0) uniform mat4 u_View;
layout (location = 1) uniform mat4 u_Proj;

void main() {
    gl_Position = u_Proj * u_View * vec4(v_Position, 1.0);
    f_Normal = v_Normal;
}
