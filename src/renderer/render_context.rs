pub struct RenderContext {
    pub window: winit::window::Window,
    pub instance: wgpu::Instance,
    pub surface: wgpu::Surface,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl RenderContext {
    pub async fn new(event_loop: &winit::event_loop::EventLoop<()>) -> RenderContext {
        let window = winit::window::WindowBuilder::new()
            .with_title("rustvoxels")
            .build(&event_loop)
            .unwrap();

        window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));

        let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
        let surface = unsafe { instance.create_surface(&window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::SPIRV_SHADER_PASSTHROUGH,
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();
        let context = Self {
            window,
            instance,
            surface,
            adapter,
            device,
            queue,
        };

        context
    }
}
