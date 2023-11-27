#![warn(clippy::pedantic)]
#![feature(generic_const_exprs)]

use wgpu::util::DeviceExt;
use window::{
    BindGroup, BindGroupLayout, Pipeline, PipelineData, Renderer, VertexBuffer, VertexLayout,
    Window,
};
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::{Key, NamedKey};
use winit::window::WindowBuilder;

mod window;

#[cfg(feature = "egl")]
#[link(name = "EGL")]
#[link(name = "GLESv2")]
extern "C" {}

struct GridPipeline;

impl Pipeline for GridPipeline {
    fn create(device: &wgpu::Device, format: wgpu::TextureFormat) -> PipelineData {
        let shader = device.create_shader_module(wgpu::include_wgsl!("grid.wgsl"));
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&GridGlobals::layout(device)],
            push_constant_ranges: &[],
        });

        PipelineData::new(
            device,
            &wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[GridLineVertex::layout()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::LineList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
                depth_stencil: None,
            },
        )
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct GridLineVertex {
    position: [f32; 2],
}

impl GridLineVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![0 => Float32x2];
}

impl VertexLayout for GridLineVertex {
    type Pipeline = GridPipeline;

    fn layout() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct GridGlobals {
    grid_size: [f32; 2],
    window_size: [f32; 2],
}

impl BindGroupLayout for GridGlobals {
    type Pipeline = GridPipeline;
    const ENTRIES: usize = 1;

    fn layout_entries() -> [wgpu::BindGroupLayoutEntry; Self::ENTRIES] {
        [wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }]
    }
}

const GRID_SIZE: f32 = 3.;

impl GridGlobals {
    fn create(window: &mut Window) -> BindGroup<Self> {
        let globals = GridGlobals {
            grid_size: [GRID_SIZE, GRID_SIZE],
            window_size: [window.size.width as f32, window.size.height as f32],
        };

        dbg!(&globals);

        let buffer = window
            .renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&[globals]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        window
            .renderer
            .create_bind_group::<GridGlobals>([wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }])
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("WGPU fun")
        .build(&event_loop)?;
    let window_id = window.id();
    let mut window = Window::new(&window);

    let mut globals = GridGlobals::create(&mut window);

    let vertex_buffer = window.renderer.create_vertex_buffer(
        &(0..20)
            .map(|i| {
                let x = (i / 2) as f32 % GRID_SIZE;
                let y = if i % 2 == 0 { 0. } else { GRID_SIZE };
                GridLineVertex { position: [x, y] }
            })
            .chain((0..20).map(|i| {
                let y = (i / 2) as f32 % GRID_SIZE;
                let x = if i % 2 == 0 { 0. } else { GRID_SIZE };
                GridLineVertex { position: [x, y] }
            }))
            .chain([
                GridLineVertex { position: [3., 3.] },
                GridLineVertex { position: [0., 0.] },
            ])
            .collect::<Vec<_>>(),
    );

    event_loop.run(move |event, elwt| match event {
        winit::event::Event::WindowEvent {
            window_id: id,
            event,
        } if window_id == id => match event {
            WindowEvent::KeyboardInput {
                event: KeyEvent {
                    logical_key, state, ..
                },
                ..
            } => match (logical_key, state) {
                (Key::Named(NamedKey::Escape), ElementState::Pressed) => elwt.exit(),
                _ => {}
            },
            WindowEvent::CloseRequested => elwt.exit(),
            WindowEvent::Resized(size) => {
                window.resize(size);
                globals = GridGlobals::create(&mut window);
            }
            WindowEvent::RedrawRequested => {
                let result = window.renderer.with_encoder(|mut encoder| {
                    let view = encoder.view();
                    let mut render_pass = encoder
                        .render_pass(&wgpu::RenderPassDescriptor {
                            label: None,
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: &view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(wgpu::Color {
                                        r: 0.0,
                                        g: 0.0,
                                        b: 0.0,
                                        a: 1.0,
                                    }),
                                    store: wgpu::StoreOp::Store,
                                },
                            })],
                            depth_stencil_attachment: None,
                            occlusion_query_set: None,
                            timestamp_writes: None,
                        })
                        .step::<GridPipeline>();

                    render_pass
                        .render_pass
                        .set_bind_group(0, globals.as_ref(), &[]);

                    render_pass.draw(&vertex_buffer);
                });

                if let Err(err) = result {
                    match err {
                        wgpu::SurfaceError::Lost => {
                            window.resize(window.size);
                        }
                        wgpu::SurfaceError::OutOfMemory => elwt.exit(),
                        _ => eprintln!("{err:?}"),
                    }
                }
            }
            _ => {}
        },
        _ => {}
    })?;

    Ok(())
}
