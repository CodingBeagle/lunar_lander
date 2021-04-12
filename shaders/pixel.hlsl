struct VertexOut
{
    // In the output structure, ":SV_POSITION" and ":COLOR" are also semantics.
    // These are used to map the Vertex shader output to the inputs of the next stages
    // such as the geometry shader of the pixel shader.
    // Semantics prefixed with "SV" are special, it stands for "System Value".
    float4 PosH : SV_POSITION;
    float4 Color : COLOR;
};

// Semantics prefixed with "SV" are special, it stands for "System Value".
float4 PS(float4 posH : SV_POSITION, float4 color : COLOR) : SV_Target
{
    return color;
}
