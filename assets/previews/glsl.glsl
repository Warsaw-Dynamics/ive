#version 450 core
#extension GL_ARB_separate_shader_objects : enable
#define MAX_LIGHTS 4

/* Lights a fragment with up to MAX_LIGHTS point lights. */
struct Light {
    vec3 position;
    float intensity;
};

layout(location = 0) in vec3 vNormal;
layout(location = 1) in vec2 vUv;
layout(location = 0) out vec4 fragColor;

layout(std140, binding = 0) uniform Scene {
    Light lights[MAX_LIGHTS];
    int lightCount;
};
uniform sampler2D albedo;

// Lambertian term of one light.
float diffuse(in Light light, vec3 normal) {
    vec3 direction = normalize(light.position - gl_FragCoord.xyz);
    return max(dot(normal, direction), 0.0) * light.intensity;
}

void main() {
    vec3 color = texture(albedo, vUv).rgb;
    float total = 0.0;
    for (int i = 0; i < lightCount && i < MAX_LIGHTS; ++i) {
        total += diffuse(lights[i], normalize(vNormal));
    }
    if (total <= 0.0) discard;
    fragColor = vec4(color * total, 1.0);
}
