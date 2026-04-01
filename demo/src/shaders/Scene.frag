#version 460 core

layout (location = 0) in vec3 f_Position;
layout (location = 1) in vec3 f_Normal;

layout (location = 0) out vec4 FragColor;

layout (location = 2) uniform float u_Time;
layout (location = 3) uniform vec3 u_ViewPos;

const vec3 LightPos = vec3(0.0, -4.0, 0.0);
const vec3 LightColor = vec3(1.0, 1.0, 0.0);

const float Ambient = 0.2; 
const float Specular = 0.5;

void main() {
    vec3 lightPos = vec3(8.0 * cos(u_Time), LightPos.y, 8.0 * sin(u_Time));

    vec3 ambient = Ambient * LightColor;
    
    vec3 normal = normalize(f_Normal);
    vec3 lightDir = normalize(lightPos - f_Position);
    float diff = max(dot(normal, lightDir), 0.0);
    vec3 diffuse = diff * LightColor;
    
    vec3 viewDir = normalize(u_ViewPos - f_Position);
    vec3 reflectDir = reflect(-lightDir, normal);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), 64);
    vec3 specular = Specular * spec * LightColor;
    
    vec3 result = (ambient + diffuse + specular) * vec3(0.65, 0.16, 0.16);
    FragColor = vec4(result, 1.0);
}
