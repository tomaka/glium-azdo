#version 410
#extension GL_ARB_shader_storage_buffer_object : require
#extension GL_ARB_bindless_texture : require
#extension GL_ARB_shader_image_load_store : require

// Uniforms / SSBO ----------------------------------------------------------------------------------------------------
layout (std430) buffer CB1
{
    sampler2D tex_address[];
};

// Input --------------------------------------------------------------------------------------------------------------
in block {
    vec2 v2TexCoord;
    flat int iDrawID;
} In;

// Output -------------------------------------------------------------------------------------------------------------
layout(location = 0) out vec4 Out_v4Color;

// Functions ----------------------------------------------------------------------------------------------------------
void main()
{
    sampler2D smplr = tex_address[In.iDrawID];
    Out_v4Color = vec4(texture(smplr, In.v2TexCoord).xyz,  1);
}
