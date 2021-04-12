// DirectX shaders are written in the HLSL (High Level Shading Language) language.
// These are text files saved in the .hlsl format.

// A "cbuffer" is a Constant Buffer.
// Constant buffers are blocks of memory which can store variables which can be accessed by a shader.
// Data in constant buffers do not vary per vertex, but stays the same.
cbuffer cbPerObject
{
    float4x4 gWorldViewProj;
}

// The input structure coming from the pipeline
struct VertexIn
{
    // :POSITION and :COLOR are parameter semantics which are mapping from the vertex buffer in the pipeline.
    float3 PosL : POSITION;
    float4 Color: COLOR;
};

void VS(float3 PosL : POSITION, float4 iColor : COLOR,
		out float4 PosH : SV_POSITION, out float4 oColor : COLOR)
{
    // Transform to homogenous clip space.
    // Notice that the vertex shader, or any other shader, doesn't do the perspective divide.
    // The perspective divide is done by hardware at a later stage.
    // The vertex shader just does the projection matrix.
    PosH = mul(float4(PosL, 1.0f), gWorldViewProj);

    // Just pass the vertex color into the pixel shader
    oColor = float4(1.0f, 0.0f, 0.0f, 1.0f);
}
