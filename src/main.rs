use std::{ffi::OsStr, iter::once, mem, ptr::null_mut};
use std::os::windows::prelude::*;

extern crate winapi;

use ultraviolet::{Vec3, Vec4};
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

// DirectX related imports
use winapi::um::d3d11::*;
use winapi::um::d3dcommon::*;

struct Vertex {
    position: Vec3,
    color: Vec4
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

        // The number of textures in the texture array.
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

        if FAILED(device_ref.CreateDepthStencilView(depth_buffer as *mut ID3D11Resource, null_mut(), &mut depth_buffer_view)) {
            println!("Failed to create depth view!");
            return
        }

        // Bind back buffer view and depth buffer view to Output Merger Stage
        immediate_device_context.as_ref().unwrap().OMSetRenderTargets(1, &back_buffer_view, depth_buffer_view);

        // TODO: Exercise - Enumerate through the available outputs (monitors) for an adapter. Use IDXGIAdapter::EnumOutputs.
        // TODO: Exercise - Each output has a lit of supported display modes. For each of them, list width, height, refresh rate, pixel format, etc...

        // Create Vertex Buffer and upload it
        let triangle_vertex_buffer = [
            Vertex {
                position: Vec3::new(0.0, 10.0, 0.0),
                color: Vec4::new(0.5, 0.5, 0.5, 1.0)
            },
            Vertex {
                position: Vec3::new(10.0, -10.0, 0.0),
                color: Vec4::new(0.5, 0.5, 0.5, 1.0)
            },
            Vertex {
                position: Vec3::new(-10.0, -10.0, 0.0),
                color: Vec4::new(0.5, 0.5, 0.5, 1.0)
            }
        ];

        // https://docs.microsoft.com/en-us/windows/win32/api/d3d11/ns-d3d11-d3d11_buffer_desc 
        // D3D11_BUFFER_DESC is used to describe the buffer we want to upload
        let vertex_buffer_description = D3D11_BUFFER_DESC {
            ByteWidth: (mem::size_of::<Vertex>() * 3) as UINT, // Size of the buffer in bytes
            Usage: D3D11_USAGE_DEFAULT,
            BindFlags: D3D11_BIND_VERTEX_BUFFER,
            CPUAccessFlags: 0,
            MiscFlags: 0,
            StructureByteStride: 0
        };

        let mut should_quit = false;
        let mut current_message = MSG::default();

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
                // RENDER
            }
        }
    }
}

fn create_swap_chain_description(main_window: *mut HWND__) -> DXGI_SWAP_CHAIN_DESC {
    let mut swap_chain_description = DXGI_SWAP_CHAIN_DESC::default();

    // DXGI_MODE_DESC width and height is the resolution of the output window
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
unsafe extern "system" fn window_proc(hwnd: HWND, u_msg: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    // TODO: Read more up on the match statement... cuz this thing powerful
    match u_msg {
        WM_QUIT | WM_DESTROY | WM_CLOSE  => {
            PostQuitMessage(0);
            0
        },
        // TODO: Read up on general Windows message processing theory
        _ => DefWindowProcW(hwnd, u_msg, w_param, l_param)
    }
}
