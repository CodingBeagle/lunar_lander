// Mod Decs
mod beagle_math;

use std::{convert::TryInto, env, ffi::{CString, OsStr}, fs, iter::once, mem, path::PathBuf, ptr::null_mut};

// rust-analyzer has an issue with unresolved import errors for platform specific modules such as std::os

use std::os::windows::prelude::*;

extern crate winapi;

use obj_loader::Vertex;
// winapi related imports
use winapi::ctypes::*;
use winapi::{Interface, um::libloaderapi::*};
use winapi::shared::windef::*;
use winapi::shared::minwindef::*;
use winapi::um::winuser::*;
use winapi::um::unknwnbase::*;
use winapi::shared::winerror::*;
use winapi::shared::dxgi::*;
use winapi::shared::dxgiformat::*;
use winapi::shared::dxgitype::*;
use winapi::shared::windowsx::*;

// Beagle Math
use beagle_math::*;

// Nalgebra
extern crate nalgebra_glm as glm;
use glm::*;

// DirectX related imports
use winapi::um::d3d11::*;
use winapi::um::d3dcommon::*;

// Own modules
pub mod obj_loader;

// Std
use std::collections::HashMap;

// Image
use image::{ImageResult, DynamicImage, io::Reader};

// num
extern crate num;
#[macro_use]
extern crate num_derive;

#[derive(Debug)]
#[repr(C)]
struct VertexConstantBuffer {
    worldViewProjection: Mat4 
}

// TODO: Hot damn... really need to read up on all these fancy traits!
#[derive(Debug)]
#[derive(FromPrimitive)]
#[derive(Eq)]
#[derive(PartialEq)]
#[derive(Hash)]
enum KeyType {
    W = 0x57,
    S = 0x53,
    A = 0x41,
    D = 0x44,
    Q = 0x51,
    Y = 0x59,
    E = 0x45,
    Up = VK_UP as isize,
    Down = VK_DOWN as isize,
    Left = VK_LEFT as isize,
    Right = VK_RIGHT as isize
}

#[derive(Debug)]
#[derive(Eq)]
#[derive(PartialEq)]
#[derive(Hash)]
#[derive(Copy, Clone)]
enum MouseKey {
    Left,
    Right,
    Middle
}

#[derive(Debug)]
#[derive(Copy, Clone)]
#[derive(PartialEq)]
enum KeyState {
    Up,
    Down
}

// Abstraction for winapi Window
#[derive(Default)]
struct Window {
    current_keyboard_state: HashMap<KeyType, bool>,
    previous_keyboard_state: HashMap<KeyType, bool>,
    current_mouse_key_state: HashMap<MouseKey, KeyState>,
    previous_mouse_key_state: HashMap<MouseKey, KeyState>,
    mouse_coords: Vec2
}

impl Window {
    fn is_key_pressed(&self, key: KeyType) -> bool {
        match self.current_keyboard_state.get(&key) {
            Some(ispressed) => *ispressed,
            None => false
        }
    }

    fn proc(&mut self, keytype: KeyType, is_pressed: bool) {
        let ispressed = self.current_keyboard_state.entry(keytype).or_insert(is_pressed);
        *ispressed = is_pressed;
    }

    fn update_mouse_position(&mut self, x: i32, y: i32) {
        self.mouse_coords.x = x as f32;
        self.mouse_coords.y = y as f32;
    }

    fn is_mouse_down(&self, mouse_key: MouseKey) -> bool {
        match self.current_mouse_key_state.get(&mouse_key) {
            Some(state) => *state == KeyState::Down,
            None => false
        }
    }

    fn was_mouse_pressed(&self, mouse_key: MouseKey) -> bool {
        let current_key_state = match self.current_mouse_key_state.get(&mouse_key) {
            Some(state) => *state,
            None => KeyState::Up
        };

        let previous_key_state = match self.previous_mouse_key_state.get(&mouse_key) {
            Some(state) => *state,
            None => KeyState::Up
        };

        current_key_state == KeyState::Down && previous_key_state == KeyState::Up 
    }

    fn was_key_pressed(&self, key: KeyType) -> bool {
        let is_currently_pressed = match self.current_keyboard_state.get(&key) {
            Some(state) => *state,
            None => false
        };

        let was_previously_pressed = match self.previous_keyboard_state.get(&key) {
            Some(state) => *state,
            None => false
        };

        is_currently_pressed == true && was_previously_pressed == false
    }

    fn update_mouse_key_state(&mut self, mouse_key: MouseKey, key_state: KeyState) {
        let state = self.current_mouse_key_state.entry(mouse_key).or_insert(key_state);
        *state = key_state;
    }

    fn update(&mut self) {
        // TODO: clone() actually seems to work... but gotta read up on the technicalities of clone vs copy
        self.previous_mouse_key_state = self.current_mouse_key_state.clone();
    }
}

