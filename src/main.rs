use std::path::Path;

use wgpu::{
    Color, CommandBuffer, CommandEncoder, CommandEncoderDescriptor, RenderPassColorAttachment,
    RenderPassDescriptor, TextureView, TextureViewDescriptor,
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let gpu_state = pollster::block_on(setup_wgpu(&window));
    gpu_state.render();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            _ => (),
        }
    });
}

struct GpuState {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl GpuState {
    fn render(&self) {
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());

        let texture = self
            .surface
            .get_current_texture()
            .expect("Cannot obtain texture from the surface");
        let texture_view = texture
            .texture
            .create_view(&TextureViewDescriptor::default());

        encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Main render pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(Color::BLACK),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        let command_buffers = [encoder.finish()];
        self.queue.submit(command_buffers);
        texture.present();
    }
}

async fn setup_wgpu(window: &Window) -> GpuState {
    use wgpu::*;
    let instance = Instance::new(InstanceDescriptor::default());
    let adapter = instance
        .request_adapter(&RequestAdapterOptions::default())
        .await
        .expect("No appropriate adapter found");

    // SAFETY: window must be valid for the lifetime of surface
    let surface = unsafe { instance.create_surface(window) }.expect("Could not create surface");

    let (device, queue) = adapter
        .request_device(&DeviceDescriptor::default(), Some(Path::new("trace")))
        .await
        .expect("No appropriate device found");

    surface.configure(
        &device,
        &SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: TextureFormat::Bgra8Unorm,
            width: 10,
            height: 20,
            present_mode: PresentMode::Fifo,
            alpha_mode: CompositeAlphaMode::Auto,
            view_formats: vec![],
        },
    );

    GpuState {
        surface,
        device,
        queue,
    }
}
