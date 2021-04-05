use std::{ffi::OsStr, iter::once, mem, ptr::null_mut};
use std::os::windows::prelude::*;

extern crate winapi;

use winapi::{shared::windef::HWND, um::libloaderapi::GetModuleHandleW};
use winapi::shared::minwindef::*;
use winapi::um::winuser::*;

fn main() {
    // TODO: Read up on unsafe block
    unsafe {
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
           CW_USEDEFAULT, CW_USEDEFAULT, 800, 800,
           null_mut(),
           null_mut(),
           h_instance,
           null_mut());

        if main_window.is_null() {
            println!("Failed to create window!");
            return
        }

        ShowWindow(main_window, SW_SHOW);

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