fn main() {
    // TODO: Read up on unsafe block
    unsafe {
        // TODO: Still need to figure out what this long chain of stuff actually does...
        let class_name : Vec<u16> = OsStr::new("mainwindow").encode_wide().chain( once(0) ).collect();
        let h_instance = GetModuleHandleW(null_mut());

        let window_class = WNDCLASSEXW {
            // TODO: Read up on generics in Rust (including it's weird syntax)
            cbSize: mem::size_of::<WNDCLASSEXW>() as UINT,
            style: CS_HREDRAW | CS_VREDRAW,
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: h_instance,
            hIcon: null_mut(),
            hIconSm: null_mut(),
            hCursor: LoadCursorW(h_instance, IDC_ARROW),
            hbrBackground: null_mut(),
            lpszMenuName: null_mut(),
            lpszClassName: class_name.as_ptr(),
            lpfnWndProc: Some(window_proc)
        };

        if RegisterClassExW(&window_class) == 0 {
            println!("Failed to register window class!");
            return
        }

        let window_title : Vec<u16> = OsStr::new("Lunar Lander").encode_wide().chain( once(0) ).collect();

        let main_window = CreateWindowExW(
           0,
           class_name.as_ptr(),
           window_title.as_ptr(),
           WS_OVERLAPPEDWINDOW,
           CW_USEDEFAULT, CW_USEDEFAULT, 800, 600,
           null_mut(),
           null_mut(),
           h_instance,
           null_mut());

        if main_window.is_null() {
            println!("Failed to create window!");
            return
        }

        ShowWindow(main_window, SW_SHOW);

        let mut window_helper = Window::default();

        // Set window user data
        // TODO: Need to read up on this cast magic...
        SetWindowLongPtrA(main_window, GWLP_USERDATA, &window_helper as *const _ as isize);

        // Initializing DirectX starts with creating an ID3D11Device and an ID3D11DeviceContext.
        // These are the two primary interfaces of DirectX 11, helping us to interface with the
        // GPU.

        // The ID3D11Device is used to check feature support and allocate resources.
        let mut device: *mut ID3D11Device = null_mut();

        // The ID3D11DeviceContext is used to set render states, bind resources to the graphics
        // pipeline, issue rendering commands, etc.
        let mut immediate_device_context: *mut ID3D11DeviceContext = null_mut();

        // Array of feature levels to support.
        // https://docs.microsoft.com/en-us/windows/win32/api/d3dcommon/ne-d3dcommon-d3d_feature_level
        // Basically, in DirectX 11, what features of DirectX a video card supports is described in
        // terms of feature levels.
        // A feature level is a well-defined set of GPU functionality.
        // When you do device creation, you attempt to create a device for a certain feature level.
        // If device creation fails, it might be that the feature level you request is not
        // supported by the GPU.
        let requested_feature_levels = [
            D3D_FEATURE_LEVEL_11_0 // So far, I only check for DirectX 11 support.
        ];

        // Actual D3D_FEATURE_LEVEL values are defined in d3dcommon.h
        // https://github.com/apitrace/dxsdk/blob/master/Include/d3dcommon.h
        let mut selected_feature_level : D3D_FEATURE_LEVEL = 0;

        // Attempt to create the device and device context.
        let device_result = D3D11CreateDevice(
            // Pointer to the video adapter. This is left null, meaning it will pick the first
            // adapter enumerated by EnumAdapters.
            // The video adapter is the GPU.
            null_mut(),
            // The Driver type.
            // I specify hardware to get hardware accelerated DirectX3D features.
            // https://docs.microsoft.com/en-us/windows/win32/api/d3dcommon/ne-d3dcommon-d3d_driver_type
            D3D_DRIVER_TYPE_HARDWARE,
            // Handle to a DLL that implements a software rasterizer. Only relevant if you chose a
            // software driver type.
            null_mut(),
            // Runtime layers to enable.
            // https://docs.microsoft.com/en-us/windows/win32/direct3d11/overviews-direct3d-11-devices-layers
            D3D11_CREATE_DEVICE_DEBUG,
            // Requested feature levels.
            requested_feature_levels.as_ptr(),
            // Number of elements in the requested feature levels array,
            requested_feature_levels.len() as u32,
            // You should always specify D3D11_SDK_VERSION for the SDKVersion parameter
            D3D11_SDK_VERSION,
            &mut device,
            &mut selected_feature_level,
            &mut immediate_device_context
            );

        if FAILED(device_result) {
            println!("Failed to create device and device context!");
            return
        }

        if selected_feature_level != D3D_FEATURE_LEVEL_11_0 {
            println!("DirectX 11 support is required, but this device does not support it! The highest selected feature level was: {}", selected_feature_level);
            return
        }

        // Reference to our Device, so we can actually call the functions
        // TODO: Read up on difference in Rust between pointers and references, because we cannot call any methods on "device" without first
        // obtaining the reference through as_ref().
        let device_ref = device.as_ref().unwrap();
 
        // After having created a Device and DeviceContext, the next step is creating a swap chain.
        // A swap chain consists of a front buffer and a back buffer.
        // The back buffer is used to draw / render an entire frame whilst the front buffer is
        // being displayed on the monitor.
        // Once the back buffer is finished drawing, it now becomes the front buffer through a
        // flip, and it is displayed on the monitor.
        // This technique is used to avoid "tearing", the visual artifact where a user would see a
        // frame being drawn as it is happening.
        // With a swap chain, only complete frames are rendered.
        // The act of swapping between the front- and back buffer is called PRESENTING.
        let mut swap_chain_description = create_swap_chain_description(main_window);

        // In order to create the swap chain, we need to call CreateSwapChain on a IDXGIFactory object.
        // An IDXGIFactory is used to create objects related to the DXGI technology.
        // The issue is that the IDXGIFactory required is the one which was implicitly used to create our device when calling D3D11CreateDevice.
        let mut idxgi_device : *mut IDXGIDevice = null_mut();
        if FAILED(device_ref.QueryInterface(&IDXGIDevice::uuidof(), &mut idxgi_device as *mut *mut _ as *mut *mut c_void)) {
            println!("Failed to obtain IDXGIDevice.");
            return
        }

        let mut idxgi_adapter : *mut IDXGIAdapter = null_mut();
        if FAILED(idxgi_device.as_ref().unwrap().GetParent(&IDXGIAdapter::uuidof(), &mut idxgi_adapter as *mut *mut _ as *mut *mut c_void)) {
            println!("Failed to obtain IDXGIAdapter.");
            return
        }

        let mut idxgi_factory : *mut IDXGIFactory = null_mut();
        if FAILED(idxgi_adapter.as_ref().unwrap().GetParent(&IDXGIFactory::uuidof(), &mut idxgi_factory as *mut *mut _ as *mut *mut c_void)) {
            println!("Failed to obtain IDXGIFactory.");
            return
        }

        // Now that we have obtained the IDXGI factory which was also used to create our device, we can create the swapchain from that factory.
        let mut idxgi_swap_chain : *mut IDXGISwapChain = null_mut();

        if FAILED(idxgi_factory.as_ref().unwrap().CreateSwapChain(device as *mut IUnknown, &mut swap_chain_description, &mut idxgi_swap_chain)) {
            println!("Failed to create the swap chain");
            return
        }

        // Now we can release the COM interfaces that we don't need any longer
        // TODO: See more on COM reference counting here: https://docs.microsoft.com/en-us/cpp/atl/queryinterface?view=msvc-160
        idxgi_device.as_ref().unwrap().Release();
        idxgi_adapter.as_ref().unwrap().Release();
        idxgi_factory.as_ref().unwrap().Release();

        // Creating Render Target View for swap chain backbuffer
        // We need to bind the back buffer of our swap chain to the Output Merger Stage (so the back buffer can be rendered to by the pipeline).
        // In order to do this, we need to create a Render Target View, and bind that view to the pipeline.
        let mut back_buffer_view : *mut ID3D11RenderTargetView = null_mut();
        let mut back_buffer : *mut ID3D11Texture2D = null_mut();

        // IDXGISwapChain::GetBuffer is used to access a swap chain's back buffers
        // Since the SwapEffect of the chain is DXGI_SWAP_EFFECT_DISCARD, we only have access to the first buffer, index zero.
        if FAILED(idxgi_swap_chain.as_ref().unwrap().GetBuffer(0, &ID3D11Texture2D::uuidof(), &mut back_buffer as *mut *mut _ as *mut *mut c_void)) {
            println!("Failed to get swap chain back buffer!");
            return
        }

        // ID3D11Device::CreateRenderTargetView creates a render-target view for accessing resource data.
        // pResource = A pointer to the resource that represents a render target, in this case our swap-chain backbuffer.
        // Render Target Views can be used to bind to the Output Merger Stage
        if FAILED(device_ref.CreateRenderTargetView(back_buffer as *mut ID3D11Resource, null_mut(), &mut back_buffer_view)) {
            println!("Failed to create a render target view from the swap chain back buffer!");
            return
        }

        // We don't need a COM object to the swap-chain back buffer any longer
        back_buffer.as_ref().unwrap().Release();

        // TODO: Read up on Depth/Stencil buffer
        // Creation of depth/stencil buffer
        // A depth/stencil buffer is simply a 2D texture used to store depth information.
        // It's used by the Output Merger Stage to determine which pixels should be visible, and which shouldn't.

        // In order to create a 2D texture, we fill out a D3D11_TEXTURE2D_DESC struct
        let mut depth_buffer_texture_description = D3D11_TEXTURE2D_DESC::default();

        // The width and height of the texture in Texels.
        // It should be the same size as the back buffer we display in our window.
        depth_buffer_texture_description.Width = 800;
        depth_buffer_texture_description.Height = 600;

        // The number of MipMap levels in the texture.
        // We only need 1 mipmap level in our depth buffer.
        // TODO: Read up on MipMap levels...
        depth_buffer_texture_description.MipLevels = 1;

        // The number of textures in the texture arrayf
        // We only need one texture for our depth buffer.
        depth_buffer_texture_description.ArraySize = 1;

        // The format of the texture.
        // DXGI_FORMAT_D24_UNORM_S8_UINT = 32-bit-z-buffer format supporting 24 bits for depth and 8 bits for stencil.
        depth_buffer_texture_description.Format = DXGI_FORMAT_D24_UNORM_S8_UINT;

        // Again, we simply use no MSAA right now, as I'm not checking for the supported quality level of my hardware.
        depth_buffer_texture_description.SampleDesc.Count = 1;
        depth_buffer_texture_description.SampleDesc.Quality = 0;

        // Usage describes how the texture should be read from and written to.
        // D3D11_USAGE_DEFAULT is the most common choice, as it describes a texture which requires Read and Write access by the GPU.
        depth_buffer_texture_description.Usage = D3D11_USAGE_DEFAULT;

        // BindFlags is used to identify how a resource should be bound to the pipeline.
        // D3D11_BIND_DEPTH_STENCIL = The texture will be bound as a depth-stencil taret for the output-merger stage.
        depth_buffer_texture_description.BindFlags = D3D11_BIND_DEPTH_STENCIL;

        let mut depth_buffer : *mut ID3D11Texture2D = null_mut();
        let mut depth_buffer_view : *mut ID3D11DepthStencilView = null_mut();

        if FAILED(device_ref.CreateTexture2D(&depth_buffer_texture_description, null_mut(), &mut depth_buffer)) {
            println!("Failed to create depth buffer!");
            return
        }

        let mut bla = D3D11_DEPTH_STENCIL_VIEW_DESC::default();
        bla.Format = depth_buffer_texture_description.Format;
        bla.ViewDimension = D3D11_DSV_DIMENSION_TEXTURE2D;

        if FAILED(device_ref.CreateDepthStencilView(depth_buffer as *mut ID3D11Resource, &bla, &mut depth_buffer_view)) {
            println!("Failed to create depth view!");
            return
        }

        let mut tihi = D3D11_DEPTH_STENCIL_DESC::default();

        /*
        tihi.DepthEnable = TRUE;
        tihi.DepthWriteMask = D3D11_DEPTH_WRITE_MASK_ALL;
        tihi.DepthFunc = D3D11_COMPARISON_LESS;

        let mut wow : *mut ID3D11DepthStencilState = null_mut();
        if FAILED( device_ref.CreateDepthStencilState(&tihi, &mut wow)) {
            panic!("Failed to create depth stencil state!");
        }

        immediate_device_context.as_ref().unwrap().OMSetDepthStencilState(wow, 1);
        */

        // Bind back buffer view and depth buffer view to Output Merger Stage
        immediate_device_context.as_ref().unwrap().OMSetRenderTargets(1, &back_buffer_view, depth_buffer_view);

        // TODO: Exercise - Enumerate through the available outputs (monitors) for an adapter. Use IDXGIAdapter::EnumOutputs.
        // TODO: Exercise - Each output has a lit of supported display modes. For each of them, list width, height, refresh rate, pixel format, etc...

        // Load texture
        let current_executable_path = env::current_exe().unwrap();

        let path_to_texture = current_executable_path.parent().unwrap().join("resources\\media\\3d_models\\crate\\crate_texture.png");

        let crate_texture = Reader::open(path_to_texture).expect("Failed to open texture file!").decode().expect("Failed to decode image.");

        let crate_texture_buffer = crate_texture.as_rgba8().expect("Failed to cast texture to RGBA8 format.");

        let the_dimensions = crate_texture_buffer.dimensions();

        let texture_description = D3D11_TEXTURE2D_DESC {
            Width: the_dimensions.0,
            Height: the_dimensions.1,
            MipLevels: 1,
            ArraySize: 1,
            // Gotta read up on this format stuff with UNORM!
            // https://docs.microsoft.com/en-us/windows/win32/direct3d10/d3d10-graphics-programming-guide-resources-data-conversion
            Format: DXGI_FORMAT_R8G8B8A8_UNORM,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0
            },
            Usage: D3D11_USAGE_DYNAMIC,
            CPUAccessFlags: D3D11_CPU_ACCESS_WRITE,
            BindFlags: D3D11_BIND_SHADER_RESOURCE,
            MiscFlags: 0
        };

        let initial_texture_data = D3D11_SUBRESOURCE_DATA {
            pSysMem: crate_texture_buffer.as_raw().as_ptr() as *const c_void,
            SysMemPitch: 1024 * (mem::size_of::<u8>() as u32 * 4 as u32),
            SysMemSlicePitch: 0
        };

        let mut texture_buffer: *mut ID3D11Texture2D = null_mut();
        let result = device_ref.CreateTexture2D(&texture_description, &initial_texture_data, &mut texture_buffer);
        if FAILED( result ) {
            panic!("Failed to create texture!");
        }

        let mut shader_texture_view = D3D11_SHADER_RESOURCE_VIEW_DESC {
            Format: DXGI_FORMAT_R8G8B8A8_UNORM,
            ViewDimension: D3D11_SRV_DIMENSION_TEXTURE2D,
            u: D3D11_SHADER_RESOURCE_VIEW_DESC_u::default()
        };

        shader_texture_view.u.Texture2D_mut().MipLevels = 1;

        let mut shader_view_resource : *mut ID3D11ShaderResourceView = null_mut();
        if FAILED( device_ref.CreateShaderResourceView(texture_buffer as *mut ID3D11Resource, &shader_texture_view, &mut shader_view_resource) ) {
            panic!("failed to create shader resource view!");
        }

        immediate_device_context.as_ref().unwrap().PSSetShaderResources(0, 1, &shader_view_resource);

        // Create Vertex Buffer and upload it
        // let loaded_model = obj_loader::load_obj();
        let path_to_cone_model = current_executable_path.parent().unwrap().join("resources\\media\\3d_models\\crate\\crate.obj");

        let loaded_model_data = obj_loader::load_obj(path_to_cone_model);

        // https://docs.microsoft.com/en-us/windows/win32/api/d3d11/ns-d3d11-d3d11_buffer_desc 
        // D3D11_BUFFER_DESC is used to describe the buffer we want to upload
        let vertex_buffer_description = D3D11_BUFFER_DESC {
            ByteWidth: (mem::size_of::<Vertex>() * loaded_model_data.vertices.len()) as UINT, // Size of the buffer in bytes
            Usage: D3D11_USAGE_DEFAULT,
            BindFlags: D3D11_BIND_VERTEX_BUFFER,
            CPUAccessFlags: 0,
            MiscFlags: 0,
            StructureByteStride: 0
        };

        // https://docs.microsoft.com/en-us/windows/win32/api/d3d11/ns-d3d11-d3d11_subresource_data
        // D3D11_SUBRESOURCE_DATA is used to describe the data we want to initialize a buffer with
        let buffer_data_description = D3D11_SUBRESOURCE_DATA {
            pSysMem: loaded_model_data.vertices.as_ptr() as *const c_void,
            SysMemPitch: 0,
            SysMemSlicePitch: 0
        };

        let mut vertex_buffer : *mut ID3D11Buffer = null_mut();
        if FAILED( device_ref.CreateBuffer(&vertex_buffer_description, &buffer_data_description, &mut vertex_buffer) ) {
            println!("Failed to create vertex buffer!");
            return
        }

        // After we have a vertex buffer, it needs to be found to an INPUT SLOT, to feed the vertices to the pipeline as input.
        let size_of_vertex_struct = mem::size_of::<Vertex>() as u32;
        let p_offsets = 0;

        immediate_device_context.as_ref().unwrap().IASetVertexBuffers(
            0, // Start Slot
            1, // Number of buffers
            &vertex_buffer,
            &size_of_vertex_struct,
            &p_offsets);

        // TODO: Read up on this whole layout object thing again...
        let semantic_name_position = CString::new("POSITION").unwrap();
        let semantic_name_uv = CString::new("UV").unwrap();
        let semantic_name_color = CString::new("COLOR").unwrap();

        let input_element_descriptions = [
            D3D11_INPUT_ELEMENT_DESC {
                SemanticName: semantic_name_position.as_ptr(),
                SemanticIndex: 0,
                Format: DXGI_FORMAT_R32G32B32_FLOAT,
                InputSlot: 0,
                AlignedByteOffset: D3D11_APPEND_ALIGNED_ELEMENT,
                InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                InstanceDataStepRate: 0
            },
            D3D11_INPUT_ELEMENT_DESC {
                SemanticName: semantic_name_uv.as_ptr(),
                SemanticIndex: 0,
                Format: DXGI_FORMAT_R32G32_FLOAT,
                InputSlot: 0,
                AlignedByteOffset: D3D11_APPEND_ALIGNED_ELEMENT, 
                InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                InstanceDataStepRate: 0
            },
            D3D11_INPUT_ELEMENT_DESC {
                SemanticName: semantic_name_color.as_ptr(),
                SemanticIndex: 0,
                Format: DXGI_FORMAT_R32G32B32A32_FLOAT,
                InputSlot: 0,
                AlignedByteOffset: D3D11_APPEND_ALIGNED_ELEMENT,
                InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                InstanceDataStepRate: 0
            }
        ];

        // parent() method will return the path without the final component, if there is one (such as a filename).
        let current_executable_path = env::current_exe().unwrap();
        let path_to_vertex_shader = current_executable_path.parent().unwrap().join("resources\\shaders\\compiled-vertex-shader.shader");

        let compiled_vertex_shader_code = fs::read(path_to_vertex_shader).unwrap();

        let mut input_layout_object : *mut ID3D11InputLayout = null_mut();
        if FAILED(device_ref.CreateInputLayout(
                input_element_descriptions.as_ptr(),
                3,
                compiled_vertex_shader_code.as_ptr() as *const c_void,
                compiled_vertex_shader_code.len(), 
                &mut input_layout_object)) {
            println!("Failed to create input layout!");
            return
        }

        immediate_device_context.as_ref().unwrap().IASetInputLayout(input_layout_object);

        // We must also tell the IA stage how to assemble the vertices into primitives.
        // You do this by specifying a "primitive type" through the Primitive Topology Method.
        immediate_device_context.as_ref().unwrap().IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST);

        // Create an Index Buffer
        // https://docs.microsoft.com/en-us/windows/win32/direct3d11/overviews-direct3d-11-resources-buffers-index-how-to
        // An Index buffer is simply buffer which contain integer offsets into a vertex buffer. It's used to render primitives more efficiently.
        // Each offset in the Index Buffer is used to identify a vertex in the vertex buffer.
        let index_buffer_data = loaded_model_data.indices;

        let index_buffer_description = D3D11_BUFFER_DESC {
            ByteWidth: (mem::size_of::<i32>() * index_buffer_data.len()) as UINT,
            Usage: D3D11_USAGE_DEFAULT,
            BindFlags: D3D11_BIND_INDEX_BUFFER,
            CPUAccessFlags: 0,
            MiscFlags: 0,
            StructureByteStride: 0
        };

        let index_buffer_data_description = D3D11_SUBRESOURCE_DATA {
            pSysMem: index_buffer_data.as_ptr() as *const c_void,
            SysMemPitch: 0,
            SysMemSlicePitch: 0
        };

        let mut index_buffer : *mut ID3D11Buffer = null_mut();
        if FAILED(device_ref.CreateBuffer(&index_buffer_description, &index_buffer_data_description, &mut index_buffer)) {
            println!("Failed to create index buffer!");
            return
        }

        immediate_device_context.as_ref().unwrap().IASetIndexBuffer(index_buffer, DXGI_FORMAT_R32_UINT, 0);

        // Create vertex shader and pixel shader
        let path_to_pixel_shader = current_executable_path.parent().unwrap().join("resources\\shaders\\compiled-pixel-shader.shader");
        let compiled_pixel_shader_code = fs::read(path_to_pixel_shader).unwrap();

        let mut vertex_shader_instance : *mut ID3D11VertexShader = null_mut();
        if FAILED(device_ref.CreateVertexShader(compiled_vertex_shader_code.as_ptr() as *const c_void, compiled_vertex_shader_code.len(), null_mut(), &mut vertex_shader_instance)) {
            println!("Failed to create vertex shader!");
            return
        }

        let mut pixel_shader_instance : *mut ID3D11PixelShader = null_mut();
        if FAILED(device_ref.CreatePixelShader(compiled_pixel_shader_code.as_ptr() as *const c_void, compiled_pixel_shader_code.len(), null_mut(), &mut pixel_shader_instance)) {
            println!("Failed to create pixel shader!");
            return
        }

        // A vertex shader must always be active for the pipeline to execute
        immediate_device_context.as_ref().unwrap().VSSetShader(vertex_shader_instance, null_mut(), 0);
        immediate_device_context.as_ref().unwrap().PSSetShader(pixel_shader_instance, null_mut(), 0);

        // Create Rasterizer state
        // TODO: Definitely read more up on this...
        // https://docs.microsoft.com/en-us/windows/win32/api/d3d11/ns-d3d11-d3d11_rasterizer_desc
        let mut rasterizer_descriptiona = D3D11_RASTERIZER_DESC::default();
        //rasterizer_descriptiona.FillMode = D3D11_FILL_WIREFRAME;
        rasterizer_descriptiona.FillMode = D3D11_FILL_SOLID;
        rasterizer_descriptiona.CullMode = D3D11_CULL_BACK;
        rasterizer_descriptiona.FrontCounterClockwise = FALSE;
        rasterizer_descriptiona.ScissorEnable = FALSE;

        // TODO: Setting this to TRUE makes everything invicible... why?
        rasterizer_descriptiona.DepthClipEnable = TRUE;
        rasterizer_descriptiona.MultisampleEnable = FALSE;

        let mut rasterizer_state : *mut ID3D11RasterizerState = null_mut();
        if FAILED(device_ref.CreateRasterizerState(&rasterizer_descriptiona, &mut rasterizer_state)) {
            println!("Failed to create rasterizer state!");
            return
    }

    immediate_device_context.as_ref().unwrap().RSSetState(rasterizer_state);

    // It appears that it is required for a viewport to be bound to the pipeline before the Draw() call succeeds.
    let viewport = D3D11_VIEWPORT {
        Height: 600.0,
        Width: 800.0,
        MinDepth: 0.0,
        MaxDepth: 1.0,
        TopLeftX: 0.0,
        TopLeftY: 0.0
    };

    immediate_device_context.as_ref().unwrap().RSSetViewports(1, &viewport);

    // Set shader buffers
    // From what I can gather, it ultimatley matters that the vertices are delivered to the Rasterizer stage in a 
    // left handed, y up, coordinate space, as the DirectX documentation states that vertices coming into the rasterizer stage
    // are assumed X axis pointing right, Y pointing up, and Z pointing away from the camera, that is, Z is positive when going "into" the screen.
    // https://docs.microsoft.com/en-us/windows/win32/direct3d11/d3d10-graphics-programming-guide-rasterizer-stage
    let fov_in_degrees: f32 = 45.0;
    let projection_matrix = perspective_fov_lh_zo(fov_in_degrees.to_radians(), 800.0, 600.0, 1.0, 100.0);

    let mut eye_position = Vec3::new(0.0, 5.0, 15.0);

    // Create view translation matrix (inverse of eye position)
    eye_position *= 1.0;
    let mut camera_postion = glm::Mat4::identity();
    camera_postion = glm::translate(&camera_postion, &eye_position);

     let mut world_view_matrix = VertexConstantBuffer {
        worldViewProjection: projection_matrix * camera_postion
     };

        // Ultraviolet stores matrices in row-major order.
        // This means that each element of a row is stored consecutively next to each other.
        // However, HLSL uses column-major by default. Meaning, each element of a column will be stored consecutively next to each other.
        // Therefore, I need to tranpose the matrix before passing it to the vertex shader.
        // https://en.wikipedia.org/wiki/Row-_and_column-major_order
        // world_view_matrix.worldViewProjection.transpose();

        let vertex_constant_buffer_description = D3D11_BUFFER_DESC {
            ByteWidth: mem::size_of::<VertexConstantBuffer>() as UINT,
            // A constant buffer should be DYNAMIC, as it should be accessible by the GPU (read-only) and the CPU (write-only)
            // Resources with D3D11_USAGE_DYNAMIC cannot be used as destination resources for the UpdateSubresource method.
            // So, if you want to change the content of a D3D11_USAGE_DYNAMIC buffer, use the Map method instead.
            // D3D11_USAGE_DYNAMIC is a good choice for a buffer that is updated once per CPU cycle, as is our constant buffer with the world matrix.
            Usage: D3D11_USAGE_DYNAMIC,
            // We indicate that the buffer should be a constant buffer
            BindFlags: D3D11_BIND_CONSTANT_BUFFER,
            // We need the CPU to have WRITE ACCESS, so that the CPU can change its contents
            CPUAccessFlags: D3D11_CPU_ACCESS_WRITE,
            MiscFlags: 0,
            StructureByteStride: 0
        };

        let vertex_constant_buffer_init_data = D3D11_SUBRESOURCE_DATA {
            pSysMem: &world_view_matrix as *const _ as *const c_void,
            SysMemPitch: 0,
            SysMemSlicePitch: 0
        };

        let mut vertex_constant_buffer : *mut ID3D11Buffer = null_mut();
        if FAILED(device_ref.CreateBuffer(&vertex_constant_buffer_description, &vertex_constant_buffer_init_data, &mut vertex_constant_buffer)) {
            println!("Failed to create vertex constant buffer!");
            return
        }

        immediate_device_context.as_ref().unwrap().VSSetConstantBuffers(0, 1, &vertex_constant_buffer);

        let mut should_quit = false;
        let mut current_message = MSG::default();

        let mut cam_x = 0.0;
        let mut cam_y = 10.0;
        let mut cam_z = -15.0;

        let mut cam_rot_x = -30f32.to_radians();
        let mut cam_rot_y = 0.0f32.to_radians();
        let mut cam_rot_z = 0f32.to_radians();

        while !should_quit {
            // PeekMessage will retrieve messages associated with the main window.
            // By specifying PM_REMOVE, we remove the message from the queue for processing.
            if PeekMessageW(&mut current_message, main_window, 0, 0, PM_REMOVE) != 0 {
                if current_message.message == WM_QUIT {
                    should_quit = true;
                }

                TranslateMessage(&current_message);
                DispatchMessageW(&current_message);
            } else {
                // UPDATE
                if window_helper.is_key_pressed(KeyType::W) {
                    cam_z += 0.2;
                }

                if window_helper.is_key_pressed(KeyType::S) {
                    cam_z -= 0.2;
                }

                if window_helper.is_key_pressed(KeyType::A) {
                    cam_x -= 0.2;
                }

                if window_helper.is_key_pressed(KeyType::D) {
                    cam_x += 0.2;
                }

                if window_helper.is_key_pressed(KeyType::Q) {
                    cam_y -= 0.2;
                }

                if window_helper.is_key_pressed(KeyType::E) {
                    cam_y += 0.2;
                }

                if window_helper.is_key_pressed(KeyType::Up) {
                    cam_rot_x += 0.02;
                }

                if window_helper.is_key_pressed(KeyType::Down) {
                    cam_rot_x -= 0.02;
                }

                if window_helper.is_key_pressed(KeyType::Left) {
                    cam_rot_y += 0.02;
                }

                if window_helper.is_key_pressed(KeyType::Right) {
                    cam_rot_y -= 0.02;
                }

                if window_helper.was_mouse_pressed(MouseKey::Left) {
                    println!("Mouse was pressed!");
                }

                if window_helper.is_mouse_down(MouseKey::Left) {
                    println!("Mouse is down!!");
                }

                // Update vertex constant buffer for world matrix
                let mut mapped_resource : D3D11_MAPPED_SUBRESOURCE = D3D11_MAPPED_SUBRESOURCE::default();

                // Buffers which are dynamic (D3D11_USAGE_DYNAMIC) cannot have their content updated by the UpdateSubresource method.
                // Instead, we have to use the "Map" method.
                // This method retrives a pointer to the data contained in a subresource (such as our constant buffer), and we can then use that pointer
                // To update its data.
                // When you call the Map method, the GPU will have its access to that subresource denied.
                if FAILED(immediate_device_context.as_ref().unwrap().Map(vertex_constant_buffer as *mut ID3D11Resource, 0, D3D11_MAP_WRITE_DISCARD, 0, &mut mapped_resource)) {
                    panic!("Failed to retrieve vertex constant buffer subresource!");
                };

                let lol : *mut VertexConstantBuffer = mapped_resource.pData as *mut VertexConstantBuffer;
                let mut eye_position = Vec3::new(cam_x, cam_y, cam_z);


                eye_position *= -1.0;
                let camera_postion = translation(&eye_position);

                let camera_rotation = rotate_x(&Mat4::identity(), cam_rot_x) * rotate_y(&Mat4::identity(), cam_rot_y) * rotate_z(&Mat4::identity(), cam_rot_z);

                (*lol).worldViewProjection = perspective_fov_lh_zo(fov_in_degrees.to_radians(), 800.0, 600.0, 0.1, 100.0) *  (camera_rotation * camera_postion); 
                // (*lol).worldViewProjection.transpose();

                // After we're done mapping new data, we have to call Unmap in order to invalidate the pointer to the buffer
                // and reenable the GPU's access to that resource
                immediate_device_context.as_ref().unwrap().Unmap(vertex_constant_buffer as *mut ID3D11Resource, 0);

                window_helper.update();

                // RENDER

                // Triangle will NOT render unless both ClearRenderTargetView and ClearDpethStencilView is called!

                immediate_device_context.as_ref().unwrap().VSSetShader(vertex_shader_instance, null_mut(), 0);
                immediate_device_context.as_ref().unwrap().PSSetShader(pixel_shader_instance, null_mut(), 0);

                let clear_color = Vec4::new(0.45, 0.6, 0.95, 1.0);
                immediate_device_context.as_ref().unwrap().ClearRenderTargetView(back_buffer_view, value_ptr(&clear_color).try_into().expect("Failed to convert clear color arr!"));
                immediate_device_context.as_ref().unwrap().ClearDepthStencilView(depth_buffer_view, D3D11_CLEAR_DEPTH | D3D11_CLEAR_STENCIL, 1.0, 0);

                immediate_device_context.as_ref().unwrap().DrawIndexed(index_buffer_data.len() as UINT, 0, 0);

                if FAILED(idxgi_swap_chain.as_ref().unwrap().Present(1, 0)) {
                    println!("Failed to present!");
                }
            }
        }
    }
}

