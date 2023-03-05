use windows::{
    s,
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, S_OK, WPARAM},
        Graphics::{
            Direct3D::D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST,
            Direct3D11::{
                ID3D11Buffer, ID3D11Device, D3D11_BIND_INDEX_BUFFER, D3D11_BIND_VERTEX_BUFFER,
                D3D11_BUFFER_DESC, D3D11_CLEAR_DEPTH, D3D11_CLEAR_STENCIL, D3D11_CPU_ACCESS_FLAG,
                D3D11_RESOURCE_MISC_FLAG, D3D11_SUBRESOURCE_DATA, D3D11_USAGE_IMMUTABLE,
            },
            Dxgi::Common::DXGI_FORMAT_R32_UINT,
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

struct Model {
    num_indices: u32,
    vb: Option<ID3D11Buffer>,
    ib: ID3D11Buffer,
}

impl Model {
    fn default(device: &ID3D11Device) -> WinResult<Self> {
        let vertices: Vec<f32> = vec![-0.5, -0.5, 0.0, 0.0, 0.5, 0.0, 0.5, -0.5, 0.0];
        let indices: Vec<u32> = vec![0, 1, 2];

        let mut vb = None;
        unsafe {
            let desc = D3D11_BUFFER_DESC {
                ByteWidth: (vertices.len() * std::mem::size_of::<f32>() * 3) as u32,
                Usage: D3D11_USAGE_IMMUTABLE,
                BindFlags: D3D11_BIND_VERTEX_BUFFER,
                CPUAccessFlags: D3D11_CPU_ACCESS_FLAG::default(),
                MiscFlags: D3D11_RESOURCE_MISC_FLAG::default(),
                StructureByteStride: (std::mem::size_of::<f32>() * 3) as u32,
            };

            let init_data = D3D11_SUBRESOURCE_DATA {
                pSysMem: &vertices as *const _ as _,
                SysMemPitch: 0,
                SysMemSlicePitch: 0,
            };

            device.CreateBuffer(&desc, Some(&init_data), Some(&mut vb))?;
        }

        let mut ib = None;
        unsafe {
            let desc = D3D11_BUFFER_DESC {
                ByteWidth: (indices.len() * std::mem::size_of::<u32>() * 3) as u32,
                Usage: D3D11_USAGE_IMMUTABLE,
                BindFlags: D3D11_BIND_INDEX_BUFFER,
                CPUAccessFlags: D3D11_CPU_ACCESS_FLAG::default(),
                MiscFlags: D3D11_RESOURCE_MISC_FLAG::default(),
                StructureByteStride: (std::mem::size_of::<u32>() * 3) as u32,
            };

            let init_data = D3D11_SUBRESOURCE_DATA {
                pSysMem: &indices as *const _ as _,
                SysMemPitch: 0,
                SysMemSlicePitch: 0,
            };

            device.CreateBuffer(&desc, Some(&init_data), Some(&mut ib))?;
        }

        Ok(Model {
            num_indices: indices.len() as u32,
            vb: vb,
            ib: ib.unwrap(),
        })
    }
}

struct App {
    dr: DeviceResources,
    model: Model,
}

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
                if let Some(dr) = std::ptr::NonNull::<App>::new(ptr as _) {
                    let app = dr.as_ref();
                    let dr = &app.dr;

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
                    let strides = (std::mem::size_of::<f32>() * 3) as u32;
                    let offsets = 0u32;
                    dr.context.IASetVertexBuffers(
                        0,
                        1,
                        Some(&app.model.vb),
                        Some(&strides),
                        Some(&offsets),
                    );
                    dr.context
                        .IASetIndexBuffer(&app.model.ib, DXGI_FORMAT_R32_UINT, 0);
                    dr.context.DrawIndexed(app.model.num_indices, 0, 0);

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
    let model = Model::default(&device_resources.device)?;
    let app = App {
        dr: device_resources,
        model,
    };

    unsafe {
        SetWindowLongPtrA(hwnd, GWLP_USERDATA, &app as *const _ as _);
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
