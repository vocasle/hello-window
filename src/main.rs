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
            Dxgi::Common::{
                DXGI_FORMAT, DXGI_FORMAT_R16_UINT, DXGI_FORMAT_R32_UINT, DXGI_FORMAT_UNKNOWN,
            },
        },
        System::LibraryLoader::GetModuleHandleA,
        UI::WindowsAndMessaging::{
            CreateWindowExA, DefWindowProcA, DispatchMessageA, GetMessageA, GetWindowLongPtrA,
            PostQuitMessage, RegisterClassA, SetWindowLongPtrA, ShowWindow, TranslateMessage,
            CW_USEDEFAULT, GWLP_USERDATA, MSG, SW_SHOWDEFAULT, WINDOW_EX_STYLE, WM_DESTROY,
            WM_KEYDOWN, WM_PAINT, WM_QUIT, WNDCLASSA, WS_OVERLAPPEDWINDOW,
        },
    },
};

use windows::core::Result as WinResult;

use crate::device_resources::device_resources::{DeviceResources, DEFAULT_HEIGHT, DEFAULT_WIDTH};

mod device_resources;

#[allow(unused_macros)]
macro_rules! result {
    ($x:expr) => {
        match $x {
            Ok(t) => t,
            Err(err) => {
                panic!(
                    "DirectX command failed: {}:{}. Error: {}",
                    file!(),
                    line!(),
                    err.message()
                );
            }
        }
    };
}

struct Model {
    num_indices: u32,
    vb: Option<ID3D11Buffer>,
    ib: ID3D11Buffer,
    ib_format: DXGI_FORMAT,
}

#[allow(dead_code)]
struct Position {
    x: f32,
    y: f32,
    z: f32,
}

impl Model {
    fn default(device: &ID3D11Device) -> WinResult<Self> {
        let vertices = vec![
            Position {
                x: -0.5,
                y: -0.5,
                z: 0.0,
            },
            Position {
                x: 0.0,
                y: 0.5,
                z: 0.0,
            },
            Position {
                x: 0.5,
                y: -0.5,
                z: 0.0,
            },
        ];
        let indices: Vec<u32> = vec![0, 1, 2];

        let vb = DeviceResources::create_buffer(device, &vertices, D3D11_BIND_VERTEX_BUFFER)?;
        let ib = DeviceResources::create_buffer(device, &indices, D3D11_BIND_INDEX_BUFFER)?;

        Ok(Model {
            num_indices: indices.len() as u32,
            vb: Some(vb),
            ib: ib,
            ib_format: DXGI_FORMAT_R32_UINT,
        })
    }