fn create_swap_chain_description(main_window: *mut HWND__) -> DXGI_SWAP_CHAIN_DESC {
    let mut swap_chain_description = DXGI_SWAP_CHAIN_DESC::default();
    
    swap_chain_description.BufferDesc.Width = 800;
    swap_chain_description.BufferDesc.Height = 600;

    // DXGI_MODE_DESC refresh rate is a struct describing the refresh rate in hertz.
    swap_chain_description.BufferDesc.RefreshRate.Numerator = 60;
    swap_chain_description.BufferDesc.RefreshRate.Denominator = 1;

    // DXGI_MODE_DESC format of the buffer
    // DXGI_FORMAT_R8G8B8A8_UNORM = four component, 32-bit unsigned-normalized-integer which
    // supports 8 bits per channel, including alpha
    swap_chain_description.BufferDesc.Format = DXGI_FORMAT_R8G8B8A8_UNORM;

    // DXGI_MODE_DESC scanline ordering is used to specify the method the raster uses to draw an
    // image.
    swap_chain_description.BufferDesc.ScanlineOrdering = DXGI_MODE_SCANLINE_ORDER_UNSPECIFIED;

    // DXGI_MODE_DESC scaling is used to indicate how the image of the buffer is stretched to fit a
    // monitors resolution.
    // Using UNSPECIFIED means that you know the native resolution of the display and want to make
    // sure you dont trigger a mode change when transitioning a swap chain to full screen.
    swap_chain_description.BufferDesc.Scaling = DXGI_MODE_SCALING_UNSPECIFIED;

    // DXGI_SAMPLE_DESC is used to describe multi sampling properties.
    // DXGI_SAMPLE_DESC Count is used to describe the number of multisamples per pixel.
    swap_chain_description.SampleDesc.Count = 1;

    // DXGI_SAMPLE_DESC Quality is used to describe the quality level.
    // Higher quality level equals lower performance.
    // For now I simply pick the lowest possible quality (0)
    swap_chain_description.SampleDesc.Quality = 0;

    // BufferUsage is used to indicate the surface usage and CPU access options for the backbuffer.
    // DXGI_USAGE_RENDER_TARGET_OUTPUT means that we want the back buffer to be used for rendering
    // output of the graphics pipeline.
    swap_chain_description.BufferUsage = DXGI_USAGE_RENDER_TARGET_OUTPUT;

    // TODO: Why do we indicate 1 and not 2 for buffer count?
    swap_chain_description.BufferCount = 1;

    // OutputWindow is a handle to the output window.
    // This value CANNOT be null.
    swap_chain_description.OutputWindow = main_window;

    // Set the output to windowed mode. This is a fairly important value.
    // If the swap-chain is in windowed mode, the front-buffer is the desktop.
    // If the swap-chain is not in windowed mode, there is a dedicated front buffer.
    // Creating a full-screen swap-chain with an unsupported display mode will cause the display to go black, preventing the end user from seeing anything.
    // I actually experienced this on my machine when I skipped this property.
    swap_chain_description.Windowed = TRUE;

    // The SwapEffect is used to indicate what to do with the pixels in a display buffer after the PRESENT action has been performed on the swap chain.
    // DXGI_SWAP_EFFECT_DISCARD simply means that the display driver will select the most efficient presentation technique for the swap chain.
    // IT also means that the contents of the back buffer is discarded after you call PRESENT.
    swap_chain_description.SwapEffect = DXGI_SWAP_EFFECT_DISCARD;

    swap_chain_description
}

