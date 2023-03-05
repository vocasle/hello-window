use std::error::Error;

use windows::{
    core::PCSTR,
    s,
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, WPARAM},
        Graphics::Gdi::{
            BeginPaint, EndPaint, FillRect, COLOR_WINDOW, COLOR_WINDOWFRAME, HBRUSH, PAINTSTRUCT,
        },
        System::LibraryLoader::GetModuleHandleA,
        UI::WindowsAndMessaging::{
            CreateWindowExA, DefWindowProcA, DispatchMessageA, GetMessageA, PostQuitMessage,
            RegisterClassA, ShowWindow, TranslateMessage, CW_USEDEFAULT, MSG, SW_SHOWDEFAULT,
            WINDOW_EX_STYLE, WM_DESTROY, WM_PAINT, WM_QUIT, WNDCLASSA, WNDCLASS_STYLES,
            WS_OVERLAPPEDWINDOW,
        },
    },
};

use windows::core::Result as WinResult;

unsafe extern "system" fn WindowProc(
    hwnd: HWND,
    uMsg: u32,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    return match uMsg {
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut ps);
            FillRect(hdc, &ps.rcPaint, HBRUSH::default());
            EndPaint(hwnd, &ps);
            LRESULT(0)
        }
        _ => DefWindowProcA(hwnd, uMsg, wParam, lParam),
    };
}

fn main() -> WinResult<()> {
    let class_name = s!("Sample Class Name");
    let mut wc = WNDCLASSA {
        ..Default::default()
    };

    let instance = unsafe { GetModuleHandleA(None)? };

    wc.lpfnWndProc = Some(WindowProc);
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
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            None,
            None,
            instance,
            None,
        )
    };

    unsafe {
        ShowWindow(hwnd, SW_SHOWDEFAULT);
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
