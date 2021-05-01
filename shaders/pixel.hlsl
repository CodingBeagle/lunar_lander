// fxc.exe /E PS /T ps_5_0 /Fo "compiled-pixel-shader.shader" ./pixel.hlsl

Texture2D <float4> mesh_texture;

SamplerState mySampleState {
    Filter = D3D11_FILTER_MIN_MAG_MIP_POINT;
};

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
void PS(float4 posH : SV_POSITION, float4 color : COLOR, float2 uvo : UV, out float4 col : SV_TARGET) 
{
    col = mesh_texture.Sample(mySampleState, uvo);
    //depth = 0.2f;
    //col = float4(0.5, 0.5, 0.5, 1.0);
}
