// NOTE: Unlike every other set of Solutions, all of the StreamingVBs currently use this shader.
// If you need to make modifications for a particular test, create a new shader. 
#version 410

// Uniforms / SSBO ----------------------------------------------------------------------------------------------------
layout(std140) uniform CB0
{
    vec2 viewport;
};

// Input --------------------------------------------------------------------------------------------------------------
layout(location = 0) in vec2 position;

// Output -------------------------------------------------------------------------------------------------------------

// Functions ----------------------------------------------------------------------------------------------------------
void main()
{
    gl_Position = vec4(position * viewport + vec2(-1, 1), 0, 1);
}
