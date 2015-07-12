#version 410
#extension GL_ARB_shader_storage_buffer_object : require
#extension GL_ARB_shader_draw_parameters : require

// Uniforms / SSBO --------------------------------------------------------------------------------
layout (std140) buffer CB0
{
    mat4 transforms[];
};

uniform mat4 ViewProjection;

// Input ------------------------------------------------------------------------------------------
layout(location = 0) in vec3 position;
layout(location = 1) in vec2 tex_coords;

//  Output ----------------------------------------------------------------------------------------
out block {
    vec2 v2TexCoord;
    flat int iDrawID;
} Out;

// Functions ----------------------------------------------------------------------------------------------------------
void main()
{
    mat4 World = transforms[gl_DrawIDARB];
    vec4 worldPos = World * vec4(position, 1);
    gl_Position = ViewProjection * worldPos;
    
    Out.v2TexCoord = tex_coords;
    Out.iDrawID = gl_DrawIDARB;
}
