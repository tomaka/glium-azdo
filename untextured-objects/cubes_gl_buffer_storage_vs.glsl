#version 410

#ifdef USE_SHADER_DRAW_PARAMETERS
#   extension GL_ARB_shader_draw_parameters : require
#endif

#extension GL_ARB_shader_storage_buffer_object : require

// Uniforms / SSBO ----------------------------------------------------------------------------------------------------
layout (std140, binding = 0) buffer CB0
{
    mat4 transforms[];
};

uniform mat4 ViewProjection;

// Input --------------------------------------------------------------------------------------------------------------
layout(location = 0) in vec3 position;
layout(location = 1) in vec3 color;
#ifndef USE_SHADER_DRAW_PARAMETERS
    layout(location = 2) in int draw_id;
#endif

#ifdef USE_SHADER_DRAW_PARAMETERS
#   define DrawID gl_DrawIDARB
#else 
#   define DrawID draw_id
#endif

// Output -------------------------------------------------------------------------------------------------------------
out block {
    vec3 v3Color;
} Out;

// Functions ----------------------------------------------------------------------------------------------------------
void main()
{
    mat4 World = transforms[DrawID];
    vec3 worldPos = vec3(World * vec4(position, 1));
    gl_Position = ViewProjection * vec4(worldPos, 1);
    Out.v3Color = color;
}
