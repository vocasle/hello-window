use windows::{
    s,
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, S_OK, WPARAM},
        Graphics::{
            Direct3D::D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST,
            Direct3D11::{D3D11_CLEAR_DEPTH, D3D11_CLEAR_STENCIL},
        },
        System::LibraryLoader::GetModuleHandleA,
        UI::WindowsAndMessaging::{
            CreateWindowExA, DefWindowProcA, DispatchMessageA, GetMessageA, GetWindowLongPtrA,
            PostQuitMessage, RegisterClassA, SetWindowLongPtrA, ShowWindow, TranslateMessage,
            CW_USEDEFAULT, GWLP_USERDATA, MSG, SW_SHOWDEFAULT, WINDOW_EX_STYLE, WM_DESTROY,
            WM_PAINT, WM_QUIT, WNDCLASSA, WS_OVERLAPPEDWINDOW,
        },
    },
};

use windows::core::Result as WinResult;

use crate::device_resources::device_resources::{DeviceResources, DEFAULT_HEIGHT, DEFAULT_WIDTH};

mod device_resources;

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    u_msg: u32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    return match u_msg {
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        WM_PAINT => {
            unsafe {
                let ptr = GetWindowLongPtrA(hwnd, GWLP_USERDATA);
                if let Some(dr) = std::ptr::NonNull::<DeviceResources>::new(ptr as _) {
                    let dr = dr.as_ref();

                    dr.context.ClearDepthStencilView(
                        &dr.dsv,
                        (D3D11_CLEAR_DEPTH | D3D11_CLEAR_STENCIL).0,
                        1f32,
                        0,
                    );

                    let clear_color = vec![1f32, 0f32, 0f32, 1f32];
                    dr.context
                        .ClearRenderTargetView(dr.rtv.get(0), clear_color.as_ptr());

                    dr.context.VSSetShader(&dr.vs, None);
                    dr.context.PSSetShader(&dr.ps, None);
                    dr.context.IASetInputLayout(&dr.il);
                    dr.context
                        .IASetPrimitiveTopology(D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
                    dr.context.OMSetRenderTargets(Some(&dr.rtv), &dr.dsv);
                    dr.context.RSSetViewports(Some(&[dr.viewport]));

                    if S_OK != dr.swapchain.Present(1, 0) {
                        panic!("Failed to present!");
                    }
                }
            }
            LRESULT(0)
        }
        _ => DefWindowProcA(hwnd, u_msg, w_param, l_param),
    };
}

fn main() -> WinResult<()> {
    let class_name = s!("Sample Class Name");
    let mut wc = WNDCLASSA {
        ..Default::default()
    };

    let instance = unsafe { GetModuleHandleA(None)? };

    wc.lpfnWndProc = Some(window_proc);
    wc.hInstance = instance;
    wc.lpszClassName = class_name;

    unsafe {
        RegisterClassA(&wc);
    }

    let hwnd = unsafe {
        CreateWindowExA(
            WINDOW_EX_STYLE::default(),
            class_name,
            s!("Lear to Program Windows"),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            DEFAULT_WIDTH as i32,
            DEFAULT_HEIGHT as i32,
            None,
            None,
            instance,
            None,
        )
    };

    unsafe {
        ShowWindow(hwnd, SW_SHOWDEFAULT);
    }

    let device_resources = DeviceResources::bind_to_wnd(hwnd)?;

    unsafe {
        SetWindowLongPtrA(hwnd, GWLP_USERDATA, &device_resources as *const _ as _);
    }

    let mut msg = MSG::default();

    loop {
        unsafe {
            GetMessageA(&mut msg, None, 0, 0);
            TranslateMessage(&msg);
            DispatchMessageA(&msg);

            if msg.message == WM_QUIT {
                break;
            }
        }
    }

    println!("All is OK!");

    return Ok(());
}
