use std::path::Path;

use wgpu::{
    Color, CommandEncoderDescriptor, FragmentState, PipelineLayoutDescriptor, PrimitiveState,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    ShaderModuleDescriptor, TextureViewDescriptor, VertexState, MultisampleState, ShaderSource, TextureFormat,
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
    let rendering = setup_pipeline(&gpu_state);
    rendering.render(&gpu_state);

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

struct Rendering {
    pipeline: wgpu::RenderPipeline,
}

impl Rendering {
    fn render(self: &Rendering, state: &GpuState) {
        let mut encoder = state
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());

        let texture = state
            .surface
            .get_current_texture()
            .expect("Cannot obtain texture from the surface");
        let texture_view = texture
            .texture
            .create_view(&TextureViewDescriptor::default());

        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
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

            pass.set_pipeline(&self.pipeline);
            pass.draw(0..3, 0..1);
        }

        let command_buffers = [encoder.finish()];
        state.queue.submit(command_buffers);
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
            width: 80,
            height: 60,
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

fn setup_pipeline(state: &GpuState) -> Rendering {
    let pipeline_layout = state
        .device
        .create_pipeline_layout(&PipelineLayoutDescriptor::default()); //&PipelineLayoutDescriptor { label: (), bind_group_layouts: (), push_constant_ranges: () });

    let source = ShaderSource::Wgsl(include_str!("main.wgsl").into());
    let module = state.device.create_shader_module(ShaderModuleDescriptor {
        label: Some("Main shader module"),
        source,
    });
    let vertex = VertexState {
        module: &module,
        entry_point: "vertex",
        buffers: &[],
    };
    let fragment = FragmentState {
        module: &module,
        entry_point: "fragment",
        targets: &[Some(TextureFormat::Bgra8Unorm.into())],
    };

    let primitive= PrimitiveState {
        topology: wgpu::PrimitiveTopology::TriangleList,
        strip_index_format: None,
        front_face: wgpu::FrontFace::Ccw,
        cull_mode: None,
        unclipped_depth: false,
        polygon_mode: wgpu::PolygonMode::Fill, // Line
        conservative: false,
    };

    let pipeline = state
        .device
        .create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Main render pipeline"),
            layout: None, // Some(&pipeline_layout);
            vertex: vertex,
            primitive,
            depth_stencil: None,
            multisample: MultisampleState::default(),
            fragment: Some(fragment),
            multiview: None,
        });

    Rendering { pipeline }
}
