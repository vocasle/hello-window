pub mod device_resources {
    use windows::Win32::{
        Foundation::{BOOL, HWND},
        Graphics::{
            Direct3D::D3D_DRIVER_TYPE_HARDWARE,
            Direct3D11::{
                D3D11CreateDevice, ID3D11DepthStencilView, ID3D11Device, ID3D11DeviceContext,
                ID3D11RenderTargetView, ID3D11Texture2D, D3D11_BIND_DEPTH_STENCIL,
                D3D11_CPU_ACCESS_FLAG, D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_CREATE_DEVICE_DEBUG,
                D3D11_DEPTH_STENCIL_VIEW_DESC, D3D11_DEPTH_STENCIL_VIEW_DESC_0,
                D3D11_DSV_DIMENSION_TEXTURE2D, D3D11_RENDER_TARGET_VIEW_DESC,
                D3D11_RENDER_TARGET_VIEW_DESC_0, D3D11_RESOURCE_MISC_FLAG,
                D3D11_RTV_DIMENSION_TEXTURE2D, D3D11_SDK_VERSION, D3D11_TEXTURE2D_DESC,
                D3D11_USAGE_DEFAULT, D3D11_VIEWPORT,
            },
            Dxgi::{
                Common::{
                    DXGI_ALPHA_MODE_IGNORE, DXGI_FORMAT_B8G8R8A8_UNORM,
                    DXGI_FORMAT_D24_UNORM_S8_UINT, DXGI_SAMPLE_DESC,
                },
                CreateDXGIFactory2, IDXGIFactory7, IDXGISwapChain1, DXGI_CREATE_FACTORY_DEBUG,
                DXGI_SCALING_NONE, DXGI_SWAP_CHAIN_DESC1, DXGI_SWAP_EFFECT_FLIP_DISCARD,
                DXGI_USAGE_RENDER_TARGET_OUTPUT,
            },
        },
    };

    use windows::core::Result as WinResult;

    pub const DEFAULT_WIDTH: u32 = 1280;
    pub const DEFAULT_HEIGHT: u32 = 720;

    pub struct DeviceResources {
        device: ID3D11Device,
        pub context: ID3D11DeviceContext,
        pub swapchain: IDXGISwapChain1,
        factory: IDXGIFactory7,
        pub viewport: D3D11_VIEWPORT,
        pub dsv: ID3D11DepthStencilView,
        pub rtv: ID3D11RenderTargetView,
    }

    impl DeviceResources {
        pub fn bind_to_wnd(hwnd: HWND) -> WinResult<Self> {
            let factory =
                unsafe { CreateDXGIFactory2::<IDXGIFactory7>(DXGI_CREATE_FACTORY_DEBUG)? };
            let mut device = None;
            let mut context = None;

            unsafe {
                D3D11CreateDevice(
                    None,
                    D3D_DRIVER_TYPE_HARDWARE,
                    None,
                    D3D11_CREATE_DEVICE_BGRA_SUPPORT | D3D11_CREATE_DEVICE_DEBUG,
                    None,
                    D3D11_SDK_VERSION,
                    Some(&mut device),
                    None,
                    Some(&mut context),
                )?;
            }

            let device = device.unwrap();
            let context = context.unwrap();

            let format = DXGI_FORMAT_B8G8R8A8_UNORM;

            let swapchain = unsafe {
                let desc = DXGI_SWAP_CHAIN_DESC1 {
                    Width: DEFAULT_WIDTH,
                    Height: DEFAULT_HEIGHT,
                    Format: format,
                    Stereo: BOOL(0),
                    SampleDesc: DXGI_SAMPLE_DESC {
                        Count: 1,
                        Quality: 0,
                    },
                    BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
                    BufferCount: 2,
                    Scaling: DXGI_SCALING_NONE,
                    SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
                    AlphaMode: DXGI_ALPHA_MODE_IGNORE,
                    Flags: 0,
                };
                factory.CreateSwapChainForHwnd(&device, hwnd, &desc, None, None)?
            };

            let viewport = D3D11_VIEWPORT {
                TopLeftX: 0f32,
                TopLeftY: 0f32,
                Width: DEFAULT_WIDTH as f32,
                Height: DEFAULT_HEIGHT as f32,
                MinDepth: 0f32,
                MaxDepth: 1f32,
            };

            let mut rtv = None;
            unsafe {
                let backbuffer = swapchain.GetBuffer::<ID3D11Texture2D>(0)?;
                let desc = D3D11_RENDER_TARGET_VIEW_DESC {
                    Format: format,
                    ViewDimension: D3D11_RTV_DIMENSION_TEXTURE2D,
                    Anonymous: D3D11_RENDER_TARGET_VIEW_DESC_0::default(),
                };
                device.CreateRenderTargetView(&backbuffer, Some(&desc), Some(&mut rtv))?
            }

            let mut dsv = None;
            unsafe {
                let mut depthbuffer = None;
                let desc = D3D11_TEXTURE2D_DESC {
                    Width: DEFAULT_WIDTH,
                    Height: DEFAULT_HEIGHT,
                    MipLevels: 1,
                    ArraySize: 1,
                    Format: DXGI_FORMAT_D24_UNORM_S8_UINT,
                    SampleDesc: DXGI_SAMPLE_DESC {
                        Count: 1,
                        Quality: 0,
                    },
                    Usage: D3D11_USAGE_DEFAULT,
                    BindFlags: D3D11_BIND_DEPTH_STENCIL,
                    CPUAccessFlags: D3D11_CPU_ACCESS_FLAG::default(),
                    MiscFlags: D3D11_RESOURCE_MISC_FLAG::default(),
                };

                device.CreateTexture2D(&desc, None, Some(&mut depthbuffer))?;

                let desc = D3D11_DEPTH_STENCIL_VIEW_DESC {
                    Format: DXGI_FORMAT_D24_UNORM_S8_UINT,
                    ViewDimension: D3D11_DSV_DIMENSION_TEXTURE2D,
                    Flags: 0,
                    Anonymous: D3D11_DEPTH_STENCIL_VIEW_DESC_0::default(),
                };

                device.CreateDepthStencilView(
                    &depthbuffer.unwrap(),
                    Some(&desc),
                    Some(&mut dsv),
                )?;
            }

            return Ok(DeviceResources {
                device: device,
                context: context,
                swapchain: swapchain,
                factory: factory,
                viewport: viewport,
                dsv: dsv.unwrap(),
                rtv: rtv.unwrap(),
            });
        }
    }
}
