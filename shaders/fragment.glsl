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

vec3 get_hex_center() {
    float X = ceil(barys.x) - barys.x;
    float Y = ceil(barys.y) - barys.y;
    float Z = ceil(barys.z) - barys.z;
    vec3 hex = floor(barys);
    if (X <= Y && X <= Z) {
        hex.x = ceil(barys.x);
    }
    else if (Y <= Z && Y <= X) {
        hex.y = ceil(barys.y);
    }
    else if (Z <= Y && Z <= X) {
        hex.z = ceil(barys.z);
    }
    bool invalid_hex = (hex.x+hex.y+hex.z) != 10.0;
    if (invalid_hex) {
        if (X <= Y || X <= Z) {
            hex.x = ceil(barys.x);
        }
        if (Y <= Z || Y <= X) {
            hex.y = ceil(barys.y);
        }
        if (Z <= Y || Z <= X) {
            hex.z = ceil(barys.z);
        }
    }
    return hex;
}

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

//    result = (vec3(TexCoords, 0.0) + result) / 2.0;
//    result = vec3(TexCoords, 0.0);
//    result = Normal;

    if (barys.x < 0.05 || barys.y < 0.05 || barys.z < 0.05) {
        result *= vec3(0.5);
    }

    vec3 hex = get_hex_center();
    // Draw borders where this pixel is not that much further from neighbor tiles
    float D = distance(hex, barys);
    float B1 = abs(D - distance(hex + vec3(1., 0., -1.), barys));
    float B2 = abs(D - distance(hex + vec3(1., -1., -0.), barys));
    float B3 = abs(D - distance(hex + vec3(0., -1., 1.), barys));
    float B4 = abs(D - distance(hex + vec3(-1., 0., 1.), barys));
    float B5 = abs(D - distance(hex + vec3(-1., 1., 0.), barys));
    float B6 = abs(D - distance(hex + vec3(0., 1., -1.), barys));
    float border = min(B1, min(B2, min(B3, min(B4, min(B5, B6)))));
    float a = 1.0 - sqrt(border);
    a = max(0.0, a);
    result -= a / 10.0;

//    if (TexCoords.x > 1.0) {
//        result += vec3(0.3);
//    }
//    else if (TexCoords.x > 0.98) {
//        result += vec3(0.3, 0.0, 0.0);
//    }

    FragColor = vec4(result, 1.0);
}
