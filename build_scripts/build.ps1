$build_script_directory = "${PSScriptRoot}\.."

# cargo build
Write-Host ""
Write-Host "**** BUILDING APPLICATION ****"
Write-Host ""

cargo build 

# Compile Shaders
Write-Host ""
Write-Host "**** COMPILING SHADERS ****"
Write-Host ""

# Compile Vertex Shader
fxc.exe /E VS /T vs_5_0 /Fo "${build_script_directory}\shaders\compiled-vertex-shader.shader" "${build_script_directory}\shaders\vertex_perspective.hlsl"

# Compile Pixel Shader
fxc.exe /E PS /T ps_5_0 /Fo "${build_script_directory}\shaders\compiled-pixel-shader.shader" "${build_script_directory}\shaders\pixel.hlsl"

# Copy compiled shaders to output directory
Write-Host ""
Write-Host "**** COPYING SHADERS TO OUTPUT DIR ****"
Write-Host ""

Copy-Item -Path "${build_script_directory}\shaders" -Recurse -Destination "${build_script_directory}\target\debug\resources" -Force
