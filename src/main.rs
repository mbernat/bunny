use core::slice;
use std::{fs::File, io::BufReader, mem::size_of, path::Path};

use wgpu::util::DeviceExt;
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

fn cast_to_bytes<T>(slice: &[T]) -> &[u8] {
    // SAFETY: input slice contains valid data and outlives the return value
    unsafe { slice::from_raw_parts(slice.as_ptr() as *const u8, size_of::<T>() * slice.len()) }
}

impl Rendering {
    fn render(self: &Rendering, state: &GpuState) {
        use wgpu::*;

        let input = BufReader::new(File::open("bunny.obj").expect("bunny.obj not found"));
        let bunny: obj::Obj<obj::Position> = obj::load_obj(input).expect("Not a valid .obj file");

        let positions = &bunny.vertices;
        let vertex_buffer = state
            .device
            .create_buffer_init(&util::BufferInitDescriptor {
                label: Some("Main vertex buffer"),
                contents: cast_to_bytes(positions),
                usage: BufferUsages::VERTEX,
            });

        let indices = &bunny.indices[..];
        let index_buffer = state
            .device
            .create_buffer_init(&util::BufferInitDescriptor {
                label: Some("Main index buffer"),
                contents: cast_to_bytes(indices),
                usage: BufferUsages::INDEX,
            });

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
            pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            pass.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint16);
            pass.draw_indexed(0..(indices.len() as u32), 0, 0..2);
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
            width: 800,
            height: 600,
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
    use wgpu::*;

    /*
    // For bind groups
    let pipeline_layout = state
        .device
        .create_pipeline_layout(&PipelineLayoutDescriptor::default());
    */

    let source = ShaderSource::Wgsl(include_str!("main.wgsl").into());
    let module = state.device.create_shader_module(ShaderModuleDescriptor {
        label: Some("Main shader module"),
        source,
    });

    let vertex_layout = VertexBufferLayout {
        array_stride: 12,
        step_mode: VertexStepMode::Vertex,
        attributes: &[VertexAttribute {
            format: VertexFormat::Float32x3,
            offset: 0,
            shader_location: 0,
        }],
    };

    let vertex = VertexState {
        module: &module,
        entry_point: "vertex",
        buffers: &[vertex_layout],
    };
    let fragment = FragmentState {
        module: &module,
        entry_point: "fragment",
        targets: &[Some(TextureFormat::Bgra8Unorm.into())],
    };

    let primitive = PrimitiveState {
        topology: wgpu::PrimitiveTopology::TriangleList,
        strip_index_format: None,
        front_face: wgpu::FrontFace::Ccw,
        cull_mode: Some(Face::Back),
        unclipped_depth: false,
        polygon_mode: wgpu::PolygonMode::Fill, // Line
        conservative: false,
    };

    let pipeline = state
        .device
        .create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Main render pipeline"),
            layout: None, // Some(&pipeline_layout);
            vertex,
            primitive,
            depth_stencil: None,
            multisample: MultisampleState::default(),
            fragment: Some(fragment),
            multiview: None,
        });

    Rendering { pipeline }
}
