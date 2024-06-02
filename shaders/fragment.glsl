#version 330 core
out vec4 FragColor;

in vec3 Normal;
in vec3 FragPos;
in vec2 TexCoords;

struct Material {
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
    float shininess;
};

struct Light {
    vec3 position;
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
};

uniform Material material;
uniform Light light;
uniform vec3 viewPos;
uniform sampler2D textureSampler;

varying vec3 barys;

void main()
{
    // Ambient
    vec3 ambient = light.ambient * material.ambient;

    // Diffuse
    vec3 norm = normalize(Normal);
    vec3 lightDir = normalize(light.position - FragPos);
    float diff = max(dot(norm, lightDir), 0.0);
    vec3 diffuse = light.diffuse * (diff * material.diffuse);

    // Specular
    vec3 viewDir = normalize(viewPos - FragPos);
    vec3 reflectDir = reflect(-lightDir, norm);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), material.shininess);
    vec3 specular = light.specular * (spec * material.specular);

    vec3 light = ambient + diffuse + specular;
    vec2 uv = (TexCoords % vec2(1.0));
    vec3 result = texelFetch(textureSampler, ivec2(uv * vec2(1080.0, 540.0)), 0).rgb * light;
    result = (vec3(TexCoords, 0.0) + result) / 2.0;
    result = vec3(TexCoords, 0.0);
    if (barys.x < 0.02 || barys.y < 0.02 || barys.z < 0.02) {
        result = vec3(0.0);
    }
//    if (TexCoords.x > 0.95 || TexCoords.x < 0.05) {
//        result = Normal;
//    }
//    if (TexCoords.x > 1.0) {
//        result += vec3(0.3);
//    }
//    else if (TexCoords.x > 0.98) {
//        result += vec3(0.3, 0.0, 0.0);
//    }
    FragColor = vec4(result, 1.0);
}
