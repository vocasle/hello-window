use windows::{
    s,
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, WPARAM},
        Graphics::Gdi::{BeginPaint, EndPaint, FillRect, HBRUSH, PAINTSTRUCT},
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
                let ptr = std::ptr::NonNull::<CustomObject>::new(ptr as _);
                if let Some(mut co) = ptr {
                    println!("Retrieved custom object. Number: {}", co.as_ref().num);
                    co.as_mut().num = co.as_ref().num + 1;
                }
            }
            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut ps);
            FillRect(hdc, &ps.rcPaint, HBRUSH::default());
            EndPaint(hwnd, &ps);
            LRESULT(0)
        }
        _ => DefWindowProcA(hwnd, u_msg, w_param, l_param),
    };
}

struct CustomObject {
    num: i32,
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

    let co = CustomObject { num: 11081994 };
    unsafe {
        SetWindowLongPtrA(hwnd, GWLP_USERDATA, &co as *const _ as _);
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