    fn from_gltf(path: &str, device: &ID3D11Device) -> WinResult<Self> {
        let mut ib = None;
        let mut vb = None;
        let mut num_indices: u32 = 0;
        let mut ib_format = DXGI_FORMAT_UNKNOWN;

        let (doc, buffers, images) = gltf::import(path).unwrap();

        for scene in doc.scenes() {
            for node in scene.nodes() {
                if let Some(mesh) = node.mesh() {
                    for prim in mesh.primitives() {
                        if let Some(acc) = prim.indices() {
                            let view = acc.view().unwrap();
                            let buff = &buffers[view.buffer().index()];
                            num_indices = acc.count() as u32;

                            ib_format = match acc.data_type() {
                                gltf::accessor::DataType::U16 => DXGI_FORMAT_R16_UINT,
                                gltf::accessor::DataType::U32 => DXGI_FORMAT_R32_UINT,
                                _ => panic!("Unexpected data type for index buffer"),
                            };

                            let desc = D3D11_BUFFER_DESC {
                                ByteWidth: view.length() as u32,
                                Usage: D3D11_USAGE_IMMUTABLE,
                                BindFlags: D3D11_BIND_INDEX_BUFFER,
                                CPUAccessFlags: D3D11_CPU_ACCESS_FLAG(0),
                                MiscFlags: D3D11_RESOURCE_MISC_FLAG(0),
                                StructureByteStride: 0,
                            };

                            let init_data = D3D11_SUBRESOURCE_DATA {
                                pSysMem: (unsafe {
                                    buff.0
                                        .as_ptr()
                                        .offset((acc.offset() + view.offset()) as isize)
                                }) as *const _ as _,
                                SysMemPitch: 0,
                                SysMemSlicePitch: 0,
                            };

                            result!(unsafe {
                                device.CreateBuffer(&desc, Some(&init_data), Some(&mut ib))
                            });
                        }

                        for (sem, acc) in prim.attributes() {
                            match sem {
                                gltf::Semantic::Positions => {
                                    let view = acc.view().unwrap();
                                    let buff = &buffers[view.buffer().index()];

                                    let desc = D3D11_BUFFER_DESC {
                                        ByteWidth: view.length() as u32,
                                        Usage: D3D11_USAGE_IMMUTABLE,
                                        BindFlags: D3D11_BIND_VERTEX_BUFFER,
                                        CPUAccessFlags: D3D11_CPU_ACCESS_FLAG(0),
                                        MiscFlags: D3D11_RESOURCE_MISC_FLAG(0),
                                        StructureByteStride: 0,
                                    };

                                    let init_data = D3D11_SUBRESOURCE_DATA {
                                        pSysMem: (unsafe {
                                            buff.0
                                                .as_ptr()
                                                .offset((acc.offset() + view.offset()) as isize)
                                        })
                                            as *const _
                                            as _,
                                        SysMemPitch: 0,
                                        SysMemSlicePitch: 0,
                                    };

                                    result!(unsafe {
                                        device.CreateBuffer(&desc, Some(&init_data), Some(&mut vb))
                                    });
                                }
                                gltf::Semantic::Normals => todo!(),
                                gltf::Semantic::Tangents => todo!(),
                                gltf::Semantic::Colors(_) => todo!(),
                                gltf::Semantic::TexCoords(_) => todo!(),
                                gltf::Semantic::Joints(_) => todo!(),
                                gltf::Semantic::Weights(_) => todo!(),
                            }
                        }
                    }
                }
            }
            break;
        }

        Ok(Model {
            num_indices,
            vb: vb,
            ib: ib.unwrap(),
            ib_format,
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

                    let clear_color = vec![1f32, 0f32, 1f32, 1f32];
                    dr.context
                        .ClearRenderTargetView(dr.rtv.get(0), clear_color.as_ptr());

                    dr.context.VSSetShader(&dr.vs, None);
                    dr.context.PSSetShader(&dr.ps, None);
                    dr.context.IASetInputLayout(&dr.il);
                    dr.context
                        .IASetPrimitiveTopology(D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
                    dr.context.OMSetRenderTargets(Some(&dr.rtv), &dr.dsv);
                    dr.context.RSSetViewports(Some(&[dr.viewport]));
                    let strides = std::mem::size_of::<Position>() as u32;
                    let offsets = 0u32;
                    dr.context.IASetVertexBuffers(
                        0,
                        1,
                        Some(&app.model.vb),
                        Some(&strides),
                        Some(&offsets),
                    );
                    dr.context
                        .IASetIndexBuffer(&app.model.ib, app.model.ib_format, 0);
                    dr.context.DrawIndexed(app.model.num_indices, 0, 0);

                    if S_OK != dr.swapchain.Present(1, 0) {
                        panic!("Failed to present!");
                    }
                }
            }
            LRESULT(0)
        }
        WM_KEYDOWN => {
            if w_param == WPARAM(0x1B) {
                unsafe { PostQuitMessage(0) }
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
    let model = Model::from_gltf(
        "C:\\Source\\glTF-Sample-Models\\2.0\\Triangle\\glTF\\Triangle.gltf",
        &device_resources.device,
    )?;
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
