// fxc.exe /E PS /T ps_5_0 /Fo "compiled-pixel-shader.shader" ./pixel.hlsl

struct VertexOut
{
    // In the output structure, ":SV_POSITION" and ":COLOR" are also semantics.
    // These are used to map the Vertex shader output to the inputs of the next stages
    // such as the geometry shader of the pixel shader.
    // Semantics prefixed with "SV" are special, it stands for "System Value".
    float4 PosH : SV_POSITION;
    float4 Color : COLOR;
};

// Pixel shader
float4 PS(float4 posH : SV_POSITION, float4 color : COLOR) : SV_Target
{
    return color;
}