// TODO: What does "extern 'system'" mean?
// About Keyboard Input Model: https://docs.microsoft.com/en-us/windows/win32/inputdev/about-keyboard-input
unsafe extern "system" fn window_proc(hwnd: HWND, u_msg: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    let window_helper = GetWindowLongPtrA(hwnd, GWLP_USERDATA) as *mut Window;

    // TODO: Read more up on the match statement... cuz this thing powerful
    match u_msg {
        WM_QUIT | WM_DESTROY | WM_CLOSE  => {
            PostQuitMessage(0);
            0
        },
        // Keydown Documentation: https://docs.microsoft.com/en-us/windows/win32/inputdev/wm-keydown
        WM_KEYDOWN | WM_KEYUP => {
            let key_press : Option<KeyType> = num::FromPrimitive::from_usize(w_param);

            let is_key_pressed = match u_msg {
                WM_KEYDOWN => true,
                _ => false
            };

            if key_press.is_some() {
                window_helper.as_mut().unwrap().proc(key_press.unwrap(), is_key_pressed);
            }

            0 // An application should return 0 if it successfully processed the message
        },
        // Documentation for MOUSE events: https://docs.microsoft.com/en-us/windows/win32/inputdev/about-mouse-input
        WM_MOUSEMOVE => {
            let x_pos : i32 = GET_X_LPARAM(l_param);
            let y_pos : i32 = GET_Y_LPARAM(l_param);

            window_helper.as_mut().unwrap().update_mouse_position(x_pos, y_pos);

            0
        },
        WM_LBUTTONDOWN => {
            window_helper.as_mut().unwrap().update_mouse_key_state(MouseKey::Left, KeyState::Down);
            0
        },
        WM_LBUTTONUP => {
            window_helper.as_mut().unwrap().update_mouse_key_state(MouseKey::Left, KeyState::Up);
            0
        },
        // TODO: Read up on general Windows message processing theory
        _ => DefWindowProcW(hwnd, u_msg, w_param, l_param)
    }
}
