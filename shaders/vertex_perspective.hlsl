// COMPILED OFFLINE USING:  fxc.exe /E VS /T vs_5_0 /Fo "compiled-vertex-shader.shader" ./vertex.hlsl

// DirectX shaders are written in the HLSL (high level shading language) language.
// These are text files saved in the .fx format.

// a "cbuffer" is a Constant Buffer.
// Constant buffers are blocks of memory which can store variables which can be 
// Accessed by a shader.
// Data is constant buffers doesn't vary per vertex, but stays the same.
cbuffer cbPerObject : register(b0)
{
    float4x4 worldViewProjection;
};

// The input structure coming from the pipeline
struct VertexIn
{
    // :POSITION and :COLOR are parameter semantics which are mapping from the vertex buffer in the
    // pipeline.
    float3 PosL : POSITION;
    float4 Color : COLOR;
};

void VS(float3 PosL : POSITION, float4 iColor : COLOR, 
                out float4 PosH : SV_POSITION, out float4 oColor : COLOR)
{
    // Transform to homogenous clip space
    // Notice that the vertex shader, or any other shader, doesn't do the perspective divide.
    // The perspective divide is done by hardware at a later stage.
    // The vertex shader just does the projection matrix.
    PosH = mul(float4(PosL, 1.0f), worldViewProjection);

    // Just pass vertex color into the pixel shader
    oColor = float4(1.0f, 0.0f, 0.0f, 1.0f);
}
