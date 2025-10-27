use std::sync::Arc;

use winit::{
    event::WindowEvent,
    event_loop::ControlFlow,
    application::ApplicationHandler, 
    event_loop::EventLoop,
    window::{Window, WindowAttributes}
};

struct State {
    window: Arc<Window>,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    width: u32,
    height: u32,
    queue: wgpu::Queue,
}

impl State {
    async fn new(window: Arc<Window>) -> State {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions::default())
                .await
                .unwrap();

        let (device, queue) = adapter.request_device(&Default::default()).await.unwrap();

        let surface = instance
            .create_surface(window.clone())
            .unwrap();

        let capabilities = surface.get_capabilities(&adapter);
        let surface_format = capabilities.formats[0];

        let surface_size = window.inner_size();

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            view_formats: vec![surface_format.add_srgb_suffix()],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            width: surface_size.width,
            height: surface_size.height,
            desired_maximum_frame_latency: 2,
            present_mode: wgpu::PresentMode::AutoVsync,
        };

        surface.configure(&device, &surface_config);

        State {
            window,
            adapter,
            device,
            width: surface_size.width,
            height: surface_size.height,
            surface_config,
            surface,
            queue
        }
    }

    fn render(&mut self)
    {
        let surface_texture = self.surface.get_current_texture().unwrap();
        let texture_view = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(self.surface_config.format.add_srgb_suffix()),
            ..Default::default()
        });

        let mut encoder = self.device.create_command_encoder(&Default::default());

        let renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor { 
            label: None, 
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &texture_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations { 
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLUE), 
                    store: wgpu::StoreOp::Store
                },
            })], 
            depth_stencil_attachment: None, 
            timestamp_writes: None, 
            occlusion_query_set: None 
        });

        // draw

        drop(renderpass);

        self.queue.submit([encoder.finish()]);
        self.window.pre_present_notify();
        surface_texture.present();
    }
}

#[derive(Default)]
struct App {
    state: Option<State>,
}

impl ApplicationHandler for App {
    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let state = self.state.as_mut().unwrap();
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                state.render();

                state.window.request_redraw();
            },
            _ => (),
        }
    }
    
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = Arc::new(
            event_loop.create_window(WindowAttributes::default()).unwrap()
        );
        
        let state = pollster::block_on(State::new(window.clone()));
        self.state = Some(state);

        window.request_redraw();
    }
}
fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    
    let mut app = App::default();
    
    event_loop.run_app(&mut app).unwrap();
}
