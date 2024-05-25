#version 330 core
layout (location = 0) in vec3 position;
layout (location = 2) in vec2 texCoords;
layout (location = 1) in vec3 normal;  // Input for normals

out vec3 FragPos;
out vec3 Normal;
out vec2 TexCoords;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

void main()
{
    FragPos = vec3(model * vec4(position, 1.0)); // Position in world space
    Normal = mat3(transpose(inverse(model))) * normal; // Transform normals
    TexCoords = texCoords;
    gl_Position = projection * view * model * vec4(position, 1.0);
}
