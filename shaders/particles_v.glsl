#version 330
layout (location = 0) in vec4 inPosition;
layout (location = 1) in vec4 inVelocity;  // Input for normals

out vec4 outPosition;
out vec4 outVelocity;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

void main() {
    float deltaTime = 0.016; // Assuming 60 FPS, for example
    vec3 toOrigin = -vec3(inPosition);
    vec3 gravityDirection = normalize(toOrigin);
    float gravityMagnitude = 0.0981; // Example constant gravity magnitude
    vec3 gravity = gravityMagnitude * gravityDirection;
    vec3 newVelocity = vec3(inVelocity) + gravity * deltaTime;

    // Update position based on new velocity
    outPosition = inPosition + vec4(newVelocity, 0.0) * deltaTime;
    outVelocity = vec4(newVelocity, 0.0);

    // Transform position to clip space
    gl_Position = projection * view * model * outPosition;
}
