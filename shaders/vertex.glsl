#version 330 core
layout (location = 0) in vec3 position;
layout (location = 1) in vec3 normal;  // Input for normals
layout (location = 2) in vec2 texCoords;
layout (location = 3) in int index;

out vec3 FragPos;
out vec3 Normal;
out vec2 TexCoords;

varying vec3 barys;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

const float SUBDIVISIONS = 1.0;

void main()
{
    int i = gl_VertexID  % 3;
    switch (i) {
        case 0:
            barys = vec3(SUBDIVISIONS, 0.0, 0.0);
            break;
        case 1:
            barys = vec3(0.0, SUBDIVISIONS, 0.0);
            break;
        case 2:
            barys = vec3(0.0, 0.0, SUBDIVISIONS);
            break;
    }


    FragPos = vec3(model * vec4(position, 1.0)); // Position in world space
    Normal = mat3(transpose(inverse(model))) * normal; // Transform normals
    TexCoords = texCoords;
    gl_Position = projection * view * model * vec4(position, 1.0);
}
